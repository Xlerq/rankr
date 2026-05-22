# Database model

Phase 2 defines the SurrealDB data model for `rankr` before backend, importer, API, UI, or scoring implementation.

The model supports a fundamental-first ranking of Polish listed companies. Price data still exists, but it is supporting context for charts and a small trend filter, not the main scoring driver.

## Collections

- `instrument` — stocks and indices, including source-specific symbols.
- `index_membership` — WIG20 membership and GPW Benchmark index weights.
- `price_daily` — Stooq daily OHLCV observations.
- `fundamental_snapshot` — GPW / Notoria financial snapshots.
- `macro_observation` — NBP FX and gold observations.
- `score_config` — fundamental-first scoring weights and scope.
- `score_result` — historical score outputs for an instrument and date.
- `data_source_log` — source fetch/import audit log.

There is no `backtest_result` collection in MVP. The current validation approach is to store `score_result` history and later compare `final_score` with the instrument price series. A portfolio top-N backtest can be added as a future extension.

## Local test

Start SurrealDB in memory:

```bash
surreal start memory --user root --pass root
```

Import schema and seed:

```bash
surreal import --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr database/schema.surql
surreal import --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr database/seed.surql
```

Open a SQL shell:

```bash
surreal sql --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr
```

Example queries:

```surql
SELECT * FROM instrument;
SELECT * FROM fundamental_snapshot;
SELECT * FROM score_config;
SELECT instrument, config, final_score, label FROM score_result;
SELECT source, operation, status, target_symbol, rows_count FROM data_source_log ORDER BY source;
```

Run repository validation:

```bash
./scripts/verify_phase2_database.sh
```
