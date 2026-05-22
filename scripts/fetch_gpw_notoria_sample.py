#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import requests
from bs4 import BeautifulSoup


ROOT_DIR = Path(__file__).resolve().parents[1]
RAW_DIR = ROOT_DIR / "data" / "raw"
HTML_OUTPUT = RAW_DIR / "11bit_gpw_notoria_sample.html"
JSON_OUTPUT = RAW_DIR / "11bit_gpw_notoria_sample.json"

ENDPOINT = "https://www.gpw.pl/ajaxindex.php"
PARAMS = {
    "action": "GPWListaSp",
    "code": "11BIT",
    "format": "html",
    "isin": "PL11BTS00015",
    "lang": "PL",
    "start": "showNotoria",
}

HEADERS = {
    "User-Agent": "rankr-thesis-sample/0.1 (+https://github.com/Xlerq/rankr)",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "pl,en;q=0.8",
}

FIELD_ALIASES = {
    "revenue": [
        "przychody ze sprzedazy",
        "przychody netto ze sprzedazy",
        "przychody ze sprzedaży",
        "przychody netto ze sprzedaży",
        "przychody",
    ],
    "operating_profit": [
        "zysk operacyjny",
        "wynik operacyjny",
        "zysk strata z dzialalnosci operacyjnej",
        "zysk strata z działalności operacyjnej",
    ],
    "net_income": [
        "zysk netto",
        "wynik netto",
        "zysk strata netto",
        "zysk strata netto udzialowcow jednostki dominujacej",
        "zysk strata netto udziałowców jednostki dominującej",
    ],
    "assets": [
        "aktywa razem",
        "aktywa",
    ],
    "equity": [
        "kapital wlasny",
        "kapitał własny",
    ],
    "long_term_liabilities": [
        "zobowiazania dlugoterminowe",
        "zobowiązania długoterminowe",
    ],
    "short_term_liabilities": [
        "zobowiazania krotkoterminowe",
        "zobowiązania krótkoterminowe",
    ],
    "operating_cash_flow": [
        "przeplywy operacyjne",
        "przepływy operacyjne",
        "przeplywy pieniezne netto z dzialalnosci operacyjnej",
        "przepływy pieniężne netto z działalności operacyjnej",
        "srodki pieniezne netto z dzialalnosci operacyjnej",
        "środki pieniężne netto z działalności operacyjnej",
    ],
    "investing_cash_flow": [
        "przeplywy inwestycyjne",
        "przepływy inwestycyjne",
        "przeplywy pieniezne netto z dzialalnosci inwestycyjnej",
        "przepływy pieniężne netto z działalności inwestycyjnej",
        "srodki pieniezne netto z dzialalnosci inwestycyjnej",
        "środki pieniężne netto z działalności inwestycyjnej",
    ],
    "financing_cash_flow": [
        "przeplywy finansowe",
        "przepływy finansowe",
        "przeplywy pieniezne netto z dzialalnosci finansowej",
        "przepływy pieniężne netto z działalności finansowej",
        "srodki pieniezne netto z dzialalnosci finansowej",
        "środki pieniężne netto z działalności finansowej",
    ],
    "ebitda": [
        "ebitda",
    ],
}


def normalize_text(value: str) -> str:
    value = value.lower().strip()
    value = re.sub(r"\s+", " ", value)
    value = value.replace("(", " ").replace(")", " ")
    value = value.replace("-", " ")
    value = value.replace("/", " ")
    return value


def cell_text(cell: Any) -> str:
    return re.sub(r"\s+", " ", cell.get_text(" ", strip=True)).strip()


def extract_tables(soup: BeautifulSoup) -> list[dict[str, Any]]:
    tables: list[dict[str, Any]] = []

    for index, table in enumerate(soup.find_all("table")):
        caption = table.find("caption")
        heading = table.find_previous(["h1", "h2", "h3", "h4", "h5", "h6"])
        headers = [cell_text(cell) for cell in table.find_all("th")]
        rows: list[list[str]] = []

        for row in table.find_all("tr"):
            cells = [cell_text(cell) for cell in row.find_all(["th", "td"])]
            if cells:
                rows.append(cells)

        tables.append(
            {
                "index": index,
                "caption": cell_text(caption) if caption else None,
                "heading": cell_text(heading) if heading else None,
                "headers": headers,
                "rows": rows,
            }
        )

    return tables


def looks_like_period(value: str) -> bool:
    return bool(
        re.search(r"\b20\d{2}\b", value)
        or re.search(r"\b20\d{2}[-/.]\d{2}[-/.]\d{2}\b", value)
        or re.search(r"\bq[1-4]\b", value.lower())
    )


def infer_report_period(tables: list[dict[str, Any]]) -> str | None:
    for table in tables:
        if table.get("heading") and looks_like_period(table["heading"]):
            return table["heading"]

        for header in table["headers"]:
            if looks_like_period(header):
                return header

        for row in table["rows"][:5]:
            for cell in row:
                if looks_like_period(cell):
                    return cell

    return None


def infer_unit(raw_text: str) -> str | None:
    patterns = [
        r"tys\.?\s*zł",
        r"tys\.?\s*pln",
        r"mln\s*zł",
        r"mln\s*pln",
        r"pln",
        r"zł",
    ]

    for pattern in patterns:
        match = re.search(pattern, raw_text, re.IGNORECASE)
        if match:
            return match.group(0)

    return None


def first_numeric_value(row: list[str]) -> str | None:
    for cell in row[1:]:
        if re.search(r"-?\d", cell):
            return cell
    return None


def find_financial_fields(tables: list[dict[str, Any]]) -> tuple[dict[str, Any], list[str]]:
    fields = {field: None for field in FIELD_ALIASES}
    notes: list[str] = []

    for table in tables:
        for row in table["rows"]:
            if len(row) < 2:
                continue

            label = row[0]
            normalized_label = normalize_text(label)

            for field, aliases in FIELD_ALIASES.items():
                if fields[field] is not None:
                    continue

                normalized_aliases = [normalize_text(alias) for alias in aliases]
                if any(alias in normalized_label for alias in normalized_aliases):
                    fields[field] = {
                        "label": label,
                        "value": first_numeric_value(row),
                        "row": row,
                        "table_index": table["index"],
                    }

    missing = [field for field, value in fields.items() if value is None]
    if missing:
        notes.append(
            "Parser did not recognize these fields: " + ", ".join(sorted(missing))
        )

    return fields, notes


def main() -> None:
    RAW_DIR.mkdir(parents=True, exist_ok=True)

    response = requests.get(
        ENDPOINT,
        params=PARAMS,
        headers=HEADERS,
        timeout=15,
    )
    response.raise_for_status()

    html = response.text
    HTML_OUTPUT.write_text(html, encoding=response.encoding or "utf-8")

    soup = BeautifulSoup(html, "html.parser")
    raw_text = re.sub(r"\s+", " ", soup.get_text(" ", strip=True)).strip()
    tables = extract_tables(soup)
    fields, parse_notes = find_financial_fields(tables)

    if not tables:
        parse_notes.append("No HTML tables found in GPW Notoria response.")

    output = {
        "code": "11BIT",
        "isin": "PL11BTS00015",
        "start": "showNotoria",
        "source": "GPW ajaxindex.php / Notoria financial data",
        "source_url": response.url,
        "fetched_at": datetime.now(timezone.utc).isoformat(),
        "http_status": response.status_code,
        "report_period": infer_report_period(tables),
        "unit": infer_unit(raw_text),
        "revenue": fields["revenue"],
        "operating_profit": fields["operating_profit"],
        "net_income": fields["net_income"],
        "assets": fields["assets"],
        "equity": fields["equity"],
        "long_term_liabilities": fields["long_term_liabilities"],
        "short_term_liabilities": fields["short_term_liabilities"],
        "operating_cash_flow": fields["operating_cash_flow"],
        "investing_cash_flow": fields["investing_cash_flow"],
        "financing_cash_flow": fields["financing_cash_flow"],
        "ebitda": fields["ebitda"],
        "table_headers": [
            {
                "table_index": table["index"],
                "caption": table["caption"],
                "heading": table.get("heading"),
                "headers": table["headers"],
            }
            for table in tables
        ],
        "table_rows": [
            {
                "table_index": table["index"],
                "caption": table["caption"],
                "heading": table.get("heading"),
                "rows": table["rows"],
            }
            for table in tables
        ],
        "raw_text": raw_text,
        "parse_notes": parse_notes,
    }

    JSON_OUTPUT.write_text(
        json.dumps(output, ensure_ascii=False, indent=2),
        encoding="utf-8",
    )

    recognized = [
        field
        for field in FIELD_ALIASES
        if output[field] is not None
    ]

    print(f"Saved raw HTML: {HTML_OUTPUT.relative_to(ROOT_DIR)}")
    print(f"Saved parsed JSON: {JSON_OUTPUT.relative_to(ROOT_DIR)}")
    print(f"Tables found: {len(tables)}")
    print("Recognized fields: " + (", ".join(recognized) if recognized else "none"))

    if parse_notes:
        print("Parse notes: " + " | ".join(parse_notes))


if __name__ == "__main__":
    main()
