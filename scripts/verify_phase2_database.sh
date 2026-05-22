#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
SCHEMA="$ROOT_DIR/database/schema.surql"
SEED="$ROOT_DIR/database/seed.surql"

die() {
  echo "ERROR: $*" >&2
  exit 1
}

require_file() {
  local file="$1"
  [[ -f "$file" ]] || die "missing file: ${file#$ROOT_DIR/}"
}

require_first_instruction() {
  local file="$1"
  local first_line
  first_line="$(head -n 1 "$file")"
  [[ "$first_line" == "OPTION IMPORT;" ]] ||
    die "${file#$ROOT_DIR/} must start with exactly: OPTION IMPORT;"
}

require_grep() {
  local pattern="$1"
  local file="$2"
  local message="$3"
  grep -Eq "$pattern" "$file" || die "$message"
}

reject_grep() {
  local pattern="$1"
  local file="$2"
  local message="$3"
  if grep -Eq "$pattern" "$file"; then
    die "$message"
  fi
}

table_block() {
  local table="$1"
  awk -v table="$table" '
    $0 ~ "^DEFINE TABLE " table " SCHEMAFULL;" { in_block = 1 }
    in_block && $0 ~ "^DEFINE TABLE " && $0 !~ "^DEFINE TABLE " table " SCHEMAFULL;" { exit }
    in_block { print }
  ' "$SCHEMA"
}

require_table_field() {
  local table="$1"
  local field="$2"
  table_block "$table" | grep -Eq "^DEFINE FIELD ${field} ON TABLE ${table} " ||
    die "missing field ${table}.${field}"
}

require_index() {
  local pattern="$1"
  local message="$2"
  tr '\n' ' ' < "$SCHEMA" | grep -Eq "$pattern" || die "$message"
}

require_file "$SCHEMA"
require_file "$SEED"
require_first_instruction "$SCHEMA"
require_first_instruction "$SEED"

required_tables=(
  instrument
  index_membership
  price_daily
  fundamental_snapshot
  macro_observation
  score_config
  score_result
  data_source_log
)

defined_tables="$(grep -E '^DEFINE TABLE ' "$SCHEMA" | awk '{print $3}' | sort | tr '\n' ' ')"
required_sorted="$(printf '%s\n' "${required_tables[@]}" | sort | tr '\n' ' ')"
[[ "$defined_tables" == "$required_sorted" ]] ||
  die "schema tables differ from Phase 2 requirements. Found: $defined_tables"

reject_grep '\bbacktest_result\b' "$SCHEMA" "schema must not define or reference backtest_result"
reject_grep '\bbacktest_result\b' "$SEED" "seed must not create or reference backtest_result"

for table in "${required_tables[@]}"; do
  require_grep "^DEFINE TABLE ${table} SCHEMAFULL;" "$SCHEMA" "missing SCHEMAFULL table: ${table}"
done

for field in symbol name isin type exchange currency sector stooq_symbol gpw_code gpwbenchmark_name is_active created_at updated_at; do
  require_table_field instrument "$field"
done
require_index 'DEFINE INDEX instrument_symbol_unique ON TABLE instrument FIELDS symbol UNIQUE;' \
  "missing unique index on instrument.symbol"

for field in index instrument as_of index_weight source; do
  require_table_field index_membership "$field"
done
require_index 'DEFINE INDEX index_membership_unique[[:space:]]+ON TABLE index_membership[[:space:]]+FIELDS index, instrument, as_of[[:space:]]+UNIQUE;' \
  "missing unique index on index_membership index+instrument+as_of"

for field in instrument date open high low close volume source; do
  require_table_field price_daily "$field"
done
require_index 'DEFINE INDEX price_daily_unique[[:space:]]+ON TABLE price_daily[[:space:]]+FIELDS instrument, date, source[[:space:]]+UNIQUE;' \
  "missing unique index on price_daily instrument+date+source"

for field in instrument report_period report_year fetched_at unit source source_url revenue operating_profit net_income depreciation_amortization assets current_assets equity long_term_liabilities short_term_liabilities operating_cash_flow investing_cash_flow capex financing_cash_flow net_cash_flow ebitda roe roa current_ratio quick_ratio debt_service_ratio raw_hash parse_notes; do
  require_table_field fundamental_snapshot "$field"
done
require_index 'DEFINE INDEX fundamental_snapshot_unique[[:space:]]+ON TABLE fundamental_snapshot[[:space:]]+FIELDS instrument, report_period, source[[:space:]]+UNIQUE;' \
  "missing unique index on fundamental_snapshot instrument+report_period+source"

for field in date series value unit source source_url; do
  require_table_field macro_observation "$field"
done
require_grep 'ASSERT \$value INSIDE \["USDPLN", "EURPLN", "GOLD_PLN"\];' "$SCHEMA" \
  "macro_observation.series must restrict values to USDPLN, EURPLN, GOLD_PLN"
require_index 'DEFINE INDEX macro_observation_unique[[:space:]]+ON TABLE macro_observation[[:space:]]+FIELDS date, series, source[[:space:]]+UNIQUE;' \
  "missing unique index on macro_observation date+series+source"

for field in name description scope sector profitability_weight financial_strength_weight cashflow_quality_weight efficiency_weight trend_weight macro_context_weight is_default is_active created_at updated_at; do
  require_table_field score_config "$field"
done
reject_grep 'momentum_weight|price_risk_weight|trading_liquidity_weight|relative_strength_weight' "$SCHEMA" \
  "score_config must not contain price-action weight fields"

for expected in \
  'name = "default_fundamental_v1"' \
  'scope = "global"' \
  'sector = NONE' \
  'profitability_weight = 0.35' \
  'financial_strength_weight = 0.25' \
  'cashflow_quality_weight = 0.20' \
  'efficiency_weight = 0.10' \
  'trend_weight = 0.10' \
  'macro_context_weight = 0.00' \
  'is_default = true' \
  'is_active = true'; do
  grep -Fq "$expected" "$SEED" || die "default score_config seed missing: $expected"
done

for field in instrument config score_date fundamental_snapshot profitability_score financial_strength_score cashflow_quality_score efficiency_score trend_score macro_context_score final_score label data_quality_score explanation; do
  require_table_field score_result "$field"
done
table_block score_result | grep -Eq 'ASSERT \$value INSIDE \["excellent", "good", "neutral", "weak", "poor"\];' ||
  die "score_result.label must restrict values to excellent, good, neutral, weak, poor"
table_block score_result | grep -Eq '^DEFINE FIELD (scope|sector) ON TABLE score_result ' &&
  die "score_result must not contain scope or sector fields"
require_index 'DEFINE INDEX score_result_unique[[:space:]]+ON TABLE score_result[[:space:]]+FIELDS instrument, score_date, config[[:space:]]+UNIQUE;' \
  "missing unique index on score_result instrument+score_date+config"

for field in source operation status instrument target_symbol started_at finished_at rows_count date_from date_to raw_file_path error_message notes; do
  require_table_field data_source_log "$field"
done
require_grep 'ASSERT \$value INSIDE \["stooq", "gpwbenchmark", "gpw_notoria", "nbp", "manual"\];' "$SCHEMA" \
  "data_source_log.source must restrict allowed sources"
require_grep 'ASSERT \$value INSIDE \["success", "failed", "partial"\];' "$SCHEMA" \
  "data_source_log.status must restrict allowed statuses"
require_grep 'CREATE data_source_log:stooq_wig20_sample_2024_01 SET' "$SEED" \
  "seed must include data_source_log for WIG20 Stooq sample"
require_grep 'target_symbol = "wig20"' "$SEED" \
  "WIG20 Stooq log must contain target_symbol = \"wig20\""

for table in "${required_tables[@]}"; do
  require_grep "CREATE ${table}:" "$SEED" "seed missing sample rows for ${table}"
done

echo "Phase 2 database verification passed."

if command -v surreal >/dev/null 2>&1; then
  cat <<'EOF'

SurrealDB CLI is available. Manual import test commands:

surreal start memory --user root --pass root
surreal import --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr database/schema.surql
surreal import --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr database/seed.surql
surreal sql --endpoint http://localhost:8000 --user root --pass root --ns rankr --db rankr
EOF
else
  echo "SurrealDB CLI is not available; skipped import command hint."
fi
