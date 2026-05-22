# rankr

Engineering thesis project for multi-factor scoring and ranking Polish stock market instruments from GPW.

## MVP

- GPW/WIG20 market and reference data
- deterministic multi-factor scoring and ranking
- basic charts and backtesting notes
- Rust-first web application

## Data Sources

- Stooq: end-of-day OHLCV prices for GPW instruments and indices.
- EODHD: company fundamentals, instrument metadata, and exchange symbol lists.
- NBP: macro/FX context, including PLN exchange rates and gold prices.

Full datasets are not committed to the repository. Only small samples and symbol maps under `data/raw/` are intended to be tracked.

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
