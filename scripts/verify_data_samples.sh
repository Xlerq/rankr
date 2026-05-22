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

check_no_hardcoded_keys() {
  local hits
  hits="$(
    grep -RInIE \
      '(api_token|apikey)=[A-Za-z0-9_]{12,}|(EODHD_API_KEY|STOOQ_API_KEY)=[A-Za-z0-9_]{12,}' \
      "$ROOT_DIR" \
      --exclude-dir=.git \
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
require_file "$RAW_DIR/kgh_stooq_sample.csv"
require_file "$RAW_DIR/wig20_stooq_sample.csv"
require_file "$RAW_DIR/eodhd_war_symbols_sample.json"
require_file "$RAW_DIR/kgh_eodhd_fundamentals_sample.json"
require_file "$RAW_DIR/nbp_usdpln_sample.json"
require_file "$RAW_DIR/nbp_eurpln_sample.json"
require_file "$RAW_DIR/nbp_gold_sample.json"

validate_stooq_csv "$RAW_DIR/kgh_stooq_sample.csv"
validate_stooq_csv "$RAW_DIR/wig20_stooq_sample.csv"

validate_json "$RAW_DIR/eodhd_war_symbols_sample.json"
validate_json "$RAW_DIR/kgh_eodhd_fundamentals_sample.json"
validate_json "$RAW_DIR/nbp_usdpln_sample.json"
validate_json "$RAW_DIR/nbp_eurpln_sample.json"
validate_json "$RAW_DIR/nbp_gold_sample.json"

check_no_hardcoded_keys

echo "Data sample verification passed."
