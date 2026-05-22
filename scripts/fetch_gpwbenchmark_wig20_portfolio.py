#!/usr/bin/env python3
from __future__ import annotations

import csv
import json
import re
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import requests
from bs4 import BeautifulSoup


ROOT_DIR = Path(__file__).resolve().parents[1]
RAW_DIR = ROOT_DIR / "data" / "raw"
JSON_OUTPUT = RAW_DIR / "wig20_portfolio_gpwbenchmark_sample.json"
CSV_OUTPUT = RAW_DIR / "wig20_symbols.csv"

INDEX_ISIN = "PL9999999987"
PAGE_URL = f"https://gpwbenchmark.pl/karta-indeksu?isin={INDEX_ISIN}"
AJAX_URL = "https://gpwbenchmark.pl/ajaxindex.php"

HEADERS = {
    "User-Agent": "rankr-thesis-sample/0.1 (+https://github.com/Xlerq/rankr)",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "pl,en;q=0.8",
}

CSV_FIELDS = [
    "symbol",
    "name",
    "isin",
    "stooq_symbol",
    "gpw_code",
    "gpwbenchmark_name",
    "sector",
    "currency",
    "index_weight",
    "source",
    "checked_at",
]

SYMBOL_MAP = {
    "ALIOR": {
        "symbol": "ALR",
        "name": "Alior Bank SA",
        "stooq_symbol": "alr",
        "sector": "Banking",
    },
    "ALLEGRO": {
        "symbol": "ALE",
        "name": "Allegro.eu SA",
        "stooq_symbol": "ale",
        "sector": "E-commerce",
    },
    "BUDIMEX": {
        "symbol": "BDX",
        "name": "Budimex SA",
        "stooq_symbol": "bdx",
        "sector": "Construction",
    },
    "CDPROJEKT": {
        "symbol": "CDR",
        "name": "CD Projekt SA",
        "stooq_symbol": "cdr",
        "sector": "Gaming",
    },
    "DINOPL": {
        "symbol": "DNP",
        "name": "Dino Polska SA",
        "stooq_symbol": "dnp",
        "sector": "Retail",
    },
    "ERSTEPL": {
        "symbol": "SPL",
        "name": "Santander Bank Polska SA",
        "stooq_symbol": "spl",
        "sector": "Banking",
    },
    "KETY": {
        "symbol": "KTY",
        "name": "Grupa Kety SA",
        "stooq_symbol": "kty",
        "sector": "Materials",
    },
    "KGHM": {
        "symbol": "KGH",
        "name": "KGHM Polska Miedz SA",
        "stooq_symbol": "kgh",
        "sector": "Mining",
    },
    "KRUK": {
        "symbol": "KRU",
        "name": "Kruk SA",
        "stooq_symbol": "kru",
        "sector": "Financial services",
    },
    "LPP": {
        "symbol": "LPP",
        "name": "LPP SA",
        "stooq_symbol": "lpp",
        "sector": "Retail",
    },
    "MBANK": {
        "symbol": "MBK",
        "name": "mBank SA",
        "stooq_symbol": "mbk",
        "sector": "Banking",
    },
    "MODIVO": {
        "symbol": "CCC",
        "name": "Modivo SA",
        "stooq_symbol": "ccc",
        "sector": "Retail",
    },
    "PEKAO": {
        "symbol": "PEO",
        "name": "Bank Polska Kasa Opieki SA",
        "stooq_symbol": "peo",
        "sector": "Banking",
    },
    "PEPCO": {
        "symbol": "PCO",
        "name": "Pepco Group NV",
        "stooq_symbol": "pco",
        "sector": "Retail",
    },
    "PGE": {
        "symbol": "PGE",
        "name": "PGE Polska Grupa Energetyczna SA",
        "stooq_symbol": "pge",
        "sector": "Utilities",
    },
    "PKNORLEN": {
        "symbol": "PKN",
        "name": "Orlen SA",
        "stooq_symbol": "pkn",
        "sector": "Energy",
    },
    "PKOBP": {
        "symbol": "PKO",
        "name": "PKO Bank Polski SA",
        "stooq_symbol": "pko",
        "sector": "Banking",
    },
    "PZU": {
        "symbol": "PZU",
        "name": "Powszechny Zaklad Ubezpieczen SA",
        "stooq_symbol": "pzu",
        "sector": "Insurance",
    },
    "TAURONPE": {
        "symbol": "TPE",
        "name": "Tauron Polska Energia SA",
        "stooq_symbol": "tpe",
        "sector": "Utilities",
    },
    "ZABKA": {
        "symbol": "ZAB",
        "name": "Zabka Group SA",
        "stooq_symbol": "zab",
        "sector": "Retail",
    },
}


def text_of(element: Any) -> str:
    return re.sub(r"\s+", " ", element.get_text(" ", strip=True)).strip()


def normalize_decimal(value: str | None) -> str:
    if not value:
        return ""
    return value.replace("\xa0", "").replace(" ", "").replace(",", ".")


def extract_cmng_id(html: str) -> str:
    match = re.search(r"cmng_id=(\d+)", html)
    return match.group(1) if match else "1010"


def extract_as_of(soup: BeautifulSoup) -> str | None:
    text = text_of(soup)
    match = re.search(r"\b(\d{2}-\d{2}-20\d{2})\b", text)
    if not match:
        return None

    day, month, year = match.group(1).split("-")
    return f"{year}-{month}-{day}"


def parse_portfolio(html: str) -> tuple[list[dict[str, Any]], list[str], str | None]:
    soup = BeautifulSoup(html, "html.parser")
    table = soup.find("table")
    if not table:
        return [], [], extract_as_of(soup)

    headers = [text_of(cell) for cell in table.find_all("th")]
    rows: list[dict[str, Any]] = []

    for tr in table.find_all("tr"):
        cells = [text_of(cell) for cell in tr.find_all("td")]
        if len(cells) < 5:
            continue

        link = tr.find("a", href=True)
        rows.append(
            {
                "instrument": cells[0],
                "isin": cells[1],
                "package": normalize_decimal(cells[2]),
                "package_pln": normalize_decimal(cells[3]),
                "index_weight": normalize_decimal(cells[4]),
                "trading_share": normalize_decimal(cells[5]) if len(cells) > 5 else "",
                "average_spread_bps": normalize_decimal(cells[6]) if len(cells) > 6 else "",
                "source_href": link["href"] if link else None,
                "raw_row": cells,
            }
        )

    return rows, headers, extract_as_of(soup)


def build_symbol_rows(
    portfolio: list[dict[str, Any]], checked_at: str
) -> tuple[list[dict[str, str]], list[str]]:
    rows: list[dict[str, str]] = []
    notes: list[str] = []

    for entry in portfolio:
        gpw_name = entry["instrument"]
        mapping = SYMBOL_MAP.get(gpw_name)

        if mapping is None:
            notes.append(f"No local Stooq mapping for GPW Benchmark instrument: {gpw_name}")
            mapping = {
                "symbol": gpw_name,
                "name": gpw_name,
                "stooq_symbol": gpw_name.lower(),
                "sector": "",
            }

        if gpw_name == "ERSTEPL" and entry["isin"] == "PLBZ00000044":
            notes.append(
                "GPW Benchmark sample labels PLBZ00000044 as ERSTEPL; local Stooq mapping uses SPL for this ISIN."
            )

        rows.append(
            {
                "symbol": mapping["symbol"],
                "name": mapping["name"],
                "isin": entry["isin"],
                "stooq_symbol": mapping["stooq_symbol"],
                "gpw_code": gpw_name,
                "gpwbenchmark_name": gpw_name,
                "sector": mapping["sector"],
                "currency": "PLN",
                "index_weight": entry["index_weight"],
                "source": "GPWBenchmark_WIG20_portfolio",
                "checked_at": checked_at,
            }
        )

    return rows, notes


def write_symbols_csv(rows: list[dict[str, str]]) -> None:
    with CSV_OUTPUT.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=CSV_FIELDS, lineterminator="\n")
        writer.writeheader()
        writer.writerows(rows)


def main() -> None:
    RAW_DIR.mkdir(parents=True, exist_ok=True)
    fetched_at = datetime.now(timezone.utc).isoformat()
    checked_at = fetched_at[:10]
    diagnostics: list[str] = []

    session = requests.Session()
    page_response = session.get(PAGE_URL, headers=HEADERS, timeout=20)
    cmng_id = extract_cmng_id(page_response.text)

    ajax_params = {
        "action": "GPWIndexes",
        "start": "ajaxPortfolio",
        "format": "html",
        "lang": "PL",
        "isin": INDEX_ISIN,
        "cmng_id": cmng_id,
        "time": str(int(time.time() * 1000)),
    }
    ajax_headers = {**HEADERS, "Referer": PAGE_URL}
    ajax_response = session.post(
        AJAX_URL,
        params=ajax_params,
        headers=ajax_headers,
        timeout=20,
    )

    portfolio, table_headers, as_of = parse_portfolio(ajax_response.text)
    parse_status = "ok" if portfolio else "fallback"

    if page_response.status_code != 200:
        diagnostics.append(f"Index page returned HTTP {page_response.status_code}.")
    if ajax_response.status_code != 200:
        diagnostics.append(f"Portfolio endpoint returned HTTP {ajax_response.status_code}.")
    if not portfolio:
        diagnostics.append(
            "No portfolio rows were parsed. The table may be rendered dynamically or the HTML structure changed."
        )

    symbol_rows: list[dict[str, str]] = []
    if portfolio:
        symbol_rows, mapping_notes = build_symbol_rows(portfolio, checked_at)
        diagnostics.extend(mapping_notes)
        write_symbols_csv(symbol_rows)

    output = {
        "source": "GPW Benchmark WIG20 portfolio",
        "page_url": PAGE_URL,
        "source_url": ajax_response.url,
        "fetched_at": fetched_at,
        "http_status": ajax_response.status_code,
        "index": "WIG20",
        "index_isin": INDEX_ISIN,
        "cmng_id": cmng_id,
        "as_of": as_of,
        "parse_status": parse_status,
        "table_headers": table_headers,
        "portfolio": portfolio,
        "symbols_csv": str(CSV_OUTPUT.relative_to(ROOT_DIR)) if symbol_rows else None,
        "symbol_rows": symbol_rows,
        "diagnostics": diagnostics,
    }

    JSON_OUTPUT.write_text(
        json.dumps(output, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    print(f"Saved GPW Benchmark sample: {JSON_OUTPUT.relative_to(ROOT_DIR)}")
    if symbol_rows:
        print(f"Updated symbol map: {CSV_OUTPUT.relative_to(ROOT_DIR)}")
    print(f"Portfolio rows parsed: {len(portfolio)}")
    if diagnostics:
        print("Diagnostics: " + " | ".join(diagnostics))


if __name__ == "__main__":
    main()
