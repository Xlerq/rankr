# GPW Scorefinder

Rust-first web application for scoring and ranking Polish stock market instruments based on historical market data, technical indicators, risk metrics and simple backtesting.

This project is inspired by financial scoring dashboards such as A1 EdgeFinder, but it is not intended to be a clone. The goal is to build an original engineering thesis project focused on Polish stock market instruments, especially WIG20 companies and selected GPW indices.

> Working name: **GPW Scorefinder**
>
> The final product name is not locked yet.

---

## Project Purpose

This repository contains an engineering thesis project.

The final product should be a web application that:

- downloads end-of-day market data for Polish stocks and indices,
- stores historical OHLCV data,
- calculates technical and risk-based scores,
- ranks instruments by final score,
- shows score breakdowns,
- visualizes price and score history,
- allows basic scoring configuration,
- performs simple historical backtests of ranking-based strategies.

The application is not a real trading system and does not provide financial advice.

---

## Main Idea

Each instrument receives a score from `0` to `100`.

The score should be calculated from several components:

```text
final_score =
  trend_weight * trend_score +
  momentum_weight * momentum_score +
  risk_weight * risk_score +
  volume_weight * volume_score +
  relative_strength_weight * relative_strength_score
```

Example components:

- `trend_score` — based on SMA50/SMA200 and trend direction,
- `momentum_score` — based on 1M/3M/6M returns,
- `risk_score` — based on volatility and max drawdown,
- `volume_score` — based on liquidity and volume behavior,
- `relative_strength_score` — based on performance versus WIG20 or WIG.

The application should show not only the final score, but also its breakdown.

Bad:

```text
KGHM score: 78
```

Good:

```text
KGHM final score: 78

trend_score: 82
momentum_score: 75
risk_score: 61
volume_score: 80
relative_strength_score: 88
```

---

## Technology Stack

### Frontend

- Rust
- Leptos
- WebAssembly
- Plotters
- CSS or Tailwind CSS

The frontend should be written in Rust. Do not replace it with React, Vue, Svelte or Angular unless explicitly requested.

Frontend responsibilities:

- dashboard view,
- ranking table,
- instrument details view,
- scoring breakdown,
- chart rendering,
- backtest result visualization,
- scoring configuration UI.

---

### Charts

- Plotters

Plotters should be used for visualizations.

Expected charts:

- price history line chart,
- SMA50/SMA200 chart,
- score history chart,
- equity curve chart,
- drawdown chart,
- benchmark comparison chart.

Do not start with candlestick charts unless the basic charts are already finished.

Priority:

1. price line chart,
2. score history chart,
3. equity curve,
4. drawdown,
5. candlestick chart as optional extension.

---

### Backend

- Rust
- Axum
- Tokio
- Serde
- Reqwest
- Tracing

Backend responsibilities:

- expose REST API,
- fetch market data,
- validate and normalize data,
- communicate with SurrealDB,
- call R scripts for scoring and backtesting,
- return JSON responses to the frontend.

Example API endpoints:

```text
GET  /api/health
GET  /api/instruments
GET  /api/instruments/{symbol}
GET  /api/prices/{symbol}
GET  /api/scores/{symbol}
GET  /api/ranking
POST /api/data/update
POST /api/score/recalculate
POST /api/backtest
```

---

### Database

- SurrealDB

SurrealDB stores:

- instruments,
- daily OHLCV data,
- score results,
- scoring configurations,
- backtest results,
- data source logs.

Expected tables/collections:

```text
instrument
price_daily
score_result
score_config
backtest_result
data_source_log
```

Example data model:

```text
instrument:
  symbol
  name
  market
  sector
  index_membership
  active

price_daily:
  instrument_id
  date
  open
  high
  low
  close
  volume
  source

score_result:
  instrument_id
  date
  trend_score
  momentum_score
  risk_score
  volume_score
  relative_strength_score
  final_score
  label

score_config:
  name
  trend_weight
  momentum_weight
  risk_weight
  volume_weight
  relative_strength_weight

backtest_result:
  config_id
  start_date
  end_date
  strategy_return
  benchmark_return
  max_drawdown
  volatility
  sharpe_proxy
```

---

### Analytics Layer

- R
- Rscript
- JSON / CSV exchange format

R is responsible for calculations, not for the web application itself.

R scripts should calculate:

- technical indicators,
- score components,
- final score,
- backtest results,
- basic statistics.

Expected R files:

```text
analytics/
  indicators.R
  scoring.R
  backtest.R
  io.R
```

Rust should call R scripts using `Rscript`.

Example flow:

```text
Rust backend
  -> exports input data to JSON or CSV
  -> runs Rscript analytics/scoring.R
  -> reads output JSON
  -> stores result in SurrealDB
```

Avoid complex Rust-R FFI unless explicitly requested.

Preferred approach:

```text
Rust -> Rscript -> JSON output -> Rust -> SurrealDB
```

---

## Market Data

The project should use end-of-day data.

Primary target:

```text
WIG20 companies + selected GPW indices
```

Recommended scope for MVP:

```text
Stocks:
- WIG20 companies

Indices:
- WIG
- WIG20
- mWIG40
- sWIG80
```

Possible data sources:

- Stooq
- GPW historical data

Do not build real-time market data integration for MVP.

Real-time or intraday market data is out of scope unless explicitly requested.

---

## MVP Scope

The MVP should include:

1. Data import for WIG20 instruments.
2. Storage of daily OHLCV data.
3. Score calculation.
4. Ranking table.
5. Instrument details page.
6. Price chart.
7. Score history chart.
8. Basic backtest.
9. Benchmark comparison against WIG20.
10. Configurable scoring weights.

Anything beyond that is optional.

---

## Non-Goals

This project should not become:

- a real-time trading terminal,
- a brokerage application,
- a trading bot,
- a clone of A1 EdgeFinder,
- a financial advice platform,
- a crypto dashboard,
- a machine learning project without a working MVP,
- an overengineered distributed system.

Avoid scope creep.

---

## Suggested Repository Structure

```text
gpw-scorefinder/
  README.md
  AI_CONTEXT.md

  frontend/
    Cargo.toml
    src/
      main.rs
      app.rs
      pages/
      components/
      charts/
      api/
      styles/

  backend/
    Cargo.toml
    src/
      main.rs
      config.rs
      routes/
      services/
      models/
      db/
      data_sources/
      analytics/
      errors.rs

  analytics/
    indicators.R
    scoring.R
    backtest.R
    io.R

  database/
    schema.surql
    seed.surql

  shared/
    Cargo.toml
    src/
      dto.rs
      types.rs

  scripts/
    update_data.sh
    run_backend.sh
    run_frontend.sh

  docs/
    thesis_notes.md
    architecture.md
    scoring_methodology.md
    data_sources.md
    api.md
```

---

## Shared Types

Prefer keeping shared DTOs in a separate Rust crate:

```text
shared/
```

This crate should contain types used by both backend and frontend.

Example:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstrumentDto {
    pub symbol: String,
    pub name: String,
    pub market: String,
    pub sector: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDto {
    pub symbol: String,
    pub date: String,
    pub trend_score: f64,
    pub momentum_score: f64,
    pub risk_score: f64,
    pub volume_score: f64,
    pub relative_strength_score: f64,
    pub final_score: f64,
    pub label: String,
}
```

This reduces duplication between backend and frontend.

---

## Scoring Labels

Suggested score labels:

```text
0-20    strong bearish
21-40   bearish
41-60   neutral
61-80   bullish
81-100  strong bullish
```

These labels are only model outputs, not investment recommendations.

---

## Backtesting Idea

The backtest should answer a simple question:

```text
Did instruments with high scores perform better than the benchmark?
```

Suggested strategy:

```text
Every month:
  1. calculate scores for all instruments,
  2. select top N instruments,
  3. hold them for one month,
  4. rebalance monthly,
  5. compare result with WIG20.
```

Basic backtest metrics:

- total return,
- benchmark return,
- max drawdown,
- volatility,
- win rate,
- Sharpe-like ratio,
- number of trades/rebalances.

Keep it simple.

Do not implement advanced portfolio optimization in MVP.

---

## Development Rules for AI Assistants

When working on this repository, follow these rules:

1. Keep the project Rust-first.
2. Do not replace Leptos with React.
3. Do not replace Plotters with JavaScript charting libraries unless explicitly requested.
4. Do not replace SurrealDB with PostgreSQL unless explicitly requested.
5. Do not move scoring logic from R to Python.
6. Do not add machine learning before the basic scoring system works.
7. Do not add real-time market data before EOD data works.
8. Do not overcomplicate architecture.
9. Prefer simple, working code over abstract patterns.
10. Every module should be understandable for an engineering thesis.

---

## Coding Style

General rules:

- write simple code,
- prefer explicit names,
- avoid unnecessary abstractions,
- keep functions small,
- validate external data,
- log important operations,
- return clear errors,
- keep API responses stable.

Rust rules:

- use `Result<T, E>` properly,
- avoid `.unwrap()` in production code,
- use `serde` for JSON serialization,
- use `tracing` for logs,
- keep DTOs separate from database models,
- avoid premature generic abstractions.

R rules:

- keep scripts readable,
- use simple data frames,
- return machine-readable JSON,
- do not create interactive R plots,
- do not hide calculation logic in unclear one-liners.

Frontend rules:

- keep components small,
- separate pages from reusable components,
- keep chart code inside dedicated chart components,
- avoid heavy animations,
- prioritize readable dashboard layout.

---

## Expected User Interface

Main views:

```text
/dashboard
  Ranking table
  Top bullish instruments
  Top bearish instruments
  Market summary

/instrument/{symbol}
  Price chart
  Score history
  Score breakdown
  Basic stats

/backtest
  Strategy result
  Benchmark comparison
  Equity curve
  Drawdown chart

/settings/scoring
  Weight configuration
```

---

## Example Ranking Table

```text
Symbol | Name       | Final Score | Trend | Momentum | Risk | Label
KGH    | KGHM       | 82.4        | 88    | 79       | 62   | bullish
PKN    | Orlen      | 74.1        | 70    | 76       | 68   | bullish
CDR    | CD Projekt | 51.6        | 48    | 55       | 43   | neutral
```

---

## Data Update Flow

Expected data update process:

```text
1. Backend receives update request.
2. Backend downloads EOD data from configured source.
3. Backend validates CSV response.
4. Backend normalizes date and numeric fields.
5. Backend stores candles in SurrealDB.
6. Backend logs update result.
```

---

## Score Calculation Flow

Expected scoring process:

```text
1. Backend loads historical prices from SurrealDB.
2. Backend writes input file for R.
3. Backend runs scoring.R.
4. R calculates indicators and score components.
5. R writes output JSON.
6. Backend reads output JSON.
7. Backend saves score_result records in SurrealDB.
8. Frontend displays updated ranking.
```

---

## Backtest Flow

Expected backtest process:

```text
1. User selects date range and scoring config.
2. Backend loads historical prices and historical scores.
3. Backend calls backtest.R.
4. R simulates ranking-based strategy.
5. R returns backtest metrics and equity curve.
6. Backend saves result.
7. Frontend displays backtest report.
```

---

## Thesis-Friendly Explanation

This project can be described as:

```text
A Rust/WebAssembly web application for multi-criteria scoring of GPW-listed financial instruments.
The system combines a Rust backend, a Rust frontend compiled to WebAssembly, a SurrealDB database,
and an R-based analytical module responsible for indicator calculation and backtesting.
```

The engineering value comes from:

- integrating multiple technologies,
- building a full-stack application,
- processing real financial time series,
- implementing a scoring model,
- visualizing results,
- validating scores through backtesting.

---

## Current Priority

Build in this order:

```text
1. Backend healthcheck
2. SurrealDB connection
3. Instrument model
4. Price data model
5. Manual CSV import
6. Data download from Stooq/GPW
7. Basic R scoring script
8. Save score results
9. Leptos dashboard
10. Plotters price chart
11. Ranking table
12. Instrument details page
13. Backtest script
14. Backtest visualization
15. Scoring settings
```

Do not start with the prettiest UI.

First make the data pipeline work.

---

## Recommended First Milestone

The first working version should:

```text
- load historical data for one instrument,
- store it in SurrealDB,
- calculate a simple score in R,
- expose the result through Axum API,
- display it in Leptos,
- render one Plotters chart.
```

Example target instrument:

```text
PKN / ORLEN
KGH / KGHM
CDR / CD Projekt
WIG20
```

---

## License

To be decided.

---

## Disclaimer

This project is created for educational and engineering thesis purposes.

It does not provide financial advice, investment recommendations or trading signals.
All calculated scores are experimental model outputs.
