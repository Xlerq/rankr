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

## Data sources

- Stooq: end-of-day OHLCV prices for GPW instruments and indices.
- GPW Benchmark: current WIG20 composition and index weights.
- GPW / Notoria: experimental free source for company financial data from GPW factsheets.
- NBP: FX and gold prices for macro context.

Full datasets are not committed. The repository tracks only small samples and symbol maps under `data/raw/`.
Private API keys belong in `.env`; this file is ignored by git.
GPW / Notoria scraping is experimental because the AJAX endpoint and HTML structure may change.

Sample commands:

```bash
pip install -r requirements-dev.txt
./scripts/fetch_stooq_samples.sh
./scripts/fetch_nbp_samples.sh
python scripts/fetch_gpwbenchmark_wig20_portfolio.py
python scripts/fetch_gpw_notoria_sample.py
./scripts/verify_data_samples.sh
```

## Database model

SurrealDB is the planned project database.

- Schema: `database/schema.surql`
- Demo seed: `database/seed.surql`
- Notes: `database/README.md`

The Phase 2 database model is tested locally in `memory` mode:

```bash
surreal start memory --user root --pass root
surreal import --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr database/schema.surql
surreal import --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr database/seed.surql
surreal sql --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr
./scripts/verify_phase2_database.sh
```

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

Source code will be added gradually during project development.

## Thesis

- LaTeX source: `thesis/main.tex`
- Final PDF target: `thesis/main.pdf`

## License

MIT
