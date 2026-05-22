# Database model

This directory contains the SurrealDB data model for rankr.

Phase 2 defines the logical data contract before backend implementation.

Main collections:

- `instrument` — stocks, indices and other market instruments.
- `index_membership` — WIG20 membership and index weights.
- `price_daily` — daily OHLCV market data.
- `fundamental_snapshot` — periodic company fundamentals.
- `macro_observation` — macro/FX/gold observations.
- `score_config` — scoring weights and configuration.
- `score_result` — calculated ranking scores.
- `backtest_result` — backtest summaries.
- `data_source_log` — import and downloader logs.
