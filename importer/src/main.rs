use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use rankr_import::{
    history::{StooqSymbolMap, parse_history_document},
    model::{Instrument, MarketPair, RawDocument, Source},
    parser::parse_document,
    prices::parse_price_document,
    source::{GpwClient, SourceError, StooqClient, TradingViewClient, parse_portfolio},
    storage::{archive_raw, read_raw},
};
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "rankr-import",
    version,
    about = "Collect GPW fundamentals, current quotes and Stooq daily history as JSON"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Archive fundamentals; omit CODE to collect the current WIG20.
    Collect {
        /// GPW company code, e.g. KGHM, PKOBP, PZU (case-insensitive).
        code: Option<String>,
        /// Directory for immutable observations.
        #[arg(short, long, default_value = "data/collected")]
        output: PathBuf,
    },
    /// Archive WIG20 prices, USD/PLN, EUR/PLN and XAU/USD, or one CODE.
    Prices {
        /// GPW company code or pair, e.g. KGHM, USDPLN, EUR/PLN, XAU/USD.
        code: Option<String>,
        /// Directory for immutable observations.
        #[arg(short, long, default_value = "data/collected")]
        output: PathBuf,
    },
    /// Archive Stooq daily history; omit CODE for the current WIG20 plus its index.
    History {
        /// GPW company code, e.g. KGHM or PKOBP, mapped using bundled identifiers.
        code: Option<String>,
        /// Directory for immutable observations. Requires STOOQ_API_KEY in env/.env.
        #[arg(short, long, default_value = "data/collected")]
        output: PathBuf,
    },
    /// Write one company's raw response as JSON to stdout.
    Fetch {
        /// GPW company code from the current WIG20.
        code: String,
    },
    /// Parse a raw JSON file offline as fundamentals, a quote or daily history.
    Parse {
        /// raw.json produced by collect, prices, history or fetch.
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Parse { file } => {
            let raw = read_raw(&file).with_context(|| format!("reading {}", file.display()))?;
            match raw.source {
                Source::Stooq => print_json(&parse_history_document(&raw)?),
                Source::GpwPrices | Source::TradingViewIdc => {
                    print_json(&parse_price_document(&raw)?)
                }
                _ => print_json(&parse_document(&raw)?),
            }
        }
        Command::Fetch { code } => {
            let client = GpwClient::new()?;
            let raw_portfolio = client.fetch_portfolio().await?;
            let instruments = select(portfolio(&raw_portfolio)?, Some(&code))?;
            let raw = client.fetch_fundamentals(&instruments[0]).await?;
            // Keep an HTTP error response inspectable through shell redirection.
            print_json(&raw)?;
            check_http(&raw)
        }
        Command::Collect { code, output } => collect(code.as_deref(), output).await,
        Command::Prices { code, output } => collect_prices(code.as_deref(), output).await,
        Command::History { code, output } => collect_history(code.as_deref(), output).await,
    }
}

fn stooq_api_key() -> Result<String> {
    match std::env::var("STOOQ_API_KEY") {
        Ok(value) => return Ok(value),
        Err(std::env::VarError::NotUnicode(_)) => bail!("STOOQ_API_KEY must be valid UTF-8"),
        Err(std::env::VarError::NotPresent) => {}
    }
    // Read .env without mutating the process environment after Tokio starts.
    let entries = match dotenvy::dotenv_iter() {
        Ok(entries) => entries,
        Err(dotenvy::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => {
            return Err(SourceError::MissingStooqApiKey.into());
        }
        Err(_) => bail!("could not read .env for STOOQ_API_KEY"),
    };
    for entry in entries {
        // dotenv parse errors can include the original line and its secret.
        let (name, value) = entry.map_err(|_| anyhow::anyhow!("invalid .env syntax"))?;
        if name == "STOOQ_API_KEY" {
            return Ok(value);
        }
    }
    Err(SourceError::MissingStooqApiKey.into())
}

async fn collect_history(code: Option<&str>, output: PathBuf) -> Result<()> {
    let client = StooqClient::new(stooq_api_key()?)?;
    let symbols = StooqSymbolMap::bundled()?;
    let instruments = match code {
        Some(code) => vec![symbols.instrument(code)?],
        None => {
            let gpw = GpwClient::new()?;
            let mut instruments = collect_instruments(&gpw, None, &output).await?;
            instruments.push(Instrument::wig20_index());
            instruments
        }
    };

    let total = instruments.len();
    let mut failures = 0;
    for instrument in instruments {
        let result = async {
            let symbol = symbols.symbol_for(&instrument)?;
            let raw = client.fetch_history(&instrument, symbol).await?;
            archive_history(&raw, &output)
        }
        .await;
        if let Err(error) = result {
            failures += 1;
            eprintln!("{}: {error:#}", instrument.code);
        }
    }
    eprintln!(
        "Collected {}/{total} daily histories; output: {}",
        total - failures,
        output.display()
    );
    ensure!(
        failures == 0,
        "{failures} histories failed; successful observations were preserved"
    );
    Ok(())
}

fn archive_history(raw: &RawDocument, output: &std::path::Path) -> Result<()> {
    let archived = archive_raw(output, raw).context("archiving Stooq CSV response")?;
    let history = match parse_history_document(raw) {
        Ok(history) => history,
        Err(error) => {
            archived.save_error(&error.to_string())?;
            return Err(anyhow::Error::new(error)
                .context(format!("saved response: {}", archived.raw_path().display())));
        }
    };
    let path = archived.save_history(&history)?;
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

async fn collect(code: Option<&str>, output: PathBuf) -> Result<()> {
    let client = GpwClient::new()?;
    let instruments = collect_instruments(&client, code, &output).await?;

    let total = instruments.len();
    let mut failures = 0;
    for instrument in instruments {
        if let Err(error) = collect_one(&client, &instrument, &output).await {
            failures += 1;
            eprintln!("{}: {error:#}", instrument.code);
        }
    }
    eprintln!(
        "Collected {}/{total} companies; output: {}",
        total - failures,
        output.display()
    );
    ensure!(
        failures == 0,
        "{failures} companies failed; successful observations were preserved"
    );
    Ok(())
}

async fn collect_instruments(
    client: &GpwClient,
    code: Option<&str>,
    output: &std::path::Path,
) -> Result<Vec<Instrument>> {
    let raw_portfolio = client
        .fetch_portfolio()
        .await
        .context("fetching WIG20 composition")?;
    let archived = archive_raw(output, &raw_portfolio).context("archiving WIG20 composition")?;
    match portfolio(&raw_portfolio) {
        Ok(instruments) => select(instruments, code),
        Err(error) => {
            archived.save_error(&format!("{error:#}"))?;
            Err(error.context(format!("saved response: {}", archived.raw_path().display())))
        }
    }
}

async fn collect_prices(code: Option<&str>, output: PathBuf) -> Result<()> {
    let pair = code.and_then(MarketPair::from_code);
    let mut collected = 0;
    let mut failures = 0;

    if pair.is_none() {
        let client = GpwClient::new()?;
        let instruments = match collect_instruments(&client, code, &output).await {
            Ok(instruments) => instruments,
            Err(error) if code.is_none() => {
                eprintln!("WIG20 prices: {error:#}");
                failures += 20;
                Vec::new()
            }
            Err(error) => return Err(error),
        };
        for instrument in instruments {
            let result = client
                .fetch_price(&instrument)
                .await
                .map_err(anyhow::Error::from)
                .and_then(|raw| archive_price(&raw, &output));
            match result {
                Ok(()) => collected += 1,
                Err(error) => {
                    failures += 1;
                    eprintln!("{}: {error:#}", instrument.code);
                }
            }
        }
    }

    if code.is_none() || pair.is_some() {
        let client = TradingViewClient::new()?;
        let pairs = pair.map_or_else(|| MarketPair::ALL.to_vec(), |pair| vec![pair]);
        for pair in pairs {
            let result = client
                .fetch_price(pair)
                .await
                .map_err(anyhow::Error::from)
                .and_then(|raw| archive_price(&raw, &output));
            match result {
                Ok(()) => collected += 1,
                Err(error) => {
                    failures += 1;
                    eprintln!("{}: {error:#}", pair.code());
                }
            }
        }
    }

    eprintln!(
        "Collected {collected}/{} prices; output: {}",
        collected + failures,
        output.display()
    );
    ensure!(
        failures == 0,
        "{failures} prices failed; successful observations were preserved"
    );
    Ok(())
}

fn archive_price(raw: &RawDocument, output: &std::path::Path) -> Result<()> {
    let archived = archive_raw(output, raw).context("archiving price response")?;
    let snapshot = match parse_price_document(raw) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            archived.save_error(&error.to_string())?;
            return Err(anyhow::Error::new(error)
                .context(format!("saved response: {}", archived.raw_path().display())));
        }
    };
    let path = archived.save_price(&snapshot)?;
    eprintln!(
        "{}: {} {} -> {}",
        snapshot.instrument.code,
        snapshot.price,
        snapshot.instrument.currency,
        path.display()
    );
    Ok(())
}

async fn collect_one(
    client: &GpwClient,
    instrument: &Instrument,
    output: &std::path::Path,
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

fn check_http(raw: &RawDocument) -> Result<()> {
    ensure!(
        (200..300).contains(&raw.http_status),
        "HTTP {} from {}",
        raw.http_status,
        raw.url
    );
    Ok(())
}

fn portfolio(raw: &RawDocument) -> Result<Vec<Instrument>> {
    check_http(raw)?;
    Ok(parse_portfolio(&raw.body)?)
}

fn select(instruments: Vec<Instrument>, code: Option<&str>) -> Result<Vec<Instrument>> {
    let Some(code) = code else {
        return Ok(instruments);
    };
    if let Some(instrument) = instruments
        .iter()
        .find(|item| item.code.eq_ignore_ascii_case(code))
    {
        return Ok(vec![instrument.clone()]);
    }
    let available = instruments
        .iter()
        .map(|item| item.code.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    bail!("unknown GPW code {code:?}; current WIG20: {available}")
}

fn print_json(value: &impl Serialize) -> Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    writeln!(stdout)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_archives_raw_http_and_parse_failures_without_a_partial_history() {
        for (status, body) in [
            (403, "access denied"),
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
            assert!(archive_history(&raw, directory.path()).is_err());
            let entries: Vec<_> = std::fs::read_dir(directory.path()).unwrap().collect();
            assert_eq!(entries.len(), 1);
            let archived = entries[0].as_ref().unwrap().path();
            assert_eq!(read_raw(&archived.join("raw.json")).unwrap(), raw);
            assert!(archived.join("error.json").exists());
            assert!(!archived.join("history.json").exists());
        }
    }
}
