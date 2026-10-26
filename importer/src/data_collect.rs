//! Per-company fundamentals and incremental daily history collection.

use std::{
    collections::BTreeMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};
use chrono::{DateTime, Utc};

use crate::{
    history::{YahooSymbolMap, parse_history_document},
    model::{DailyPriceHistory, Instrument},
    parser::parse_document,
    source::{GpwClient, YahooClient},
    storage::{archive_raw, replace_history},
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

pub(crate) struct SavedHistory {
    pub path: PathBuf,
    pub history: DailyPriceHistory,
}

pub(crate) fn latest_histories(files: &[PathBuf]) -> BTreeMap<String, SavedHistory> {
    let mut selected: BTreeMap<String, SavedHistory> = BTreeMap::new();
    // The file list is sorted: tied fetched_at values keep the first lexical path,
    // as in the ranking's existing history selection.
    for path in files {
        if path.file_name().is_none_or(|name| name != "history.json") {
            continue;
        }
        let history = match read_history(path) {
            Ok(history) => history,
            Err(error) => {
                eprintln!("{}: {error:#}", path.display());
                continue;
            }
        };
        let code = history.instrument.code.clone();
        if selected
            .get(&code)
            .is_some_and(|current| current.history.fetched_at >= history.fetched_at)
        {
            continue;
        }
        selected.insert(
            code,
            SavedHistory {
                path: path.clone(),
                history,
            },
        );
    }
    selected
}

fn read_history(path: &Path) -> Result<DailyPriceHistory> {
    let reader = BufReader::new(File::open(path)?);
    Ok(serde_json::from_reader(reader)?)
}

pub(crate) async fn collect_history(
    client: &YahooClient,
    symbols: &YahooSymbolMap,
    instrument: &Instrument,
    output: &Path,
    saved: Option<SavedHistory>,
    now: DateTime<Utc>,
) -> Result<()> {
    let symbol = symbols.symbol_for(instrument)?;
    let last_date = saved
        .as_ref()
        .and_then(|saved| saved.history.candles.iter().map(|candle| candle.date).max());
    if let Some(saved) = &saved {
        ensure!(
            saved.history.instrument.isin == instrument.isin && saved.history.symbol == symbol,
            "stored history does not match {} ({})",
            instrument.code,
            instrument.isin
        );
    }
    // There cannot be a finished session after today. Avoid sending inverted bounds.
    if last_date.is_some_and(|date| date >= now.date_naive()) {
        eprintln!("{}: no new sessions", instrument.code);
        return Ok(());
    }
    let raw = client
        .fetch_history(instrument, symbol, last_date, now)
        .await?;
    let parsed = if saved.is_some() {
        crate::yahoo::parse_history(&raw, true)
    } else {
        parse_history_document(&raw)
    };
    let mut history = match parsed {
        Ok(history) => history,
        Err(error) => {
            let archived = archive_raw(output, &raw)?;
            archived.save_error(&error.to_string())?;
            return Err(anyhow::Error::new(error)
                .context(format!("saved response: {}", archived.raw_path().display())));
        }
    };
    // Providers may repeat the last session at a range boundary; keep stored prices.
    history
        .candles
        .retain(|candle| last_date.is_none_or(|last| candle.date > last));
    let added = history.candles.len();
    if added == 0 {
        eprintln!("{}: no new sessions", instrument.code);
        return Ok(());
    }
    let path = saved.as_ref().map(|saved| saved.path.clone());
    if let Some(mut saved) = saved {
        saved.history.candles.extend(history.candles);
        saved
            .history
            .skipped_candles
            .extend(history.skipped_candles);
        saved.history.fetched_at = history.fetched_at;
        saved.history.source_url = history.source_url;
        history = saved.history;
    }
    history.candles.sort_by_key(|candle| candle.date);
    history.candles.dedup_by_key(|candle| candle.date);
    history.skipped_candles.retain(|skipped| {
        history
            .candles
            .binary_search_by_key(&skipped.date, |candle| candle.date)
            .is_err()
    });
    history.skipped_candles.sort_by_key(|candle| candle.date);
    history.skipped_candles.dedup_by_key(|candle| candle.date);

    // Nothing is written for an empty update. A nonempty update retains its raw
    // response and atomically replaces the same history.json instead of copying it.
    let archived = archive_raw(output, &raw)?;
    let path = match path {
        Some(path) => {
            replace_history(&path, &history)?;
            path
        }
        None => archived.save_history(&history)?,
    };
    eprintln!(
        "{}: added {added} daily candles -> {}",
        instrument.code,
        path.display()
    );
    Ok(())
}
