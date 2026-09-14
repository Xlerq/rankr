use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use rankr_import::{
    model::{FundamentalSnapshot, Instrument, RawDocument, Source},
    parser::parse_fundamentals,
    source::{GpwClient, parse_portfolio},
    storage::{archive_raw, read_raw},
};
use serde::Serialize;

#[derive(Parser)]
#[command(name = "rankr-import", version, about = "Collect basic GPW fundamentals as JSON")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Archive raw and parsed data; omit CODE to collect the current WIG20.
    Collect {
        /// GPW company code, e.g. KGHM, PKOBP, PZU (case-insensitive).
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
    /// Parse a raw JSON file offline and write fundamentals as JSON to stdout.
    Parse {
        /// raw.json produced by collect or fetch.
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
            print_json(&snapshot(&raw)?)
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
    }
}

async fn collect(code: Option<&str>, output: PathBuf) -> Result<()> {
    let client = GpwClient::new()?;
    let raw_portfolio = client.fetch_portfolio().await.context("fetching WIG20 composition")?;
    let archived = archive_raw(&output, &raw_portfolio).context("archiving WIG20 composition")?;
    let instruments = match portfolio(&raw_portfolio) {
        Ok(instruments) => select(instruments, code)?,
        Err(error) => {
            archived.save_error(&format!("{error:#}"))?;
            return Err(error.context(format!("saved response: {}", archived.raw_path().display())));
        }
    };

    let total = instruments.len();
    let mut failures = 0;
    for instrument in instruments {
        if let Err(error) = collect_one(&client, &instrument, &output).await {
            failures += 1;
            eprintln!("{}: {error:#}", instrument.code);
        }
    }
    eprintln!("Collected {}/{total} companies; output: {}", total - failures, output.display());
    ensure!(failures == 0, "{failures} companies failed; successful observations were preserved");
    Ok(())
}

async fn collect_one(client: &GpwClient, instrument: &Instrument, output: &std::path::Path) -> Result<()> {
    let raw = client.fetch_fundamentals(instrument).await?;
    let archived = archive_raw(output, &raw).context("archiving raw response")?;
    let parsed = match snapshot(&raw) {
        Ok(parsed) => parsed,
        Err(error) => {
            archived.save_error(&format!("{error:#}"))?;
            return Err(error.context(format!("saved response: {}", archived.raw_path().display())));
        }
    };
    let path = archived.save_snapshot(&parsed)?;
    eprintln!("{}: {} -> {}", instrument.code, parsed.fundamentals.report_period, path.display());
    Ok(())
}

fn check_http(raw: &RawDocument) -> Result<()> {
    ensure!((200..300).contains(&raw.http_status), "HTTP {} from {}", raw.http_status, raw.url);
    Ok(())
}

fn portfolio(raw: &RawDocument) -> Result<Vec<Instrument>> {
    check_http(raw)?;
    Ok(parse_portfolio(&raw.body)?)
}

fn select(instruments: Vec<Instrument>, code: Option<&str>) -> Result<Vec<Instrument>> {
    let Some(code) = code else { return Ok(instruments) };
    if let Some(instrument) = instruments.iter().find(|item| item.code.eq_ignore_ascii_case(code)) {
        return Ok(vec![instrument.clone()]);
    }
    let available = instruments.iter().map(|item| item.code.as_str()).collect::<Vec<_>>().join(", ");
    bail!("unknown GPW code {code:?}; current WIG20: {available}")
}

fn snapshot(raw: &RawDocument) -> Result<FundamentalSnapshot> {
    check_http(raw)?;
    ensure!(matches!(raw.source, Source::GpwNotoria), "expected a GPW/Notoria fundamentals response");
    let instrument = raw.instrument.clone().context("raw JSON is missing the instrument")?;
    Ok(FundamentalSnapshot {
        instrument,
        source: raw.source.clone(),
        source_url: raw.url.clone(),
        fetched_at: raw.fetched_at,
        fundamentals: parse_fundamentals(&raw.body)?,
    })
}

fn print_json(value: &impl Serialize) -> Result<()> {
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, value)?;
    writeln!(stdout)?;
    Ok(())
}
