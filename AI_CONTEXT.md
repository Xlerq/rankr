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

1. Import reference data, OHLCV prices, and financial data for the full WIG20.
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

The v1 score families are:

- fundamentals: financial condition,
- valuation: cheapness,
- growth/dynamics: revenue and profit growth, not the ROE level alone,
- trend: price trend/momentum.

Each company has component scores (metrics/signals), each with an assigned, fixed weight. The final score is the sum (component score × weight), measures relative growth potential, and determines table sorting. Do not normalize the final score or map the sum to an expected percentage return. Explain the result through its component scores and their weights.

v1 uses one set of weights for the entire WIG20, including banks. Some bank fields may be empty. Do not introduce a separate banking model in v1.

Seasonality (for example, average return in a given month) is only a later option if price history is available; it is not part of MVP. Sector macro and COT are also outside v1 and may only be optional extensions after MVP.

Legacy note: `database/schema.surql` and `database/seed.surql` still describe the old `0-100` / `fundamental-first` model. They are not the authoritative product contract, and future agents must not use them to override the goal above. Leave both files unchanged in this documentation-only task; aligning them with the product goal belongs to later work. This task does not implement scoring or define metric formulas, numeric weights, thresholds, component normalization, or missing-data rules.

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
It collects basic GPW/Notoria fundamentals and obtains the current WIG20 list from
GPW Benchmark. CLI: `collect [CODE]`, `fetch CODE`, `parse FILE`.

- One package with a reusable library and a thin CLI; keep HTTP, pure parsing,
  serializable model types and JSON persistence in separate modules.
- Persist raw JSON before parsing during collection; retain all financial fields.
- Monetary values are Decimal in Rust and decimal strings in JSON, preserving
  currency, unit scale and sign. Missing values are null, not zero.
- Do not introduce parser-version metadata. It was explicitly excluded by the user.
- No database connection or schema changes in the importer stage. `database/`
  contains a legacy design, not the active importer contract.
- The next storage integration is SurrealDB in the same application/data pipeline.
- Prices will come from GPW, not Stooq. Detailed bank fundamentals come later.
- Current source access/publication frequency is accepted; do not reopen this work.
- Analytics and new import code are Rust, as specified in ROADMAP.md.

Run `cargo fmt --all -- --check`, `cargo test --workspace --locked` and
`cargo clippy --workspace --all-targets --locked -- -D warnings` for Rust changes.

## Style

Keep the implementation clear, direct, and thesis-friendly. Prefer the simplest working version over broad abstractions.
