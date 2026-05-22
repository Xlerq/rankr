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

command -v curl >/dev/null 2>&1 || {
  echo "ERROR: curl is required." >&2
  exit 1
}

mkdir -p "$RAW_DIR"

START_DATE="${START_DATE:-2024-01-01}"
END_DATE="${END_DATE:-2024-01-31}"

fetch_json() {
  local url="$1"
  local output="$2"
  local tmp
  tmp="$(mktemp)"

  curl -fsSL "$url" -o "$tmp"
  mv "$tmp" "$output"
}

fetch_json \
  "https://api.nbp.pl/api/exchangerates/rates/a/usd/${START_DATE}/${END_DATE}/?format=json" \
  "$RAW_DIR/nbp_usdpln_sample.json"

fetch_json \
  "https://api.nbp.pl/api/exchangerates/rates/a/eur/${START_DATE}/${END_DATE}/?format=json" \
  "$RAW_DIR/nbp_eurpln_sample.json"

fetch_json \
  "https://api.nbp.pl/api/cenyzlota/${START_DATE}/${END_DATE}/?format=json" \
  "$RAW_DIR/nbp_gold_sample.json"

echo "Saved NBP samples to data/raw/ for $START_DATE..$END_DATE."
