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

if [[ -z "${EODHD_API_KEY:-}" ]]; then
  echo "ERROR: missing EODHD_API_KEY. Add it to .env or export it before running this script." >&2
  exit 1
fi

command -v curl >/dev/null 2>&1 || {
  echo "ERROR: curl is required." >&2
  exit 1
}

mkdir -p "$RAW_DIR"

fetch_json() {
  local url="$1"
  local output="$2"
  local tmp
  tmp="$(mktemp)"

  curl -fsSL "$url" -o "$tmp"
  mv "$tmp" "$output"
}

fetch_json \
  "https://eodhd.com/api/exchange-symbol-list/WAR?api_token=${EODHD_API_KEY}&fmt=json" \
  "$RAW_DIR/eodhd_war_symbols_sample.json"

fetch_json \
  "https://eodhd.com/api/fundamentals/KGH.WAR?api_token=${EODHD_API_KEY}&fmt=json" \
  "$RAW_DIR/kgh_eodhd_fundamentals_sample.json"

echo "Saved EODHD samples to data/raw/."
