#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
RAW_DIR="$ROOT_DIR/data/raw"

if [[ -f "$ROOT_DIR/.env" ]]; then
  set -a
  # shellcheck disable=SC1091
  . "$ROOT_DIR/.env"
  set +a
fi

if [[ -z "${STOOQ_API_KEY:-}" ]]; then
  echo "ERROR: missing STOOQ_API_KEY. Add it to .env or export it before running this script." >&2
  echo "Stooq currently requires an API key for CSV downloads." >&2
  exit 1
fi

command -v curl >/dev/null 2>&1 || {
  echo "ERROR: curl is required." >&2
  exit 1
}

mkdir -p "$RAW_DIR"

START_DATE="${START_DATE:-2024-01-01}"
END_DATE="${END_DATE:-2024-01-31}"
STOOQ_START="${START_DATE//-/}"
STOOQ_END="${END_DATE//-/}"

validate_stooq_csv() {
  local file="$1"
  local header
  header="$(head -n 1 "$file")"

  case "$header" in
    "Date,Open,High,Low,Close,Volume"*|"Data,Otwarcie,Najwyzszy,Najnizszy,Zamkniecie,Wolumen"*)
      return 0
      ;;
  esac

  echo "ERROR: unexpected Stooq CSV header in $file:" >&2
  echo "$header" >&2
  echo "Check STOOQ_API_KEY and the requested symbol/date range." >&2
  return 1
}

fetch_stooq_csv() {
  local symbol="$1"
  local output="$2"
  local tmp
  tmp="$(mktemp)"

  curl -fsSL --get "https://stooq.com/q/d/l/" \
    --data-urlencode "s=$symbol" \
    --data-urlencode "i=d" \
    --data-urlencode "d1=$STOOQ_START" \
    --data-urlencode "d2=$STOOQ_END" \
    --data-urlencode "apikey=$STOOQ_API_KEY" \
    -o "$tmp"

  validate_stooq_csv "$tmp"
  mv "$tmp" "$output"
}

fetch_stooq_csv "kgh" "$RAW_DIR/kgh_stooq_sample.csv"
fetch_stooq_csv "wig20" "$RAW_DIR/wig20_stooq_sample.csv"

echo "Saved Stooq samples to data/raw/ for $START_DATE..$END_DATE."
