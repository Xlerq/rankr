OPTION IMPORT;

-- rankr SurrealDB schema
-- Phase 2: data model

DEFINE TABLE instrument SCHEMAFULL;

DEFINE FIELD symbol ON TABLE instrument TYPE string
  ASSERT $value != "";

DEFINE FIELD name ON TABLE instrument TYPE string
  ASSERT $value != "";

DEFINE FIELD isin ON TABLE instrument TYPE option<string>;

DEFINE FIELD type ON TABLE instrument TYPE string
  ASSERT $value INSIDE ["stock", "index"];

DEFINE FIELD exchange ON TABLE instrument TYPE string
  ASSERT $value != "";

DEFINE FIELD currency ON TABLE instrument TYPE string
  ASSERT $value != "";

DEFINE FIELD sector ON TABLE instrument TYPE option<string>;

DEFINE FIELD stooq_symbol ON TABLE instrument TYPE option<string>;

DEFINE FIELD gpw_code ON TABLE instrument TYPE option<string>;

DEFINE FIELD gpwbenchmark_name ON TABLE instrument TYPE option<string>;

DEFINE FIELD is_active ON TABLE instrument TYPE bool
  DEFAULT true;

DEFINE FIELD created_at ON TABLE instrument TYPE datetime
  DEFAULT time::now();

DEFINE FIELD updated_at ON TABLE instrument TYPE datetime
  DEFAULT time::now();

DEFINE INDEX instrument_symbol_unique ON TABLE instrument FIELDS symbol UNIQUE;
DEFINE INDEX instrument_isin_unique ON TABLE instrument FIELDS isin UNIQUE;


-- WIG20 membership and index weights from GPW Benchmark.
DEFINE TABLE index_membership SCHEMAFULL;

DEFINE FIELD index ON TABLE index_membership TYPE record<instrument>;

DEFINE FIELD instrument ON TABLE index_membership TYPE record<instrument>;

DEFINE FIELD as_of ON TABLE index_membership TYPE datetime;

DEFINE FIELD index_weight ON TABLE index_membership TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD source ON TABLE index_membership TYPE string
  ASSERT $value INSIDE ["gpwbenchmark"];

DEFINE FIELD created_at ON TABLE index_membership TYPE datetime
  DEFAULT time::now();

DEFINE INDEX index_membership_unique
  ON TABLE index_membership
  FIELDS index, instrument, as_of
  UNIQUE;


-- Daily OHLCV market data from Stooq.
DEFINE TABLE price_daily SCHEMAFULL;

DEFINE FIELD instrument ON TABLE price_daily TYPE record<instrument>;

DEFINE FIELD date ON TABLE price_daily TYPE datetime;

DEFINE FIELD open ON TABLE price_daily TYPE number
  ASSERT $value >= 0;

DEFINE FIELD high ON TABLE price_daily TYPE number
  ASSERT $value >= 0;

DEFINE FIELD low ON TABLE price_daily TYPE number
  ASSERT $value >= 0;

DEFINE FIELD close ON TABLE price_daily TYPE number
  ASSERT $value >= 0;

DEFINE FIELD volume ON TABLE price_daily TYPE option<number>;

DEFINE FIELD source ON TABLE price_daily TYPE string
  ASSERT $value INSIDE ["stooq"];

DEFINE FIELD created_at ON TABLE price_daily TYPE datetime
  DEFAULT time::now();

DEFINE INDEX price_daily_unique
  ON TABLE price_daily
  FIELDS instrument, date, source
  UNIQUE;


-- Periodic company fundamentals from GPW / Notoria.
DEFINE TABLE fundamental_snapshot SCHEMAFULL;

DEFINE FIELD instrument ON TABLE fundamental_snapshot TYPE record<instrument>;

DEFINE FIELD report_period ON TABLE fundamental_snapshot TYPE string
  ASSERT $value != "";

DEFINE FIELD report_year ON TABLE fundamental_snapshot TYPE option<int>;

DEFINE FIELD fetched_at ON TABLE fundamental_snapshot TYPE datetime;

DEFINE FIELD unit ON TABLE fundamental_snapshot TYPE string
  ASSERT $value INSIDE ["tys_pln", "pln", "unknown"];

DEFINE FIELD source ON TABLE fundamental_snapshot TYPE string
  ASSERT $value INSIDE ["gpw_notoria"];

DEFINE FIELD source_url ON TABLE fundamental_snapshot TYPE option<string>;

DEFINE FIELD revenue ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD operating_profit ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD net_income ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD depreciation_amortization ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD assets ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD current_assets ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD equity ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD long_term_liabilities ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD short_term_liabilities ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD operating_cash_flow ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD investing_cash_flow ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD capex ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD financing_cash_flow ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD net_cash_flow ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD ebitda ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD roe ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD roa ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD current_ratio ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD quick_ratio ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD debt_service_ratio ON TABLE fundamental_snapshot TYPE option<number>;

DEFINE FIELD raw_hash ON TABLE fundamental_snapshot TYPE option<string>;

DEFINE FIELD parse_notes ON TABLE fundamental_snapshot TYPE array
  DEFAULT [];

DEFINE FIELD created_at ON TABLE fundamental_snapshot TYPE datetime
  DEFAULT time::now();

DEFINE INDEX fundamental_snapshot_unique
  ON TABLE fundamental_snapshot
  FIELDS instrument, report_period, source
  UNIQUE;


-- Macro, FX and gold observations from NBP.
DEFINE TABLE macro_observation SCHEMAFULL;

DEFINE FIELD date ON TABLE macro_observation TYPE datetime;

DEFINE FIELD series ON TABLE macro_observation TYPE string
  ASSERT $value INSIDE ["USDPLN", "EURPLN", "GOLD_PLN"];

DEFINE FIELD value ON TABLE macro_observation TYPE number
  ASSERT $value >= 0;

DEFINE FIELD unit ON TABLE macro_observation TYPE string
  ASSERT $value INSIDE ["pln_per_usd", "pln_per_eur", "pln_per_gram"];

DEFINE FIELD source ON TABLE macro_observation TYPE string
  ASSERT $value INSIDE ["nbp"];

DEFINE FIELD source_url ON TABLE macro_observation TYPE option<string>;

DEFINE FIELD created_at ON TABLE macro_observation TYPE datetime
  DEFAULT time::now();

DEFINE INDEX macro_observation_unique
  ON TABLE macro_observation
  FIELDS date, series, source
  UNIQUE;


-- Scoring configuration for fundamental-first ranking.
DEFINE TABLE score_config SCHEMAFULL;

DEFINE FIELD name ON TABLE score_config TYPE string
  ASSERT $value != "";

DEFINE FIELD description ON TABLE score_config TYPE option<string>;

DEFINE FIELD profitability_weight ON TABLE score_config TYPE number
  ASSERT $value >= 0 AND $value <= 1;

DEFINE FIELD financial_strength_weight ON TABLE score_config TYPE number
  ASSERT $value >= 0 AND $value <= 1;

DEFINE FIELD cashflow_quality_weight ON TABLE score_config TYPE number
  ASSERT $value >= 0 AND $value <= 1;

DEFINE FIELD efficiency_weight ON TABLE score_config TYPE number
  ASSERT $value >= 0 AND $value <= 1;

DEFINE FIELD trend_weight ON TABLE score_config TYPE number
  ASSERT $value >= 0 AND $value <= 1;

DEFINE FIELD macro_context_weight ON TABLE score_config TYPE number
  ASSERT $value >= 0 AND $value <= 1;

DEFINE FIELD is_default ON TABLE score_config TYPE bool
  DEFAULT false;

DEFINE FIELD is_active ON TABLE score_config TYPE bool
  DEFAULT true;

DEFINE FIELD created_at ON TABLE score_config TYPE datetime
  DEFAULT time::now();

DEFINE FIELD updated_at ON TABLE score_config TYPE datetime
  DEFAULT time::now();

DEFINE INDEX score_config_name_unique
  ON TABLE score_config
  FIELDS name
  UNIQUE;
