OPTION IMPORT;

-- rankr SurrealDB schema
-- Phase 2: data model for a fundamental-first ranking application.

DEFINE TABLE instrument SCHEMAFULL;

DEFINE FIELD symbol ON TABLE instrument TYPE string
  ASSERT $value != "";

DEFINE FIELD name ON TABLE instrument TYPE string
  ASSERT $value != "";

DEFINE FIELD isin ON TABLE instrument TYPE string
  ASSERT $value != "";

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


DEFINE TABLE index_membership SCHEMAFULL;

DEFINE FIELD index ON TABLE index_membership TYPE record<instrument>;

DEFINE FIELD instrument ON TABLE index_membership TYPE record<instrument>;

DEFINE FIELD as_of ON TABLE index_membership TYPE datetime;

DEFINE FIELD index_weight ON TABLE index_membership TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD source ON TABLE index_membership TYPE string
  ASSERT $value INSIDE ["gpwbenchmark", "manual"];

DEFINE INDEX index_membership_unique
  ON TABLE index_membership
  FIELDS index, instrument, as_of
  UNIQUE;


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

DEFINE FIELD volume ON TABLE price_daily TYPE number
  ASSERT $value >= 0;

DEFINE FIELD source ON TABLE price_daily TYPE string
  ASSERT $value INSIDE ["stooq", "manual"];

DEFINE INDEX price_daily_unique
  ON TABLE price_daily
  FIELDS instrument, date, source
  UNIQUE;


DEFINE TABLE fundamental_snapshot SCHEMAFULL;

DEFINE FIELD instrument ON TABLE fundamental_snapshot TYPE record<instrument>;

DEFINE FIELD report_period ON TABLE fundamental_snapshot TYPE string
  ASSERT $value != "";

DEFINE FIELD report_year ON TABLE fundamental_snapshot TYPE int
  ASSERT $value >= 1900;

DEFINE FIELD fetched_at ON TABLE fundamental_snapshot TYPE datetime;

DEFINE FIELD unit ON TABLE fundamental_snapshot TYPE string
  ASSERT $value != "";

DEFINE FIELD source ON TABLE fundamental_snapshot TYPE string
  ASSERT $value INSIDE ["gpw_notoria", "manual"];

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

DEFINE INDEX fundamental_snapshot_unique
  ON TABLE fundamental_snapshot
  FIELDS instrument, report_period, source
  UNIQUE;


DEFINE TABLE macro_observation SCHEMAFULL;

DEFINE FIELD date ON TABLE macro_observation TYPE datetime;

DEFINE FIELD series ON TABLE macro_observation TYPE string
  ASSERT $value INSIDE ["USDPLN", "EURPLN", "GOLD_PLN"];

DEFINE FIELD value ON TABLE macro_observation TYPE number;

DEFINE FIELD unit ON TABLE macro_observation TYPE string
  ASSERT $value != "";

DEFINE FIELD source ON TABLE macro_observation TYPE string
  ASSERT $value INSIDE ["nbp", "manual"];

DEFINE FIELD source_url ON TABLE macro_observation TYPE option<string>;

DEFINE INDEX macro_observation_unique
  ON TABLE macro_observation
  FIELDS date, series, source
  UNIQUE;


DEFINE TABLE score_config SCHEMAFULL;

DEFINE FIELD name ON TABLE score_config TYPE string
  ASSERT $value != "";

DEFINE FIELD description ON TABLE score_config TYPE string;

DEFINE FIELD scope ON TABLE score_config TYPE string
  ASSERT $value INSIDE ["global", "sector"];

DEFINE FIELD sector ON TABLE score_config TYPE option<string>;

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


DEFINE TABLE score_result SCHEMAFULL;

DEFINE FIELD instrument ON TABLE score_result TYPE record<instrument>;

DEFINE FIELD config ON TABLE score_result TYPE record<score_config>;

DEFINE FIELD score_date ON TABLE score_result TYPE datetime;

DEFINE FIELD fundamental_snapshot ON TABLE score_result TYPE record<fundamental_snapshot>;

DEFINE FIELD profitability_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD financial_strength_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD cashflow_quality_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD efficiency_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD trend_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD macro_context_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD final_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD label ON TABLE score_result TYPE string
  ASSERT $value INSIDE ["excellent", "good", "neutral", "weak", "poor"];

DEFINE FIELD data_quality_score ON TABLE score_result TYPE number
  ASSERT $value >= 0 AND $value <= 100;

DEFINE FIELD explanation ON TABLE score_result TYPE string;

DEFINE INDEX score_result_unique
  ON TABLE score_result
  FIELDS instrument, score_date, config
  UNIQUE;


DEFINE TABLE data_source_log SCHEMAFULL;

DEFINE FIELD source ON TABLE data_source_log TYPE string
  ASSERT $value INSIDE ["stooq", "gpwbenchmark", "gpw_notoria", "nbp", "manual"];

DEFINE FIELD operation ON TABLE data_source_log TYPE string
  ASSERT $value != "";

DEFINE FIELD status ON TABLE data_source_log TYPE string
  ASSERT $value INSIDE ["success", "failed", "partial"];

DEFINE FIELD instrument ON TABLE data_source_log TYPE option<record<instrument>>;

DEFINE FIELD target_symbol ON TABLE data_source_log TYPE option<string>;

DEFINE FIELD started_at ON TABLE data_source_log TYPE datetime;

DEFINE FIELD finished_at ON TABLE data_source_log TYPE option<datetime>;

DEFINE FIELD rows_count ON TABLE data_source_log TYPE int
  ASSERT $value >= 0;

DEFINE FIELD date_from ON TABLE data_source_log TYPE option<datetime>;

DEFINE FIELD date_to ON TABLE data_source_log TYPE option<datetime>;

DEFINE FIELD raw_file_path ON TABLE data_source_log TYPE option<string>;

DEFINE FIELD error_message ON TABLE data_source_log TYPE option<string>;

DEFINE FIELD notes ON TABLE data_source_log TYPE option<string>;

DEFINE INDEX data_source_log_source_status
  ON TABLE data_source_log
  FIELDS source, status;
