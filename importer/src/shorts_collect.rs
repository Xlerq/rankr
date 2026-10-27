//! KNF collection workflow: archive every response before parsing any of its rows.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};

use crate::{
    model::{Instrument, RawDocument, ShortsSnapshot},
    shorts::{latest_positions, parse_page},
    source::KnfClient,
    storage::archive_raw,
};

pub async fn collect(client: &KnfClient, instruments: &[Instrument], output: &Path) -> usize {
    collect_with(instruments, output, async |offset| {
        Ok(client.fetch_shorts(offset).await?)
    })
    .await
}

async fn collect_with(
    instruments: &[Instrument],
    output: &Path,
    mut fetch: impl AsyncFnMut(usize) -> Result<RawDocument>,
) -> usize {
    let mut pages: BTreeMap<usize, RawDocument> = BTreeMap::new();
    let mut failure: Option<String> = None;
    let mut fetch_cached = async |instrument: &Instrument, offset| {
        if let Some(error) = &failure {
            anyhow::bail!("{error}");
        }
        let mut raw = match pages.get(&offset) {
            Some(raw) => raw.clone(),
            None => match fetch(offset).await {
                Ok(raw) => {
                    pages.insert(offset, raw.clone());
                    raw
                }
                Err(error) => {
                    failure = Some(format!("{error:#}"));
                    return Err(error);
                }
            },
        };
        raw.instrument = Some(instrument.clone());
        Ok(raw)
    };
    let mut saved = 0;
    for instrument in instruments {
        match collect_company(instrument, output, &mut fetch_cached).await {
            Ok(path) => {
                saved += 1;
                eprintln!("{}: {}", instrument.code, path.display());
            }
            Err(error) => eprintln!("{}: {error:#}", instrument.code),
        }
    }
    saved
}

async fn collect_company(
    instrument: &Instrument,
    output: &Path,
    fetch: &mut impl AsyncFnMut(&Instrument, usize) -> Result<RawDocument>,
) -> Result<PathBuf> {
    let raw = fetch(instrument, 0)
        .await
        .context("fetching KNF register")?;
    let archived = archive_raw(output, &raw).context("archiving KNF response")?;

    let result: Result<PathBuf> = async {
        let first = parse_page(&raw)?;
        let total = first.total;
        let mut offset = first.records_count;
        let mut positions = first.positions;
        let mut fetched_at = raw.fetched_at;
        let mut page_number = 2;
        while offset < total {
            // Offset counts source rows, not deduplicated holders.
            let next = fetch(instrument, offset).await?;
            archived.save_raw_page(page_number, &next)?;
            let page = parse_page(&next)?;
            ensure!(
                page.total == total,
                "KNF total changed during pagination; retry collection"
            );
            ensure!(
                page.records_count <= total - offset,
                "KNF returned more rows than its total"
            );
            offset += page.records_count;
            positions.extend(page.positions);
            fetched_at = fetched_at.max(next.fetched_at);
            page_number += 1;
        }

        let snapshot = ShortsSnapshot {
            code: instrument.code.clone(),
            isin: instrument.isin.clone(),
            fetched_at,
            positions: latest_positions(positions),
        };
        // Publish only a complete observation. Failure leaves raw pages and error.json.
        Ok(archived.save_shorts(&snapshot)?)
    }
    .await;

    if let Err(error) = &result {
        archived.save_error(&format!("{error:#}"))?;
    }
    result.with_context(|| format!("saved response: {}", archived.raw_path().display()))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::{TimeZone, Utc};
    use serde_json::Value;

    use super::*;
    use crate::model::Source;

    const POSITIONS: &str = include_str!("../tests/fixtures/knf_shorts.json");
    const EMPTY: &str = include_str!("../tests/fixtures/knf_empty.json");

    fn read_raw(path: &Path) -> serde_json::Result<RawDocument> {
        serde_json::from_slice(&fs::read(path).unwrap())
    }

    fn raw(instrument: &Instrument, body: &str) -> RawDocument {
        RawDocument {
            source: Source::KnfShorts,
            instrument: Some(instrument.clone()),
            fetched_at: Utc.with_ymd_and_hms(2026, 10, 21, 12, 0, 0).unwrap(),
            url: "https://rss.knf.gov.pl/rss_pub/JSON".into(),
            http_status: 200,
            body: body.into(),
        }
    }

    fn budimex() -> Instrument {
        crate::history::YahooSymbolMap::bundled()
            .unwrap()
            .instrument("BUDIMEX")
            .unwrap()
    }

    fn snapshot(directory: &Path) -> ShortsSnapshot {
        serde_json::from_slice(&fs::read(directory.join("shorts.json")).unwrap()).unwrap()
    }

    fn page(offset: usize) -> String {
        let mut json: Value = serde_json::from_str(POSITIONS).unwrap();
        let records = json["records"].as_array().unwrap();
        json["records"] = Value::Array(records[offset..offset + 2].to_vec());
        json.to_string()
    }

    #[tokio::test]
    async fn fetches_each_register_page_once_and_filters_companies_locally() {
        let root = tempfile::tempdir().unwrap();
        let companies = [
            crate::history::YahooSymbolMap::bundled()
                .unwrap()
                .instrument("KGHM")
                .unwrap(),
            budimex(),
        ];
        let mut offsets = Vec::new();
        let saved = collect_with(&companies, root.path(), async |offset| {
            offsets.push(offset);
            let mut response = raw(&companies[1], &page(offset));
            response.instrument = None;
            Ok(response)
        })
        .await;
        assert_eq!(saved, 2);
        assert_eq!(offsets, [0, 2]);
        let kghm = root.path().join("2026-10-21_KGHM_shorts");
        let budimex = root.path().join("2026-10-21_BUDIMEX_shorts");
        assert!(snapshot(&kghm).positions.is_empty());
        assert_eq!(snapshot(&budimex).positions.len(), 2);
        assert_eq!(read_raw(&kghm.join("raw.json")).unwrap().body, page(0));
        assert_eq!(
            read_raw(&budimex.join("raw_page_2.json")).unwrap().body,
            page(2)
        );
    }

    #[tokio::test]
    async fn a_shared_network_failure_is_not_repeated_and_preserves_existing_snapshots() {
        let root = tempfile::tempdir().unwrap();
        let company = budimex();
        let mut previous = raw(&company, EMPTY);
        previous.fetched_at -= chrono::Duration::days(1);
        let archived = archive_raw(root.path(), &previous).unwrap();
        let old_path = archived
            .save_shorts(&ShortsSnapshot {
                code: company.code.clone(),
                isin: company.isin.clone(),
                fetched_at: previous.fetched_at,
                positions: Vec::new(),
            })
            .unwrap();
        let before = fs::read(&old_path).unwrap();
        let mut requests = 0;
        let companies = [
            company,
            crate::history::YahooSymbolMap::bundled()
                .unwrap()
                .instrument("KGHM")
                .unwrap(),
        ];
        let saved = collect_with(&companies, root.path(), async |_| {
            requests += 1;
            anyhow::bail!("KNF request timed out");
        })
        .await;
        assert_eq!(saved, 0);
        assert_eq!(requests, 1);
        assert_eq!(fs::read(old_path).unwrap(), before);
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), 1);
    }

    #[tokio::test]
    async fn invalid_json_is_archived_before_parsing_and_never_published_as_empty() {
        let root = tempfile::tempdir().unwrap();
        let company = budimex();
        let mut fetch = async |instrument: &Instrument, _| Ok(raw(instrument, "{broken JSON"));
        assert!(
            collect_company(&company, root.path(), &mut fetch)
                .await
                .is_err()
        );
        let directory = root.path().join("2026-10-21_BUDIMEX_shorts");
        assert_eq!(
            read_raw(&directory.join("raw.json")).unwrap().body,
            "{broken JSON"
        );
        assert!(directory.join("error.json").exists());
        assert!(!directory.join("shorts.json").exists());
    }

    #[tokio::test]
    async fn archives_pages_before_parsing_and_deduplicates_across_pages() {
        let root = tempfile::tempdir().unwrap();
        let company = budimex();
        let directory = root.path().join("2026-10-21_BUDIMEX_shorts");
        let mut offsets = Vec::new();
        let mut fetch = async |instrument: &Instrument, offset| {
            offsets.push(offset);
            if offset > 0 {
                assert_eq!(read_raw(&directory.join("raw.json")).unwrap().body, page(0));
                assert!(!directory.join("shorts.json").exists());
            }
            let mut response = raw(instrument, &page(offset));
            response.fetched_at += chrono::Duration::seconds(offset as i64);
            Ok(response)
        };
        collect_company(&company, root.path(), &mut fetch)
            .await
            .unwrap();
        assert_eq!(offsets, [0, 2]);
        assert_eq!(
            read_raw(&directory.join("raw_page_2.json")).unwrap().body,
            page(2)
        );
        let snapshot = snapshot(&directory);
        assert_eq!(
            snapshot.fetched_at,
            Utc.with_ymd_and_hms(2026, 10, 21, 12, 0, 2).unwrap()
        );
        assert_eq!(
            snapshot.positions,
            latest_positions(parse_page(&raw(&company, POSITIONS)).unwrap().positions)
        );
        assert_eq!(snapshot.positions.len(), 2);
    }

    #[tokio::test]
    async fn a_failed_or_inconsistent_later_page_does_not_publish_a_partial_snapshot() {
        let company = budimex();
        for failure in ["http", "network", "json", "changed_total", "empty_page"] {
            let root = tempfile::tempdir().unwrap();
            let mut fetch = async |instrument: &Instrument, offset| {
                let mut response = raw(instrument, &page(offset));
                if offset > 0 {
                    match failure {
                        "http" => response.http_status = 503,
                        "network" => anyhow::bail!("connection lost"),
                        "json" => response.body = "{broken JSON".into(),
                        "changed_total" => {
                            response.body = response.body.replace("\"total\":4", "\"total\":5")
                        }
                        "empty_page" => {
                            response.body = r#"{"status":"success","total":4,"records":[]}"#.into()
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(response)
            };
            assert!(
                collect_company(&company, root.path(), &mut fetch)
                    .await
                    .is_err(),
                "{failure}"
            );
            let directory = root.path().join("2026-10-21_BUDIMEX_shorts");
            assert!(directory.join("raw.json").exists());
            assert_eq!(
                directory.join("raw_page_2.json").exists(),
                failure != "network"
            );
            assert!(directory.join("error.json").exists());
            assert!(!directory.join("shorts.json").exists());
        }
    }

    #[tokio::test]
    async fn repeated_collections_preserve_both_snapshots() {
        let root = tempfile::tempdir().unwrap();
        let company = budimex();
        let mut fetch = async |instrument: &Instrument, _| Ok(raw(instrument, POSITIONS));
        collect_company(&company, root.path(), &mut fetch)
            .await
            .unwrap();
        let first = root.path().join("2026-10-21_BUDIMEX_shorts");
        let first_bytes = fs::read(first.join("shorts.json")).unwrap();
        let mut fetch = async |instrument: &Instrument, _| Ok(raw(instrument, EMPTY));
        collect_company(&company, root.path(), &mut fetch)
            .await
            .unwrap();
        let second = root.path().join("2026-10-21_BUDIMEX_shorts_2");
        assert_eq!(fs::read(first.join("shorts.json")).unwrap(), first_bytes);
        assert_eq!(snapshot(&first).positions.len(), 2);
        assert!(snapshot(&second).positions.is_empty());
    }
}
