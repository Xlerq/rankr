//! Per-company collection shared by the CLI and the daily refresh.

use std::path::Path;

use anyhow::{Context, Result};

use crate::{
    history::{YahooSymbolMap, parse_history_document},
    model::{Instrument, RawDocument},
    parser::parse_document,
    source::{GpwClient, YahooClient},
    storage::archive_raw,
};

pub async fn collect_fundamentals(
    client: &GpwClient,
    instrument: &Instrument,
    output: &Path,
) -> Result<()> {
    let raw = client.fetch_fundamentals(instrument).await?;
    let archived = archive_raw(output, &raw).context("archiving raw response")?;
    let parsed = match parse_document(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            archived.save_error(&format!("{error:#}"))?;
            return Err(anyhow::Error::new(error)
                .context(format!("saved response: {}", archived.raw_path().display())));
        }
    };
    let path = archived.save_snapshot(&parsed)?;
    eprintln!(
        "{}: {} -> {}",
        instrument.code,
        parsed.fundamentals.report_period,
        path.display()
    );
    Ok(())
}

pub async fn collect_history(
    client: &YahooClient,
    symbols: &YahooSymbolMap,
    instrument: &Instrument,
    output: &Path,
    latest_only: bool,
) -> Result<()> {
    let symbol = symbols.symbol_for(instrument)?;
    let raw = if latest_only {
        client.fetch_recent_history(instrument, symbol).await?
    } else {
        client.fetch_history(instrument, symbol).await?
    };
    archive_history(&raw, output, latest_only)
}

fn archive_history(raw: &RawDocument, output: &Path, latest_only: bool) -> Result<()> {
    let archived = archive_raw(output, raw).context("archiving daily price response")?;
    let mut history = match parse_history_document(raw) {
        Ok(history) => history,
        Err(error) => {
            archived.save_error(&error.to_string())?;
            return Err(anyhow::Error::new(error)
                .context(format!("saved response: {}", archived.raw_path().display())));
        }
    };
    if latest_only {
        let last = history.candles.pop().expect("nonempty history");
        history.candles.clear();
        history.candles.push(last);
    }
    let path = archived.save_history(&history)?;
    if !history.skipped_candles.is_empty() {
        eprintln!(
            "{}: omitted {} candles; dates and reasons are recorded in skipped_candles in {}",
            history.instrument.code,
            history.skipped_candles.len(),
            path.display()
        );
    }
    // The parser rejects an empty series; report actual coverage, including IPOs.
    eprintln!(
        "{}: {} daily candles, {}..{} -> {}",
        history.instrument.code,
        history.candles.len(),
        history.candles.first().expect("nonempty history").date,
        history.candles.last().expect("nonempty history").date,
        path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::Source, storage::read_raw};

    fn yahoo_raw() -> RawDocument {
        RawDocument {
            source: Source::YahooFinance,
            instrument: Some(
                YahooSymbolMap::bundled()
                    .unwrap()
                    .instrument("KGHM")
                    .unwrap(),
            ),
            fetched_at: chrono::Utc::now(),
            url: "https://query1.finance.yahoo.com/v8/finance/chart/KGH.WA?interval=1d&range=1mo"
                .into(),
            http_status: 200,
            body: include_str!("../tests/fixtures/yahoo_daily.json").into(),
        }
    }

    #[test]
    fn company_prices_and_history_archive_the_same_daily_model() {
        for (latest_only, count) in [(true, 1), (false, 3)] {
            let directory = tempfile::tempdir().unwrap();
            let raw = yahoo_raw();
            archive_history(&raw, directory.path(), latest_only).unwrap();
            let observation = std::fs::read_dir(directory.path())
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            assert_eq!(read_raw(&observation.join("raw.json")).unwrap(), raw);
            let history: crate::model::DailyPriceHistory =
                serde_json::from_slice(&std::fs::read(observation.join("history.json")).unwrap())
                    .unwrap();
            assert_eq!(history.candles.len(), count);
            assert_eq!(history.source, Source::YahooFinance);
            assert_eq!(
                history.candles.last().unwrap().date.to_string(),
                "2024-01-04"
            );
            assert!(history.candles.last().unwrap().adjusted_close.is_some());
            assert!(!observation.join("price.json").exists());
        }
    }

    #[test]
    fn yahoo_failures_preserve_raw_and_error_without_a_daily_history() {
        for (status, body) in [
            (429, "Too Many Requests"),
            (200, "<html>Access denied</html>"),
            (
                200,
                r#"{"chart":{"result":null,"error":{"code":"Not Found","description":"No data"}}}"#,
            ),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let mut raw = yahoo_raw();
            raw.http_status = status;
            raw.body = body.into();
            assert!(archive_history(&raw, directory.path(), true).is_err());
            let observation = std::fs::read_dir(directory.path())
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            assert_eq!(read_raw(&observation.join("raw.json")).unwrap(), raw);
            assert!(observation.join("error.json").exists());
            assert!(!observation.join("history.json").exists());
        }
    }

    #[test]
    fn history_archives_raw_http_and_parse_failures_without_a_partial_history() {
        for (status, body) in [
            (403, "access denied"),
            (200, "Access denied"),
            (
                200,
                "<html><noscript>This site requires JavaScript to verify your browser.</noscript></html>",
            ),
            (
                200,
                "Date,Open,High,Low,Close,Volume\n2024-01-02,10,1,2,5,100\n",
            ),
        ] {
            let directory = tempfile::tempdir().unwrap();
            let raw = RawDocument {
                source: Source::Stooq,
                instrument: Some(Instrument::wig20_index()),
                fetched_at: chrono::Utc::now(),
                url: "https://stooq.com/q/d/l/?s=wig20&i=d".into(),
                http_status: status,
                body: body.into(),
            };
            assert!(archive_history(&raw, directory.path(), false).is_err());
            let entries: Vec<_> = std::fs::read_dir(directory.path()).unwrap().collect();
            assert_eq!(entries.len(), 1);
            let archived = entries[0].as_ref().unwrap().path();
            assert_eq!(read_raw(&archived.join("raw.json")).unwrap(), raw);
            assert!(archived.join("error.json").exists());
            assert!(!archived.join("history.json").exists());
        }
    }
}
