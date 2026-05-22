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

In future historical analysis, `fundamental_snapshot` should probably gain `published_at` or `available_at`. That would prevent using a financial report before it was actually available to the market.

## Validation notes

Future OHLCV importers should reject malformed candles:

- `high >= open`
- `high >= close`
- `high >= low`
- `low <= open`
- `low <= close`
- `low <= high`

The active `score_config` weights should sum to `1.0`. The Phase 2 validator checks the default seed configuration.

## Planned API contract

These endpoints are not implemented yet. This is the planned response shape for the backend and frontend contract.

`GET /api/ranking`

```json
{
  "score_date": "2026-05-22",
  "config": "default_fundamental_v1",
  "items": [
    {
      "symbol": "11B",
      "name": "11 bit studios SA",
      "isin": "PL11BTS00015",
      "sector": "Gaming",
      "final_score": 71.7,
      "label": "good",
      "data_quality_score": 90.0,
      "component_scores": {
        "profitability_score": 62.0,
        "financial_strength_score": 90.0,
        "cashflow_quality_score": 85.0,
        "efficiency_score": 55.0,
        "trend_score": 50.0,
        "macro_context_score": 50.0
      },
      "fundamental_snapshot": {
        "report_period": "I-IV kw. 2025",
        "report_year": 2025,
        "source": "gpw_notoria"
      },
      "latest_price": {
        "date": "2024-01-03",
        "close": 116.03
      }
    }
  ]
}
```

`GET /api/instruments/{symbol}`

```json
{
  "symbol": "KGH",
  "name": "KGHM Polska Miedz SA",
  "isin": "PLKGHM000017",
  "type": "stock",
  "exchange": "GPW",
  "currency": "PLN",
  "sector": "Mining",
  "stooq_symbol": "kgh",
  "gpw_code": "KGHM",
  "gpwbenchmark_name": "KGHM",
  "is_active": true
}
```

`GET /api/prices/{symbol}`

```json
{
  "symbol": "KGH",
  "source": "stooq",
  "prices": [
    {
      "date": "2024-01-02",
      "open": 121.871,
      "high": 122.614,
      "low": 117.812,
      "close": 118.951,
      "volume": 475373.24396983
    }
  ]
}
```

`GET /api/scores/{symbol}`

```json
{
  "symbol": "11B",
  "scores": [
    {
      "score_date": "2026-05-22",
      "config": "default_fundamental_v1",
      "fundamental_snapshot": "fundamental_snapshot:bit11_2025_iv_gpw_notoria",
      "final_score": 71.7,
      "label": "good",
      "data_quality_score": 90.0,
      "explanation": "Seed example for fundamental-first scoring.",
      "component_scores": {
        "profitability_score": 62.0,
        "financial_strength_score": 90.0,
        "cashflow_quality_score": 85.0,
        "efficiency_score": 55.0,
        "trend_score": 50.0,
        "macro_context_score": 50.0
      }
    }
  ]
}
```

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
