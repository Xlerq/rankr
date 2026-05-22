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

1. Import WIG20 reference data, OHLCV prices, fundamentals, and macro samples.
2. Store instruments, prices, fundamentals, macro observations, score configs, score results, and source logs in SurrealDB.
3. Calculate deterministic fundamental-first scoring.
4. Store score results over time.
5. Expose ranking through the backend API.
6. Show ranking and instrument details in Leptos.
7. Render charts with Plotters.
8. Compare score history with price history for validation notes.

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

The current direction is `fundamental-first scoring`. Fundamentals dominate the final score, while price action is only a supporting signal.

Expected components:

- `profitability_score`,
- `financial_strength_score`,
- `cashflow_quality_score`,
- `efficiency_score`,
- `trend_score` as a light technical filter,
- `macro_context_score` as optional context, disabled by default in MVP.

The final score should be explainable through component scores and normalized to a `0-100` scale.
Trend has a low weight. Profitability, financial strength, cashflow quality, and efficiency are the dominant parts of the MVP scoring model.

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

1. SurrealDB schema and seed.
2. Data sample validation.
3. Backend healthcheck.
4. SurrealDB connection.
5. Importers for Stooq, GPW Benchmark, GPW / Notoria, and NBP.
6. Fundamental-first scoring.
7. Ranking API.
8. Leptos dashboard.
9. Plotters chart.
10. Instrument detail view.
11. Score history versus price validation.
12. Thesis write-up in `thesis/main.tex`.

## Style

Keep the implementation clear, direct, and thesis-friendly. Prefer the simplest working version over broad abstractions.
