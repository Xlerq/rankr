# AI_CONTEXT.md

Strict working context for AI agents in this repository.

## Project

`rankr` is an engineering thesis project.

Goal: build a web application written entirely in Rust (frontend, backend, and scoring engine) for ranking GPW companies by relative growth potential over 3–12 months.

A higher final score means a greater chance of achieving a better return than other companies in the same basket. The score measures relative growth potential, not standalone company quality or a predicted percentage return.

The MVP basket is the full WIG20, using end-of-day prices and financial data. Other GPW stocks are a later extension. Treat WIG20 as the starting basket, not a permanent limit of 20 companies; do not design that extension in this task.

This is not:

- a trading bot,
- a brokerage integration,
- a real-time market terminal,
- financial advice,
- a machine learning project for MVP.

## Thesis

The thesis is written in LaTeX.

- Main source: `thesis/main.tex`
- Final PDF target: `thesis/main.pdf`

Do not recreate scattered thesis documentation in `docs/`. Put thesis content into `thesis/main.tex` unless the user asks for a split LaTeX structure later.

## MVP

Build around this flow:

1. Import reference data, full available Yahoo daily company OHLCV history, and financial data for the full WIG20.
2. Store instruments, prices, financial data, score configs, score results, and source logs in SurrealDB.
3. Calculate deterministic growth-potential scoring as the sum (component score × weight).
4. Store score results over time.
5. Expose ranking through the backend API.
6. Show a company table as the main view in Leptos, sorted by the final score, with component scores, their weights, and company details.
7. Render charts with Plotters.
8. Compare score history with price history for validation notes.

## Stack

- Frontend: Rust, Leptos, WebAssembly
- Backend: Rust, Axum, Tokio
- Scoring engine: Rust
- Database: SurrealDB
- Charts: Plotters
- Analytics: R / Rscript
- Data exchange: JSON / CSV

Do not replace the stack unless the user explicitly asks.

## Repository Structure

- `thesis/` - LaTeX thesis source and thesis assets.
- `frontend/` - future Leptos frontend.
- `backend/` - future Axum backend.
- `analytics/` - future R scripts and sample IO.
- `database/` - future SurrealDB schema and seed files.
- `shared/` - future shared Rust DTOs.
- `scripts/` - future helper scripts.
- `tests/` - future tests and fixtures.
- `data/` - raw and processed market data placeholders.

Keep the repository minimal. Do not add extra documentation files unless they clearly reduce friction.

## Scoring

The scoring must be deterministic and explainable.

The v1 score families are exactly:

- `fundamental`: the most important family, covering financial condition, valuation, and revenue/profit dynamics,
- `technical`: price trend/momentum and monthly seasonality from multi-year daily history,
- `sentiment`: sentiment, with a schema field now and no data source specified.

Each company has component scores (metrics/signals), each with an assigned, fixed weight. The final score is the sum (component score × weight), measures relative growth potential, and determines table sorting. Do not normalize the final score or map the sum to an expected percentage return. Explain the result through its component scores and their weights.

v1 uses one set of weights for the entire WIG20, including banks. Some bank fields may be empty. Do not introduce a separate banking model in v1.

Seasonality describes price behaviour in analogous months over roughly 12 or more years and belongs to `technical`, without a fourth family weight. Import the daily history now; the seasonality formula and scoring engine remain for a later implementation stage. That stage must mark insufficient series length instead of assuming every company has a long history. Sector macro and COT are outside v1 and may only be optional extensions after MVP.

Database contract: `score_config` and `score_result` in `database/schema.surql` define the product contract for the three families above. Each weight is a nonnegative number with no upper bound or required total. Component scores and `final_score` are unrestricted numbers, including negative values; `final_score` is the sum of the three component scores multiplied by their corresponding weights. Only `data_quality_score` retains its 0–100 range. `database/seed.surql` provides the illustrative `default_growth_v1` configuration with the largest weight on `fundamental` and a record-shape placeholder, not a calibrated ranking. This contract does not implement scoring or define component formulas.

## Out of Scope for MVP

- real-time data,
- intraday data,
- order execution,
- brokerage APIs,
- crypto/forex/options support,
- complex authentication,
- payment systems,
- social features,
- microservices,
- Kubernetes,
- machine learning.

## Current implementation and scope

The Rust workspace currently contains the `rankr-import` package in `importer/`.
It collects basic GPW/Notoria fundamentals, Yahoo daily company prices/history and TradingView FX/gold quotes,
and obtains the current WIG20 list from GPW Benchmark. CLI: `collect [CODE]`,
`prices [CODE]`, `history [CODE]`, `fetch CODE`, `parse FILE`.

- One package with a reusable library and a thin CLI; keep HTTP, pure parsing,
  serializable model types and JSON persistence in separate modules.
- Persist raw JSON before parsing during collection; retain all financial fields.
- Monetary values are Decimal in Rust and decimal strings in JSON, preserving
  currency, unit scale and sign. Missing values are null, not zero.
- Do not introduce parser-version metadata. It was explicitly excluded by the user.
- The importer has no database connection. The scoring tables define the product
  contract; other collections retain their earlier design and are not the active
  importer contract.
- The next storage integration is SurrealDB in the same application/data pipeline.
- Price source contract: company daily prices and full daily history come from
  Yahoo Finance without API keys. `prices CODE` selects the latest complete,
  finished daily candle; `history CODE` requests all available daily history.
  Both save `history.json` with source `yahoo_finance`. FX/gold quotes come from
  TradingView and remain `price.json` snapshots.
- Without CODE, collect the live WIG20 companies. Do not add the index itself:
  Yahoo has insufficient WIG20 index history. Map GPW code and ISIN to the bundled
  `yahoo_symbol` in `data/raw/wig20_symbols.csv`; never guess missing identifiers.
- Preserve raw responses before parsing and keep full histories in ignored
  `data/collected/`. Use explicit daily request bounds, not `range=max`.
  Preserve Decimal OHLCV and a separate provider `adjusted_close`; do not
  recompute dividends/splits or implement seasonality during data-source work.
- Use Warsaw session dates. Exclude today's candle before the regular session end
  plus 15 minutes. Record missing/invalid candles in `skipped_candles`, without
  filling gaps or substituting live quotes. Reject structural/identity errors.
- Retain offline parsing of legacy Stooq and GPW archives and their source labels.
  No Stooq or GPW company-price downloads remain in the active importer.
- Price archives are required for technical signals. Fundamentals remain current
  GPW/Notoria observations collected over time; do not invent report history.
  Detailed bank fundamentals come later.
- Current source access/publication frequency is accepted; do not reopen this work.
- Analytics and new import code are Rust, as specified in ROADMAP.md.

Run `cargo fmt --all -- --check`, `cargo test --workspace --locked` and
`cargo clippy --workspace --all-targets --locked -- -D warnings` for Rust changes.

## Style

Keep the implementation clear, direct, and thesis-friendly. Prefer the simplest working version over broad abstractions.
