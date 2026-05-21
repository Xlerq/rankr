# Architecture

Rankr is planned as a Rust-first engineering thesis web application for scoring and ranking Polish stock market instruments from GPW.

Planned components:
- Frontend: Rust, Leptos, WebAssembly.
- Backend: Rust, Axum, Tokio.
- Database: SurrealDB.
- Charts: Plotters.
- Analytics support: R scripts executed through Rscript.
- Data exchange: JSON and CSV.

The MVP architecture should support GPW/WIG20 end-of-day ingestion, calculation of ranking metrics, storage of processed results, and presentation of ranked instruments in the web UI.

Source code will be added gradually. This file describes the target architecture only and does not define code modules.

