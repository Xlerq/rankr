use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use rankr_import::{
    model::{Instrument, MarketPair, RawDocument, Source},
    parser::parse_document,
    prices::parse_price_document,
    source::{GpwClient, TradingViewClient, parse_portfolio},
    storage::{archive_raw, read_raw},
};
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "rankr-import",
    version,
    about = "Collect GPW fundamentals and market prices as JSON"
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
    /// Write one company's raw response as JSON to stdout.
    Fetch {
        /// GPW company code from the current WIG20.
        code: String,
    },
    /// Parse a raw JSON file offline and write fundamentals or a price as JSON.
    Parse {
        /// raw.json produced by collect, prices or fetch.
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
                Source::GpwPrices | Source::TradingViewIdc => print_json(&parse_price_document(&raw)?),
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
    }
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
            let result = client.fetch_price(&instrument).await
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
            let result = client.fetch_price(pair).await
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
    eprintln!("{}: {} {} -> {}", snapshot.instrument.code, snapshot.price,
        snapshot.instrument.currency, path.display());
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
