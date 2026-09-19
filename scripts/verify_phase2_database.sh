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

seed_value() {
  local field="$1"
  local value
  value="$(
    grep -E "^[[:space:]]+${field} = " "$SEED" |
      head -n 1 |
      sed -E 's/^[[:space:]]+[[:alnum:]_]+ = (-?[0-9]+([.][0-9]+)?)[,;]$/\1/'
  )" || die "seed missing numeric field: ${field}"
  [[ "$value" =~ ^-?[0-9]+([.][0-9]+)?$ ]] ||
    die "seed field ${field} must contain a numeric literal"
  printf '%s\n' "$value"
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

for field in symbol name isin type exchange currency sector stooq_symbol yahoo_symbol gpw_code gpwbenchmark_name is_active created_at updated_at; do
  require_table_field instrument "$field"
done
require_index 'DEFINE INDEX instrument_symbol_unique ON TABLE instrument FIELDS symbol UNIQUE;' \
  "missing unique index on instrument.symbol"

for field in index instrument as_of index_weight source; do
  require_table_field index_membership "$field"
done
require_index 'DEFINE INDEX index_membership_unique[[:space:]]+ON TABLE index_membership[[:space:]]+FIELDS index, instrument, as_of[[:space:]]+UNIQUE;' \
  "missing unique index on index_membership index+instrument+as_of"

for field in instrument date open high low close adjusted_close volume source; do
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

for field in name description scope sector fundamental_weight technical_weight sentiment_weight is_default is_active created_at updated_at; do
  require_table_field score_config "$field"
done
for field in fundamental_weight technical_weight sentiment_weight; do
  table_block score_config | tr '\n' ' ' | grep -Eq \
    "DEFINE FIELD ${field} ON TABLE score_config TYPE number[[:space:]]+ASSERT "'\$value >= 0;' ||
    die "score_config.${field} must be a nonnegative number without an upper bound"
done
for file in "$SCHEMA" "$SEED"; do
  reject_grep '\b(profitability|financial_strength|cashflow|efficiency|macro_context)_|\btrend_(weight|score)\b|default_fundamental_v1' "$file" \
    "${file#"$ROOT_DIR"/} must not contain legacy scoring fields or configuration names"
  reject_grep '\b(label|excellent|good|neutral|weak|poor)\b' "$file" \
    "${file#"$ROOT_DIR"/} must not contain score labels or their assertions"
done
reject_grep 'momentum_weight|price_risk_weight|trading_liquidity_weight|relative_strength_weight' "$SCHEMA" \
  "score_config must not contain additional scoring families"

for expected in \
  'CREATE score_config:default_growth_v1 SET' \
  'name = "default_growth_v1"' \
  'scope = "global"' \
  'sector = NONE' \
  'is_default = true' \
  'is_active = true'; do
  grep -Fq "$expected" "$SEED" || die "default score_config seed missing: $expected"
done

fundamental_weight="$(seed_value fundamental_weight)"
technical_weight="$(seed_value technical_weight)"
sentiment_weight="$(seed_value sentiment_weight)"
awk -v fundamental="$fundamental_weight" \
    -v technical="$technical_weight" \
    -v sentiment="$sentiment_weight" \
    'BEGIN { exit !(fundamental >= 0 && technical >= 0 && sentiment >= 0 && fundamental > technical && fundamental > sentiment) }' ||
  die "default score_config weights must be nonnegative, with fundamental_weight strictly largest"

for field in instrument config score_date fundamental_snapshot fundamental_score technical_score sentiment_score final_score data_quality_score explanation; do
  require_table_field score_result "$field"
done
for field in fundamental_score technical_score sentiment_score final_score; do
  require_grep "^DEFINE FIELD ${field} ON TABLE score_result TYPE number;$" "$SCHEMA" \
    "score_result.${field} must be an unrestricted number, allowing negative values"
done
table_block score_result | tr '\n' ' ' | grep -Eq \
  'DEFINE FIELD data_quality_score ON TABLE score_result TYPE number[[:space:]]+ASSERT \$value >= 0 AND \$value <= 100;' ||
  die "score_result.data_quality_score must retain its 0-100 range"
table_block score_result | grep -Eq '^DEFINE FIELD (scope|sector) ON TABLE score_result ' &&
  die "score_result must not contain scope or sector fields"
require_index 'DEFINE INDEX score_result_unique[[:space:]]+ON TABLE score_result[[:space:]]+FIELDS instrument, score_date, config[[:space:]]+UNIQUE;' \
  "missing unique index on score_result instrument+score_date+config"

require_grep '^CREATE score_result:bit11_2026_05_22_default_growth_v1 SET' "$SEED" \
  "seed must include the bit11 placeholder score_result"
require_grep 'config = score_config:default_growth_v1,' "$SEED" \
  "placeholder score_result must reference default_growth_v1"
fundamental_score="$(seed_value fundamental_score)"
technical_score="$(seed_value technical_score)"
sentiment_score="$(seed_value sentiment_score)"
final_score="$(seed_value final_score)"
awk -v fundamental="$fundamental_score" -v fundamental_weight="$fundamental_weight" \
    -v technical="$technical_score" -v technical_weight="$technical_weight" \
    -v sentiment="$sentiment_score" -v sentiment_weight="$sentiment_weight" \
    -v final="$final_score" \
    'BEGIN { exit !(final == fundamental * fundamental_weight + technical * technical_weight + sentiment * sentiment_weight) }' ||
  die "placeholder final_score must equal the weighted sum of its three component scores"

for field in source operation status instrument target_symbol started_at finished_at rows_count date_from date_to raw_file_path error_message notes; do
  require_table_field data_source_log "$field"
done
require_grep 'ASSERT \$value INSIDE \["yahoo_finance", "stooq", "gpwbenchmark", "gpw_notoria", "nbp", "manual"\];' "$SCHEMA" \
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
