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

- `fundamental`: financial condition, valuation, and revenue/profit dynamics,
- `technical`: SMA200 price trend and calendar-month seasonality from daily history,
- `sentiment`: a proposed proxy from the KNF public net short-position register, with no importer or scoring implementation.

Each company will have three family scores, each with weight 1. The planned final score is `fundamental_score + technical_score + sentiment_score`, measures relative growth potential, and determines table sorting. Do not normalize it or map the sum to an expected percentage return. The weights are a convention, not calibration.

v1 uses equal weights `1 / 1 / 1` for the three families across the entire WIG20,
including banks. Fundamental indicator templates and thresholds depend on GICS;
this does not introduce sector-specific family weights.

Technical v1 is implemented in the pure `importer/src/technical.rs` module and
exposed through `rankr-import technical --as-of YYYY-MM-DD history.json`.
Its formula is exactly `technical_score = seasonality_points + trend_points`.
Both components are -1, 0 or +1, with no internal weights, 0–100 scaling or
separate momentum signal.

- Input: project only `candles[].date` and `candles[].close` from history JSON,
  including the Yahoo shape. Ignore provider metadata and `adjusted_close`.
  Filter strictly to `date < as_of` before calculations and validation.
- Seasonality: use the calendar month of `as_of`. Average all available completed
  matching months from prior years, using `last_close(month) / last_close(previous
  calendar month) - 1`. Skip pairs with no preceding-month close. One usable year
  is enough. R > 0.02 gives +1; R < -0.02 gives -1; otherwise 0, including the
  exact ±2% boundaries. Zero usable years means `seasonality_points: null` and
  `technical_score: null`, never imputed zero.
- Trend: arithmetic SMA200 of the latest 200 past daily closes, including the
  latest close. `d = abs(close - SMA200) / SMA200`; d < 0.03 gives 0, otherwise
  +1 above SMA200 or -1 below it. Exactly ±3% is directional. Fewer than 200
  sessions gives `trend_points: 0`, with SMA200/d null.
- JSON explanation: R (`r`), usable year count, latest session/close, SMA200,
  d and available trend-session count. Serialize Decimal statistics as exact JSON numbers.
  Reject duplicate past dates and nonpositive past closes explicitly.

Fundamental points are implemented in the pure `importer/src/fundamental_points.rs`
module and exposed through `rankr-import fundamental-points DIRECTORY`.
Keep `fundamental-table` as the separate raw-ratio command without points.

- Reuse the existing five Decimal ratios, including monetary `unit_multiplier`
  handling and `null` for missing inputs or zero denominators. Do not annualize,
  convert currencies or substitute provider ROE/ROA.
- GICS assignments in `data/raw/wig20_symbols.csv` and the fixed thresholds are
  a thesis convention, not fitted to WIG20. Preserve the old `sector` column.
  GICS selects only templates and thresholds, never a peer ranking. Do not use
  quartiles, terciles or other companies' values.
- Outside Financials: margins > 0.02 give +1 and < -0.02 give -1;
  `equity_ratio` > 0.50 gives +1 and < 0.30 gives -1;
  `debt_to_equity` < 0.50 gives +1 and > 1.50 gives -1;
  `ocf_to_net_income` > 1 gives +1 and < 0 gives -1.
  All other available values, including exact boundaries, give 0.
- All of GICS sector 40 (Financials), including Banks, Insurance and Financial
  Services, uses only `equity_ratio`: > 0.10 gives +1, < 0.08 gives -1,
  otherwise 0. The other four ratios and their points are null in the points
  table, even when the raw data is present. This is exclusion, not a penalty.
- Missing ratios have null points. `fundamental_score` sums only available
  points; no available points means null. An available zero yields score 0.
  Do not normalize the sum or combine it with technical here.
- Recursively select the latest `fundamentals.json` per code by `fetched_at`,
  using the same selection as `fundamental-table`. Skip unknown GICS codes and
  bad files with paths on stderr. Output pretty JSON sorted by score descending,
  null last, then code A–Z. Report company and null-score counts on stderr.
  Zero valid companies is an error with empty stdout. No network or database.
- Valuation and year-over-year dynamics remain unimplemented. KNF is selected
  below as a limited sentiment proxy; there is no sentiment importer or scoring.
  The full three-family result remains planned.

There is currently no `rank` command. Keep the standalone family tables; defer
their combination until sentiment is implemented. All three configured weights
are 1, including the future sentiment weight. Missing sentiment is not zero.
Financials keeps its narrower fundamental range (−1 to +1, versus −5 to +5
outside Financials), without rescaling or changing GICS thresholds. Equal
weights do not imply equal ranges or equal influence of the families.

### Selected sentiment source — proposal only

- Source: [KNF public Short Selling Register](https://rss.knf.gov.pl/rss_pub/),
  reached also through `https://rss.knf.gov.pl/`. Its public frontend reads JSON
  from `https://rss.knf.gov.pl/rss_pub/JSON` using POST form field `request`
  containing a JSON object: `cmd=get`, `method=Default`, `language=pl`,
  `limit`/`offset`, `search` and `sort`. This is the site's unversioned endpoint,
  not a guaranteed stable API contract. No API key or account was required in
  the read-only check on 2026-10-21. Do not add download code in this task.
- Verified response: HTTP 200, `application/json`, `status=success`, `total`
  and `records`. An ISIN search for `PLBUDMX00013` returned BUDIMEX's two current
  holder records. The full current response had 20 records, covering 6 of the
  20 ISINs in the project's WIG20 map. These are dated coverage observations,
  not guaranteed future counts or model-performance results.
- Fields: `ISIN`, `ISSUER_NAME`, `HOLDER_FULL_NAME`, `NET_SHORT_POSITION_O`,
  `POSITION_DATE` and `MODIFY_DATE`. The position is a decimal string in percent
  of issued share capital, not the fraction of traders voting bearish or daily
  short-sale turnover. Join to GPW codes by the existing ISIN, not a guessed name.
  A proposed issuer measure is the sum of current disclosed net short-position
  percentages across distinct holders. Use only the current register and the
  latest record per holder/ISIN; do not sum historical disclosures as positions.
- [KNF disclosure explanation](https://www.knf.gov.pl/podmioty/Rejestry_i_Ewidencje/rks?articleId=56674&p_id=18):
  public disclosure starts at 0.5% for a holder, with further 0.1 percentage-point
  thresholds. The 0.1% reporting threshold to KNF is not the public threshold.
  Smaller positions, long positions and the opinions of all investors are not
  covered. No entry does not mean no shorts, a closed position, or crowd optimism.
  Points depend on comparing snapshots, not on interpreting a single missing entry.
- Scoring rule (a convention, not calibration): compare the summed position
  percentages of two snapshots with the same ISIN and a strictly newer `fetched_at`.
  An increase gives −1, a decrease +1, and no change 0, including when the holder
  list changes. There is no 0.5% filter in scoring and no minimum time gap.
  One empty list contributes a total of zero; two empty lists or a missing pair
  give `null`. A positive point means only a
  decline in disclosed short exposure, not measured crowd optimism. Retain both
  source dates and fetch time; a later disclosure must not be backdated in tests
  of predictive performance.
- Rejected: votes about WIG20 as an index, generic forex-style retail positioning
  without an identifiable GPW company, and model-based sentiment from news text.

Seasonality belongs to `technical`, without a fourth family weight. Sector macro
and COT are outside v1 and may only be optional extensions after MVP.

Database contract: `score_config` and `score_result` in `database/schema.surql` define the product contract for the three families above. The generic schema permits nonnegative weights; the v1 seed and verifier require exactly `1 / 1 / 1`. Component scores and `final_score` are unrestricted numbers, including negative values; with these weights the planned final score is the sum of the three family scores. Only `data_quality_score` retains its 0–100 range. `database/seed.surql` provides `default_growth_v1` and a record-shape placeholder: `40 + 10 - 5 = 45`, not a calibrated ranking or actual sentiment data. This contract does not implement scoring or define component formulas.

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
`prices [CODE]`, `history [CODE]`, `fetch CODE`, `parse FILE`,
`technical --as-of YYYY-MM-DD history.json`,
`technical-table --as-of YYYY-MM-DD DIRECTORY`,
`fundamental-table DIRECTORY`, `fundamental-points DIRECTORY`.

- One package with a reusable library and a thin CLI; keep HTTP, pure parsing,
  serializable model types and JSON persistence in separate modules.
- Persist raw JSON before parsing during collection; retain all financial fields.
- Monetary values are Decimal in Rust and exact numbers in JSON, preserving
  currency, unit scale and sign without conversion through f64. Accept legacy
  quoted decimals on input. Missing values are null, not zero.
- Archive directories use `YYYY-MM-DD_CODE_kind` (UTC fetch date), such as
  `2026-10-18_KGHM_history`; repeat names add `_2`, `_3`, etc. Never overwrite
  an existing observation. Older directory names remain readable.
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
