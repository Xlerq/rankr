# AI_CONTEXT.md

Strict working context for AI agents in this repository.

## Project

`rankr` is an engineering thesis project.

Goal: build a Rust-first web application for scoring and ranking Polish stock market instruments from GPW, starting with WIG20 end-of-day data.

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

1. Import GPW/WIG20 EOD OHLCV data.
2. Store daily prices in SurrealDB.
3. Calculate deterministic scoring.
4. Store score results.
5. Expose ranking through the backend API.
6. Show ranking and instrument details in Leptos.
7. Render charts with Plotters.
8. Run a simple historical backtest.

## Stack

- Frontend: Rust, Leptos, WebAssembly
- Backend: Rust, Axum, Tokio
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

Expected components:

- trend,
- momentum,
- risk,
- volume,
- relative strength.

The final score should be explainable through component scores and normalized to a `0-100` scale.

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

## Build Order

Preferred order:

1. Backend healthcheck.
2. SurrealDB connection.
3. Instrument and price models.
4. Manual CSV import.
5. Stooq/GPW data download.
6. Basic scoring.
7. Ranking API.
8. Leptos dashboard.
9. Plotters chart.
10. Instrument detail view.
11. Basic backtest.
12. Thesis write-up in `thesis/main.tex`.

## Style

Keep the implementation clear, direct, and thesis-friendly. Prefer the simplest working version over broad abstractions.
