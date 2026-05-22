#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RAW_DIR="$ROOT_DIR/data/raw"

die() {
  echo "ERROR: $*" >&2
  exit 1
}

require_file() {
  local file="$1"
  [[ -f "$file" ]] || die "missing file: ${file#$ROOT_DIR/}"
}

validate_stooq_csv() {
  local file="$1"
  local header
  header="$(head -n 1 "$file")"

  case "$header" in
    "Date,Open,High,Low,Close,Volume"*|"Data,Otwarcie,Najwyzszy,Najnizszy,Zamkniecie,Wolumen"*)
      return 0
      ;;
  esac

  die "invalid Stooq OHLCV header in ${file#$ROOT_DIR/}: $header"
}

validate_json() {
  local file="$1"

  if command -v jq >/dev/null 2>&1; then
    jq -e . "$file" >/dev/null
    return 0
  fi

  die "jq is required to validate JSON samples. Install jq and rerun this script."
}

validate_jq_expr() {
  local file="$1"
  local expr="$2"
  local message="$3"

  if ! command -v jq >/dev/null 2>&1; then
    die "jq is required to validate JSON sample content. Install jq and rerun this script."
  fi

  jq -e "$expr" "$file" >/dev/null || die "$message"
}

validate_wig20_symbols_csv() {
  local file="$1"
  local header
  local rows

  header="$(head -n 1 "$file")"
  [[ "$header" == "symbol,name,isin,stooq_symbol,gpw_code,gpwbenchmark_name,sector,currency,index_weight,source,checked_at" ]] ||
    die "invalid wig20_symbols.csv header: $header"

  rows="$(( $(wc -l < "$file") - 1 ))"
  [[ "$rows" -ge 1 ]] || die "wig20_symbols.csv has no data rows"
}

check_no_hardcoded_keys() {
  local hits
  hits="$(
    grep -RInIE \
      '(api_token|apikey)=[A-Za-z0-9_]{12,}|(EODHD_API_KEY|STOOQ_API_KEY)=[A-Za-z0-9_]{12,}' \
      "$ROOT_DIR" \
      --exclude-dir=.git \
      --exclude-dir=.venv \
      --exclude-dir=venv \
      --exclude-dir=target \
      --exclude-dir=node_modules \
      --exclude-dir=__pycache__ \
      --exclude='.env' \
      --exclude='*.pdf' \
      --exclude='*.png' \
      --exclude='*.zip' || true
  )"

  if [[ -n "$hits" ]]; then
    echo "$hits" >&2
    die "possible hardcoded API key found"
  fi
}

require_file "$RAW_DIR/wig20_symbols.csv"
require_file "$RAW_DIR/wig20_portfolio_gpwbenchmark_sample.json"
require_file "$RAW_DIR/kgh_stooq_sample.csv"
require_file "$RAW_DIR/wig20_stooq_sample.csv"
require_file "$RAW_DIR/nbp_usdpln_sample.json"
require_file "$RAW_DIR/nbp_eurpln_sample.json"
require_file "$RAW_DIR/nbp_gold_sample.json"
require_file "$RAW_DIR/11bit_gpw_notoria_sample.html"
require_file "$RAW_DIR/11bit_gpw_notoria_sample.json"

validate_wig20_symbols_csv "$RAW_DIR/wig20_symbols.csv"

validate_stooq_csv "$RAW_DIR/kgh_stooq_sample.csv"
validate_stooq_csv "$RAW_DIR/wig20_stooq_sample.csv"

validate_json "$RAW_DIR/wig20_portfolio_gpwbenchmark_sample.json"
validate_json "$RAW_DIR/nbp_usdpln_sample.json"
validate_json "$RAW_DIR/nbp_eurpln_sample.json"
validate_json "$RAW_DIR/nbp_gold_sample.json"
validate_json "$RAW_DIR/11bit_gpw_notoria_sample.json"

validate_jq_expr "$RAW_DIR/wig20_portfolio_gpwbenchmark_sample.json" \
  '.http_status == 200 and (.portfolio | length) >= 1' \
  "GPW Benchmark sample must have http_status=200 and at least one portfolio row"

validate_jq_expr "$RAW_DIR/11bit_gpw_notoria_sample.json" \
  '.http_status == 200' \
  "GPW Notoria sample must have http_status=200"

for field in report_period unit revenue net_income assets equity ebitda; do
  validate_jq_expr "$RAW_DIR/11bit_gpw_notoria_sample.json" \
    ".${field} != null" \
    "GPW Notoria sample is missing required field: ${field}"
done

check_no_hardcoded_keys

echo "Data sample verification passed."
