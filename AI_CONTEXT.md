# AI_CONTEXT.md

This file is a strict working guide for AI coding agents operating in this repository.

Read this before generating code, modifying architecture, adding dependencies or changing project scope.

---

## Project Summary

This is an engineering thesis project.

The product is a Rust-first web application for scoring and ranking Polish stock market instruments, especially WIG20 companies and selected GPW indices.

The system should:

- import end-of-day OHLCV data,
- store historical prices,
- calculate technical and risk-based scores,
- rank instruments,
- show score breakdowns,
- visualize price and score history,
- run simple ranking-based backtests.

This is not a trading bot.
This is not a financial advice system.
This is not a clone of A1 EdgeFinder.

---

## Locked Technology Stack

Do not change these technologies unless the user explicitly asks.

### Frontend

- Rust
- Leptos
- WebAssembly
- Plotters

Do not replace with:

- React
- Vue
- Svelte
- Angular
- JavaScript charting libraries

Plotters is intentionally chosen even if it is harder.

### Backend

- Rust
- Axum
- Tokio
- Serde
- Reqwest
- Tracing

### Database

- SurrealDB

Do not replace with:

- PostgreSQL
- SQLite
- MongoDB
- ClickHouse
- TimescaleDB

unless explicitly requested.

### Analytics

- R
- Rscript

Do not replace with:

- Python
- pandas
- scikit-learn

unless explicitly requested.

### Data Exchange

- JSON
- CSV

Preferred calculation pipeline:

```text
Rust backend -> input JSON/CSV -> Rscript -> output JSON -> Rust backend -> SurrealDB
```

Do not introduce complex Rust-R FFI unless explicitly requested.

---

## Main Product Scope

Build the application around this MVP:

1. Import EOD data for WIG20 instruments.
2. Store daily OHLCV data in SurrealDB.
3. Calculate score components in R.
4. Store score results.
5. Show ranking table in Leptos.
6. Show instrument details page.
7. Render charts using Plotters.
8. Run simple backtest.
9. Compare strategy result against WIG20.
10. Allow user to configure score weights.

Anything else is secondary.

---

## What Not To Do

Do not add:

- machine learning before basic scoring works,
- real-time data,
- brokerage integration,
- order execution,
- crypto support,
- forex support,
- options support,
- complex authentication,
- payment system,
- social features,
- notifications,
- AI-generated investment advice,
- portfolio optimization,
- advanced risk engine,
- microservice architecture,
- Kafka,
- Kubernetes,
- unnecessary Docker complexity.

Do not overengineer this.

The goal is to finish a strong engineering thesis project, not to create a production Bloomberg Terminal.

---

## Architecture

Target architecture:

```text
[Leptos Frontend / WASM]
          |
          | HTTP / JSON
          v
[Rust Backend / Axum]
          |
          | queries
          v
[SurrealDB]

[Rust Backend]
          |
          | Rscript
          v
[R Analytics Scripts]
```

Keep the architecture simple and understandable.

---

## Repository Structure

Preferred structure:

```text
frontend/
backend/
analytics/
database/
shared/
scripts/
docs/
```

### `frontend/`

Contains the Leptos frontend.

Responsibilities:

- pages,
- reusable components,
- Plotters chart components,
- API client,
- styling.

### `backend/`

Contains the Axum backend.

Responsibilities:

- routes,
- data download,
- database access,
- calling R scripts,
- serialization,
- error handling.

### `analytics/`

Contains R scripts.

Responsibilities:

- indicators,
- scoring,
- backtesting,
- JSON/CSV input-output.

### `database/`

Contains SurrealDB schema and seed files.

### `shared/`

Contains Rust DTOs shared between backend and frontend.

### `docs/`

Contains thesis notes, architecture notes and methodology.

---

## Scoring Methodology

Keep scoring explainable.

The final score should be a weighted sum of components:

```text
final_score =
  trend_weight * trend_score +
  momentum_weight * momentum_score +
  risk_weight * risk_score +
  volume_weight * volume_score +
  relative_strength_weight * relative_strength_score
```

Do not create a black-box scoring system.

Every final score must be explainable through its components.

Expected components:

### Trend Score

Based on:

- SMA50,
- SMA200,
- price above/below moving averages,
- moving average slope.

### Momentum Score

Based on:

- 1M return,
- 3M return,
- 6M return,
- possibly 12M return.

### Risk Score

Based on:

- volatility,
- max drawdown,
- downside risk if easy to implement.

Higher risk should usually reduce score.

### Volume Score

Based on:

- average volume,
- volume trend,
- liquidity proxy.

### Relative Strength Score

Based on:

- performance versus WIG20,
- performance versus WIG,
- rank versus other instruments.

---

## Score Labels

Use this default mapping:

```text
0-20    strong bearish
21-40   bearish
41-60   neutral
61-80   bullish
81-100  strong bullish
```

These are model labels, not financial recommendations.

Always avoid language like:

- "buy this stock",
- "sell this stock",
- "guaranteed return",
- "safe investment".

Prefer:

- "high model score",
- "bullish model classification",
- "ranked highly by the scoring model",
- "performed better in historical backtest".

---

## Backtesting Methodology

Keep the first backtest simple.

Suggested strategy:

```text
Every month:
  1. calculate scores for all instruments,
  2. rank instruments,
  3. select top N,
  4. hold for one month,
  5. rebalance monthly,
  6. compare against WIG20.
```

Metrics:

- total return,
- benchmark return,
- max drawdown,
- volatility,
- Sharpe-like ratio,
- win rate,
- number of rebalances.

Do not add advanced portfolio optimization before this basic backtest works.

---

## Data Policy

Use end-of-day data.

Preferred instruments:

```text
WIG20 companies
WIG
WIG20
mWIG40
sWIG80
```

Possible sources:

- Stooq
- GPW historical data

Real-time data is out of scope.

Intraday data is out of scope for MVP.

When implementing data import:

- validate dates,
- validate numeric values,
- handle missing rows,
- handle duplicate candles,
- log data source and import time,
- store source name.

---

## Rust Backend Rules

Use simple Rust.

Prefer:

- explicit structs,
- clear modules,
- readable service functions,
- `Result<T, AppError>`,
- `serde` DTOs,
- `tracing` logs,
- `reqwest` for HTTP,
- `tokio::process::Command` for running Rscript.

Avoid:

- `.unwrap()` in application logic,
- global mutable state,
- premature generics,
- deep trait hierarchies,
- magical macros unless framework-required,
- hiding business logic inside route handlers.

Backend routes should be thin.

Business logic should live in services.

---

## Frontend Rules

Use Leptos.

Keep components small and practical.

Suggested frontend sections:

```text
pages/
  dashboard.rs
  instrument.rs
  backtest.rs
  scoring_settings.rs

components/
  ranking_table.rs
  score_badge.rs
  score_breakdown.rs
  metric_card.rs

charts/
  price_chart.rs
  score_history_chart.rs
  equity_curve_chart.rs
  drawdown_chart.rs
```

Plotters chart code should stay in dedicated chart components.

Do not mix large chart rendering code into page files.

Start with simple charts:

1. line chart of close prices,
2. score history line chart,
3. equity curve,
4. drawdown.

Do not start with candlestick charts.

---

## R Script Rules

R scripts should be simple and boring.

Expected files:

```text
analytics/
  io.R
  indicators.R
  scoring.R
  backtest.R
```

R scripts should:

- read JSON or CSV input,
- calculate results,
- write JSON output,
- avoid interactive plots,
- avoid hidden global state,
- avoid extremely clever one-liners,
- fail clearly when input data is invalid.

The R layer should not talk directly to the frontend.

The R layer should not own application state.

---

## SurrealDB Rules

Use SurrealDB as the source of persisted application data.

Suggested tables:

```text
instrument
price_daily
score_result
score_config
backtest_result
data_source_log
```

Store score history.

Do not only store the latest score.

The application must be able to show how an instrument score changed over time.

---

## API Design Rules

Use JSON API.

Keep endpoints predictable.

Suggested endpoints:

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
GET  /api/backtests/{id}
GET  /api/configs/scoring
POST /api/configs/scoring
```

Use stable DTOs from the `shared` crate when possible.

---

## Testing Priorities

Prioritize tests for:

1. scoring normalization,
2. data import parsing,
3. missing data handling,
4. score label mapping,
5. backtest calculations,
6. API response shape.

Do not try to test every Leptos component at the beginning.

---

## First Milestone

The first useful milestone is not a beautiful UI.

The first useful milestone is:

```text
One instrument end-to-end:
  1. load historical data,
  2. store it in SurrealDB,
  3. calculate a score in R,
  4. save result,
  5. expose it via API,
  6. display it in Leptos,
  7. draw one Plotters chart.
```

Recommended first instruments:

```text
KGH
PKN
CDR
WIG20
```

---

## Prioritized Build Order

Use this order:

```text
1. Backend healthcheck
2. SurrealDB connection
3. Instrument model
4. Price data model
5. Manual CSV import
6. Data download from Stooq/GPW
7. Basic R scoring script
8. Save score results
9. API ranking endpoint
10. Leptos dashboard shell
11. Ranking table
12. Plotters price chart
13. Instrument details page
14. Score breakdown
15. Score history chart
16. Backtest script
17. Backtest endpoint
18. Backtest visualization
19. Scoring settings
20. Polish UI text and thesis polish
```

Do not skip directly to UI polish.

---

## Style Preference

The code should be:

- clear,
- direct,
- readable,
- thesis-friendly,
- not too clever,
- not enterprise-overengineered.

The user prefers useful, concrete solutions over abstract architecture discussion.

When in doubt, implement the simplest working version first.

---

## Naming Notes

The final application name is not locked.

Preferred naming direction:

- trading-related,
- short,
- sharp,
- contains a hard letter `R`,
- sounds good as a Rust project,
- similar vibe to names like `scnar`, `ratatui`, `tradr`, `raker`, `radar`.

Do not rename the repository everywhere without explicit approval.

Use temporary placeholder names until the final name is chosen.
