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
