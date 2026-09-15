# Database model

Phase 2 defines the SurrealDB data model for `rankr` before backend, importer, API, UI, or scoring implementation.

The scoring contract ranks WIG20 companies by relative growth potential over 3–12 months. A higher final score means a greater chance of a better return than other companies in the same basket; it is neither a standalone assessment of company quality nor a percentage-return forecast.

## Collections

- `instrument` — stocks and indices, including source-specific symbols.
- `index_membership` — WIG20 membership and GPW Benchmark index weights.
- `price_daily` — Stooq daily OHLCV observations.
- `fundamental_snapshot` — GPW / Notoria financial snapshots.
- `macro_observation` — NBP FX and gold observations.
- `score_config` — weights for the fundamental, technical, and sentiment families, plus configuration metadata.
- `score_result` — the three component scores and their weighted sum for an instrument, date, and configuration.
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

The v1 scoring contract has exactly three families:

- `fundamental` — the most important family: financial condition, valuation, and revenue/profit dynamics.
- `technical` — price trend/momentum.
- `sentiment` — sentiment, with its field present now and no data source specified.

`score_config` stores `fundamental_weight`, `technical_weight`, and `sentiment_weight` as nonnegative numbers, without an upper bound or a required total. v1 uses one set of weights for the entire WIG20, including banks.

`score_result` stores `fundamental_score`, `technical_score`, `sentiment_score`, and `final_score` as unrestricted numbers, including negative values. The final score is the sum of each component score multiplied by its corresponding weight, with no final-score normalization or mapping to a percentage return. Only `data_quality_score` retains its 0–100 range. The unique key remains `(instrument, score_date, config)`; instrument, configuration, date, fundamental snapshot, and explanation are retained.

The seed configuration is `default_growth_v1`, with `is_default = true` and illustrative weights `3 / 1 / 1`; the fundamental weight is strictly larger than either other weight. The placeholder for `instrument:bit11` uses component scores `40 / 10 / -5`, giving a final score of `125`. It demonstrates the record shape, not a calibrated ranking. The validator checks the scoring field contract and this arithmetic; scoring algorithms and component formulas remain unimplemented.

## Planned API contract

These endpoints are not implemented yet. This is the planned response shape for the backend and frontend contract. Scoring values below are the same illustrative placeholders as in the seed.

`GET /api/ranking`

```json
{
  "score_date": "2026-05-22",
  "config": "default_growth_v1",
  "items": [
    {
      "symbol": "11B",
      "name": "11 bit studios SA",
      "isin": "PL11BTS00015",
      "sector": "Gaming",
      "final_score": 125,
      "data_quality_score": 90.0,
      "component_scores": {
        "fundamental_score": 40,
        "technical_score": 10,
        "sentiment_score": -5
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
      "config": "default_growth_v1",
      "fundamental_snapshot": "fundamental_snapshot:bit11_2025_iv_gpw_notoria",
      "final_score": 125,
      "data_quality_score": 90.0,
      "explanation": "Record-shape placeholder with illustrative component scores; not a calibrated ranking or a percentage-return forecast. No sentiment data source is defined.",
      "component_scores": {
        "fundamental_score": 40,
        "technical_score": 10,
        "sentiment_score": -5
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
SELECT instrument, config, fundamental_score, technical_score, sentiment_score, final_score FROM score_result;
SELECT source, operation, status, target_symbol, rows_count FROM data_source_log ORDER BY source;
```

Run repository validation:

```bash
./scripts/verify_phase2_database.sh
```
