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

Engineering thesis project for multi-factor scoring and ranking Polish stock market instruments from GPW.

## MVP

- GPW/WIG20 market and reference data
- deterministic multi-factor scoring and ranking
- basic charts and score history validation notes
- Rust-first web application

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
  parser.rs   # pure HTML -> typed fundamentals
  storage.rs  # raw and parsed JSON archive
  main.rs     # CLI and orchestration
```

The library exposes these modules separately. A future SurrealDB adapter can store
`RawDocument` and `FundamentalSnapshot` and link their records without changing the
HTML parser or HTTP client. The existing files in `database/` are an older design;
this importer does not apply them or connect to a database.

Verification:

```bash
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Tests use fixtures and a local HTTP server, without contacting GPW. Old Python/Bash
scripts and `data/raw/` samples remain as earlier research material. Prices from
GPW, detailed banking fundamentals, macro data and scoring are later steps.

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

The fundamentals importer is implemented; the application backend and frontend are planned.

## Thesis

- LaTeX source: `thesis/main.tex`
- Final PDF target: `thesis/main.pdf`

## License

MIT
