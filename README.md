```
           ███████████     █████████   ██████   █████ █████   ████ ███████████  
          ▒▒███▒▒▒▒▒███   ███▒▒▒▒▒███ ▒▒██████ ▒▒███ ▒▒███   ███▒ ▒▒███▒▒▒▒▒███ 
           ▒███    ▒███  ▒███    ▒███  ▒███▒███ ▒███  ▒███  ███    ▒███    ▒███ 
           ▒██████████   ▒███████████  ▒███▒▒███▒███  ▒███████     ▒██████████  
           ▒███▒▒▒▒▒███  ▒███▒▒▒▒▒███  ▒███ ▒▒██████  ▒███▒▒███    ▒███▒▒▒▒▒███ 
           ▒███    ▒███  ▒███    ▒███  ▒███  ▒▒█████  ▒███ ▒▒███   ▒███    ▒███ 
           █████   █████ █████   █████ █████  ▒▒█████ █████ ▒▒████ █████   █████
          ▒▒▒▒▒   ▒▒▒▒▒ ▒▒▒▒▒   ▒▒▒▒▒ ▒▒▒▒▒    ▒▒▒▒▒ ▒▒▒▒▒   ▒▒▒▒ ▒▒▒▒▒   ▒▒▒▒▒ 
```

Engineering thesis project: a web application written entirely in Rust (frontend,
backend, and scoring engine) for ranking GPW companies by relative growth potential
over 3–12 months.

A higher final score means a greater chance of achieving a better return than
other companies in the same basket. The score measures relative growth potential,
not standalone company quality or a predicted percentage return. The application
does not provide investment advice.

## MVP

- the full WIG20 basket, using end-of-day prices, financial and reference data;
  other GPW stocks are a later extension, so 20 companies are not a permanent limit
- a company table as the main view, showing the three family scores,
  their assigned weights, and the final score
- deterministic scoring from exactly three families: `fundamental` (the most
  important family, covering financial condition, valuation, and revenue/profit
  dynamics), `technical` (price trend/momentum and monthly seasonality from
  multi-year daily history), and `sentiment` (sentiment, with its data source
  unspecified)
- the final score is the weighted sum of the three family scores, measures growth
  potential, and determines table sorting; no final-score normalization or
  conversion to an expected percentage return
- one set of fixed weights for the entire WIG20, including banks
- basic charts and score history validation notes

Seasonality belongs to `technical`, without a fourth family weight. Its formula
and the scoring engine remain to be implemented. Sector macro and COT are optional
extensions after MVP.

## Fundamentals importer

The `rankr-import` Rust CLI fetches basic fundamentals for the current WIG20 from
GPW / Notoria. GPW Benchmark supplies the current company list; use its company
codes (`KGHM`, `PKOBP`, `PZU`), not Stooq tickers. Codes are case-insensitive.

Install once from the repository root:

```bash
cargo install --path importer --locked
```

Collect the current WIG20, or one company:

```bash
rankr-import collect
rankr-import collect KGHM
rankr-import collect --output /path/to/archive
```

Each run archives the WIG20 response and creates a new observation directory for
each downloaded company under `data/collected/`:

```text
data/collected/<timestamp>-<suffix>/
  raw.json           # source, company, URL, UTC fetch time, HTTP status, HTML
  fundamentals.json  # company, source metadata and parsed financial fields
```

Raw responses are saved before parsing. A parse/HTTP failure leaves `raw.json`
and an `error.json` explanation. A network failure without a response is reported
to stderr. Collection continues with other companies and exits unsuccessfully if
any company failed. Repeated collections preserve previous files, including
unchanged observations; financial revision deduplication belongs to the later
DB integration. Back up the archive directory to preserve the collected history.

Two small commands are available for inspecting or recovering individual reports:

```bash
rankr-import fetch KGHM > /tmp/kghm.raw.json
rankr-import parse /tmp/kghm.raw.json > /tmp/kghm.fundamentals.json
```

`fetch` writes the raw response to stdout, including a received HTTP error body.
`parse` uses only the supplied raw JSON and writes parsed JSON to stdout. Neither
command creates additional archive files. Diagnostic messages go to stderr, so
JSON output can be piped to `jq`. Use a new output file when reparsing an archive
rather than overwriting its original observation. No scheduler is installed.

All 20 fields from the KGHM financial table are represented explicitly. The model
also includes financial income and the five provider ratios seen in other company
tables. Missing fields are JSON `null`; unknown rows are retained in `extra_fields`.
Malformed known numbers or ambiguous reports cause a parse error. Consolidated
and standalone data remain distinct; the report label is preserved without
inventing publication dates or fiscal period boundaries.

Amounts use `rust_decimal::Decimal` in Rust and decimal strings in JSON, such as
`"24711000.00"`. The original currency, sign and scale are preserved: a value in
thousands has `unit_multiplier: 1000`. The multiplier applies to monetary fields,
not provider ratios. Ratios retain the source scale and are not recomputed.

```text
importer/src/
  model.rs    # serializable data types, independent of HTTP/files/database
  source.rs   # HTTP client and current WIG20 composition
  parser.rs   # pure HTML/raw document -> typed fundamentals/snapshot
  prices.rs   # pure current-quote parsing
  history.rs  # GPW/Stooq symbol mapping and pure daily OHLCV CSV parsing
  storage.rs  # raw and parsed JSON archive
  main.rs     # CLI and orchestration
```

The library exposes these modules separately. A future SurrealDB adapter can store
`RawDocument` and `FundamentalSnapshot` and link their records without changing the
HTML parser or HTTP client. `parser::parse_document(&raw)` is the shared entry point
for turning a response into a validated snapshot. The existing files in `database/`
are an older design; this importer does not apply them or connect to a database.

Verification:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Tests use small fixtures and local HTTP servers, without contacting GPW, Stooq or
TradingView. Old Python/Bash scripts and `data/raw/` samples remain as earlier
research material. Detailed banking fundamentals, macro data and scoring are
later steps.

## Daily price history

`history` downloads the entire available Stooq daily OHLCV series. It supplies
price data for `technical`: trend/momentum and seasonal price behaviour in the
same months over roughly 12 or more years. A newer listing may have a shorter
history; the command reports the candle count and actual date range. Identifying
insufficient history for a signal belongs to the later scoring stage.

Set `STOOQ_API_KEY` in the environment or in a local `.env` file (the environment
takes precedence):

```bash
export STOOQ_API_KEY='your-api-key'
rankr-import history
rankr-import history KGHM
rankr-import history PKOBP --output /path/to/archive
```

Without a code, the command obtains the current WIG20 basket from GPW Benchmark
and adds the WIG20 index. With a code, it imports one company. Use GPW codes such
as `KGHM`, not Stooq tickers such as `kgh`. The bundled
`data/raw/wig20_symbols.csv` maps the GPW code and ISIN to a Stooq symbol. A new or
changed constituent without a matching mapping produces an error; update the
mapping and rebuild/reinstall the importer rather than guessing its ticker.

The request uses the daily interval without date limits. Each response creates
an immutable observation under `data/collected/`:

```text
data/collected/<timestamp>-<suffix>/
  raw.json      # original CSV body, source, instrument, fetch time, HTTP status, URL
  history.json  # source=stooq, instrument, symbol, metadata and daily candles
```

The API key is omitted from the archived URL. The raw response is saved before
parsing; HTTP or CSV failures leave `raw.json` and `error.json`. Other companies
continue to be collected, with a nonzero exit status if any failed. Full histories
stay in the ignored `data/collected/` directory and must not be committed.

Each candle has a date and Decimal OHLCV values, serialized as decimal strings,
including fractional volume. The parser checks nonnegative values, OHLC bounds
and unique dates, then sorts candles by date. It preserves Stooq's scale and
price adjustments, without filling missing sessions or recalculating dividends.
An archived response can be parsed offline:

```bash
rankr-import parse /path/to/observation/raw.json > /tmp/history.json
```

`rankr-import prices [CODE]` remains available for current GPW quotes and
TradingView FX/gold quotes. These produce `price.json` snapshots; they are not
daily history and are not stored as `price_daily` candles by this importer.

## Planned Stack

- Frontend: Rust, Leptos, WebAssembly
- Backend: Rust, Axum, Tokio
- Database: SurrealDB
- Charts: Plotters
- Analytics: R / Rscript
- Data exchange: JSON / CSV

## Out of Scope for MVP

- real-time market data
- trading bot logic
- brokerage integration
- machine learning

The fundamentals, current-quote and daily-history imports are implemented; the
application backend and frontend are planned.

## Thesis

- LaTeX source: `thesis/main.tex`
- Final PDF target: `thesis/main.pdf`

## License

MIT
