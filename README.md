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
codes (`KGHM`, `PKOBP`, `PZU`), not provider tickers. Codes are case-insensitive.

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
  history.rs  # GPW/Yahoo symbol mapping and legacy archive parsing
  yahoo.rs    # pure Yahoo daily OHLCV JSON parsing and validation
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

Tests use small fixtures and local HTTP servers, without contacting GPW, Yahoo or
TradingView. Old Python/Bash scripts and `data/raw/` samples remain as earlier
research material. Detailed banking fundamentals, macro data and scoring are
later steps.

## Daily company prices and history

Company daily prices come from Yahoo Finance's chart endpoint. No account, API key
or `.env` file is needed. Use GPW company codes, such as `KGHM` and `PKOBP`:

```bash
rankr-import history KGHM   # entire available daily history
rankr-import history        # current WIG20 companies
rankr-import prices KGHM    # latest complete, finished daily candle
rankr-import prices         # current WIG20 daily candles plus FX/gold quotes
```

After changing the source, run the checkout with
`cargo run --locked -p rankr-import -- history KGHM`, or reinstall using
`cargo install --path importer --locked --force`.

`history` requests `interval=1d`, `period1=0`, and the current timestamp as `period2`.
It avoids `range=max`, which can aggregate long histories. `prices` requests the
last month of daily candles and archives the latest complete, finished candle.
Both company commands write the same `history.json` structure; company `prices`
no longer writes an intraday `price.json` snapshot.

Without CODE, GPW Benchmark supplies the current company list. The bundled
`data/raw/wig20_symbols.csv` maps GPW code and ISIN to an explicit `yahoo_symbol`
(e.g. `KGH.WA`, `PKO.WA`, `EBP.WA`, `MDV.WA`). Unknown or changed mappings fail
rather than guessing tickers. Update the map and rebuild/reinstall when needed.
The WIG20 index itself is excluded: Yahoo did not provide usable historical
coverage for it. `history WIG20` is unsupported; the company basket still comes
from the live WIG20 portfolio.

Each response creates an immutable observation:

```text
data/collected/<timestamp>-<suffix>/
  raw.json      # original Yahoo JSON body, company, fetch time, HTTP status, URL
  history.json  # source=yahoo_finance, symbol, currency, candles, skipped_candles
```

Each candle contains `date`, `open`, `high`, `low`, `close`, `adjusted_close`, and
`volume`. Numbers are Decimal values serialized as decimal strings, preserving
Yahoo's precision, including its visible floating-point artifacts. `close` and
Yahoo's split/dividend-adjusted `adjusted_close` are stored separately; no dividend
or split adjustments are recomputed. The importer does not calculate seasonality
or change its formula. Available history may be shorter than the planned signal
lookback, especially for recent listings.

Dates use `Europe/Warsaw`, including DST. Today's candle is withheld until Yahoo's
regular session end plus a 15-minute publication margin; unknown session timing
also withholds today's candle. Older sessions are eligible. Null values (including
adjusted close), invalid values and inconsistent OHLC ranges are omitted and listed
in `skipped_candles` with their dates and reasons, with a warning on stderr. No
missing sessions are filled, no high/low values are swapped, and a live quote is
never substituted for a missing daily close. Check the reported final date: the
latest complete candle can be older than the latest session if Yahoo has gaps.

HTTP/API errors, malformed JSON, mismatched metadata/array lengths, duplicate dates,
or a response without usable finished candles leave `raw.json` and `error.json`.
Other companies continue, and the command exits unsuccessfully if any company
failed. A usable series with omitted rows is saved with its explicit quality report.
Yahoo's chart endpoint is unofficial and may change or throttle requests. The
collector spaces company requests by 500 ms and uses the shared bounded retries.

Full histories remain in ignored `data/collected/` and must not be committed. An
archived response can be parsed offline (all usable candles from the raw response,
including the recent month downloaded by `prices`):

```bash
rankr-import parse /path/to/observation/raw.json > /tmp/history.json
```

Offline parsing also supports old Stooq CSV and GPW quote archives. There are no
active Stooq downloads or GPW company-price requests. The earlier Stooq samples
and database seed retain their original source labels. FX/gold commands such as
`prices USDPLN` use TradingView and write their existing `price.json` snapshots.

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
