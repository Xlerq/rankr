//! KNF collection workflow: archive every response before parsing any of its rows.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

use crate::{
    model::{Instrument, RawDocument, ShortsSnapshot},
    shorts::{latest_positions, parse_page},
    source::KnfClient,
    storage::archive_raw,
};

pub async fn collect_one(client: &KnfClient, instrument: &Instrument, output: &Path) -> Result<()> {
    let path = collect_company(instrument, output, &mut async |instrument, offset| {
        Ok(client.fetch_shorts(instrument, offset).await?)
    })
    .await?;
    eprintln!("{}: {}", instrument.code, path.display());
    Ok(())
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
        let mut positions = first.positions;
        let mut fetched_at = raw.fetched_at;
        let mut page_number = 2;
        while positions.len() < total {
            // Offset counts source rows, not deduplicated holders.
            let next = fetch(instrument, positions.len()).await?;
            archived.save_raw_page(page_number, &next)?;
            let page = parse_page(&next)?;
            ensure!(
                page.total == total,
                "KNF total changed during pagination; retry collection"
            );
            ensure!(
                page.positions.len() <= total - positions.len(),
                "KNF returned more rows than its total"
            );
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
