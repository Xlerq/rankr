use std::collections::BTreeMap;

use rust_decimal::Decimal;
use scraper::{ElementRef, Html, Selector};
use thiserror::Error;

use crate::model::{FundamentalSnapshot, Fundamentals, RawDocument, Source};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("HTTP {0}; response does not contain a successful report")]
    HttpStatus(u16),
    #[error("expected a GPW/Notoria fundamentals response")]
    InvalidSource,
    #[error("raw document is missing the instrument")]
    MissingInstrument,
    #[error("no recognized financial table found")]
    MissingTable,
    #[error("multiple financial tables found; cannot choose a report safely")]
    MultipleTables,
    #[error("missing or ambiguous {0}")]
    InvalidMetadata(&'static str),
    #[error("expected a label and one value for row {0:?}")]
    InvalidRow(String),
    #[error("invalid financial number {value:?} in row {label:?}")]
    InvalidNumber { label: String, value: String },
    #[error("conflicting values for field {0:?}")]
    ConflictingField(String),
}

/// Turn an archived response into a snapshot without HTTP or storage access.
/// Both the CLI and future database adapters use this same validation path.
pub fn parse_document(raw: &RawDocument) -> Result<FundamentalSnapshot, ParseError> {
    if !(200..300).contains(&raw.http_status) {
        return Err(ParseError::HttpStatus(raw.http_status));
    }
    if raw.source != Source::GpwNotoria {
        return Err(ParseError::InvalidSource);
    }
    let instrument = raw
        .instrument
        .clone()
        .ok_or(ParseError::MissingInstrument)?;
    Ok(FundamentalSnapshot {
        instrument,
        source: raw.source,
        source_url: raw.url.clone(),
        fetched_at: raw.fetched_at,
        fundamentals: parse_fundamentals(&raw.body)?,
    })
}

/// Parse one GPW/Notoria report without network or filesystem access.
pub fn parse_fundamentals(html: &str) -> Result<Fundamentals, ParseError> {
    let document = Html::parse_document(html);
    let tables = Selector::parse("table").expect("static selector");
    let rows = Selector::parse("tr").expect("static selector");
    let cells = Selector::parse("th, td").expect("static selector");
    let candidates: Vec<_> = document
        .select(&tables)
        .filter(|table| {
            table.select(&rows).any(|row| {
                row.select(&cells)
                    .next()
                    .is_some_and(|cell| field_name(&text(cell)).is_some())
            })
        })
        .collect();
    let table = match candidates.as_slice() {
        [] => return Err(ParseError::MissingTable),
        [table] => table,
        _ => return Err(ParseError::MultipleTables),
    };

    let report_period = report_period(&document)?;
    let metadata = text(document.root_element());
    let (currency, unit_multiplier) = currency_and_unit(&metadata)?;
    let consolidated = consolidation(&metadata)?;
    let mut values = BTreeMap::new();
    let mut extra_fields = BTreeMap::new();

    for row in table.select(&rows) {
        let cells: Vec<_> = row.select(&cells).map(text).collect();
        let Some(label) = cells.first() else {
            continue;
        };
        if cells.len() != 2 || label.is_empty() {
            return Err(ParseError::InvalidRow(label.clone()));
        }
        let value = &cells[1];
        if let Some(field) = field_name(label) {
            let parsed = parse_number(value).map_err(|()| ParseError::InvalidNumber {
                label: label.clone(),
                value: value.clone(),
            })?;
            if values
                .insert(field, parsed)
                .is_some_and(|old| old != parsed)
            {
                return Err(ParseError::ConflictingField(label.clone()));
            }
        } else if extra_fields
            .insert(label.clone(), value.clone())
            .is_some_and(|old| old != *value)
        {
            return Err(ParseError::ConflictingField(label.clone()));
        }
    }

    let get = |field| values.get(field).copied().flatten();
    Ok(Fundamentals {
        report_period,
        currency,
        unit_multiplier,
        consolidated,
        revenue: get("revenue"),
        sales_profit: get("sales_profit"),
        other_operating_income: get("other_operating_income"),
        operating_profit: get("operating_profit"),
        financial_income: get("financial_income"),
        pretax_profit: get("pretax_profit"),
        net_income_parent: get("net_income_parent"),
        depreciation_amortization: get("depreciation_amortization"),
        assets: get("assets"),
        non_current_assets: get("non_current_assets"),
        current_assets: get("current_assets"),
        equity_parent: get("equity_parent"),
        share_capital: get("share_capital"),
        long_term_liabilities: get("long_term_liabilities"),
        short_term_liabilities: get("short_term_liabilities"),
        operating_cash_flow: get("operating_cash_flow"),
        investing_cash_flow: get("investing_cash_flow"),
        capex: get("capex"),
        financing_cash_flow: get("financing_cash_flow"),
        net_cash_flow: get("net_cash_flow"),
        ebitda: get("ebitda"),
        reported_roe: get("reported_roe"),
        reported_roa: get("reported_roa"),
        reported_current_ratio: get("reported_current_ratio"),
        reported_quick_ratio: get("reported_quick_ratio"),
        reported_debt_service_ratio: get("reported_debt_service_ratio"),
        extra_fields,
    })
}

fn text(element: ElementRef<'_>) -> String {
    element
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Match complete labels; e.g. total equity is not parent shareholders' equity.
fn field_name(label: &str) -> Option<&'static str> {
    let normalized = label
        .to_lowercase()
        .replace(['/', '(', ')'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    Some(match normalized.as_str() {
        "przychody ze sprzedaży" | "przychody netto ze sprzedaży" => "revenue",
        "zysk strata ze sprzedaży" => "sales_profit",
        "pozostałe przychody operacyjne" => "other_operating_income",
        "zysk strata z działalności operacyjnej" => "operating_profit",
        "przychody finansowe" => "financial_income",
        "zysk strata brutto" => "pretax_profit",
        "zysk strata netto udziałowców jednostki dominującej" => "net_income_parent",
        "amortyzacja noty" | "amortyzacja" => "depreciation_amortization",
        "aktywa" | "aktywa razem" => "assets",
        "aktywa trwałe" => "non_current_assets",
        "aktywa obrotowe" => "current_assets",
        "kapitał własny udziałowców podmiotu dominującego" => "equity_parent",
        "kapitał podstawowy" => "share_capital",
        "zobowiązania długoterminowe" => "long_term_liabilities",
        "zobowiązania krótkoterminowe" => "short_term_liabilities",
        "przepływy operacyjne" => "operating_cash_flow",
        "przepływy inwestycyjne" => "investing_cash_flow",
        "nabycie rzeczowych aktywów trwałych oraz wartości niematerialnych" => "capex",
        "przepływy finansowe" => "financing_cash_flow",
        "przepływy pieniężne netto" => "net_cash_flow",
        "ebitda" => "ebitda",
        "stopa zwrotu z kapitału własnego roe" => "reported_roe",
        "stopa zwrotu z aktywów roa" => "reported_roa",
        "wskaźnik płynności bieżącej" => "reported_current_ratio",
        "wskaźnik płynności szybkiej" => "reported_quick_ratio",
        "wskaźnik obsługi zadłużenia" => "reported_debt_service_ratio",
        _ => return None,
    })
}

fn report_period(document: &Html) -> Result<String, ParseError> {
    let headings = Selector::parse("h1, h2, h3, h4, h5, h6, caption").expect("static selector");
    let mut periods: Vec<_> = document
        .select(&headings)
        .map(text)
        .filter(|heading| {
            is_year(heading)
                || heading.to_lowercase().contains("kw.")
                    && heading.split(|c: char| !c.is_ascii_digit()).any(is_year)
        })
        .collect();
    periods.sort();
    periods.dedup();
    match periods.as_slice() {
        [period] => Ok(period.clone()),
        _ => Err(ParseError::InvalidMetadata("report period")),
    }
}

fn is_year(value: &str) -> bool {
    value.len() == 4
        && value.bytes().all(|c| c.is_ascii_digit())
        && value
            .parse::<u16>()
            .is_ok_and(|year| (1900..=2199).contains(&year))
}

fn currency_and_unit(text: &str) -> Result<(String, u32), ParseError> {
    let lower = text.to_lowercase();
    let words: Vec<_> = lower.split_whitespace().collect();
    let mut metadata = Vec::new();
    for (index, window) in words.windows(3).enumerate() {
        if window != ["dane", "finansowe", "w"] {
            continue;
        }
        let first = words.get(index + 3).copied().unwrap_or_default();
        let (unit, currency) = match first {
            "tys." | "tys" => (1_000, words.get(index + 4).copied().unwrap_or_default()),
            "mln" | "mln." => (1_000_000, words.get(index + 4).copied().unwrap_or_default()),
            _ => (1, first),
        };
        let currency = if currency == "zł" { "pln" } else { currency };
        if currency.len() != 3 || !currency.bytes().all(|c| c.is_ascii_alphabetic()) {
            return Err(ParseError::InvalidMetadata("currency and unit"));
        }
        metadata.push((currency.to_uppercase(), unit));
    }
    metadata.sort();
    metadata.dedup();
    match metadata.as_slice() {
        [value] => Ok(value.clone()),
        _ => Err(ParseError::InvalidMetadata("currency and unit")),
    }
}

fn consolidation(text: &str) -> Result<bool, ParseError> {
    let lower = text.to_lowercase();
    match (
        lower.contains("dane skonsolidowane"),
        lower.contains("dane jednostkowe"),
    ) {
        (true, false) => Ok(true),
        (false, true) => Ok(false),
        _ => Err(ParseError::InvalidMetadata("consolidation status")),
    }
}

fn parse_number(value: &str) -> Result<Option<Decimal>, ()> {
    let value = value.trim().replace('−', "-");
    if matches!(value.as_str(), "" | "-" | "–" | "—") {
        return Ok(None);
    }
    let unsigned = value.strip_prefix(['-', '+']).unwrap_or(&value);
    let mut parts = unsigned.split(',');
    let integer = parts.next().ok_or(())?;
    let fraction = parts.next();
    if parts.next().is_some()
        || fraction.is_some_and(|part| part.is_empty() || !part.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err(());
    }
    let groups: Vec<_> = integer.split_whitespace().collect();
    if groups.is_empty()
        || groups
            .iter()
            .any(|group| !group.bytes().all(|c| c.is_ascii_digit()))
        || (groups.len() > 1
            && (groups[0].len() > 3 || groups[1..].iter().any(|group| group.len() != 3)))
    {
        return Err(());
    }
    let canonical: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    Decimal::from_str_exact(&canonical.replace(',', "."))
        .map(Some)
        .map_err(|_| ())
}
