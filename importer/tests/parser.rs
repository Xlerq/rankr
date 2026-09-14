use rankr_import::{model::Fundamentals, parser::{ParseError, parse_fundamentals}};
use rust_decimal::Decimal;

const ELEVEN_BIT: &str = include_str!("../../data/raw/11bit_gpw_notoria_sample.html");
const KGHM: &str = include_str!("fixtures/kghm.html");

fn number(value: &str) -> Option<Decimal> {
    Some(Decimal::from_str_exact(value).unwrap())
}

fn report(rows: &str) -> String {
    format!(
        "<h3>I-II kw. 2026</h3><table>{rows}</table>\
         <div>Dane finansowe w tys. PLN<br>Dane skonsolidowane</div>"
    )
}

#[test]
fn parses_every_financial_field_and_ratio_in_the_existing_source_fixture() {
    assert_eq!(
        parse_fundamentals(ELEVEN_BIT).unwrap(),
        Fundamentals {
            report_period: "I-IV kw. 2025".into(),
            currency: "PLN".into(),
            unit_multiplier: 1_000,
            consolidated: false,
            revenue: number("141034.32"),
            sales_profit: number("24256.12"),
            other_operating_income: number("1190.98"),
            operating_profit: number("21893.18"),
            financial_income: number("1667.71"),
            pretax_profit: number("11500.37"),
            net_income_parent: number("6942.67"),
            depreciation_amortization: number("50224.23"),
            assets: number("268804.56"),
            non_current_assets: number("176030.90"),
            current_assets: number("92773.67"),
            equity_parent: number("236421.69"),
            share_capital: number("241.72"),
            long_term_liabilities: number("3291.49"),
            short_term_liabilities: number("29091.38"),
            operating_cash_flow: number("62525.92"),
            investing_cash_flow: number("-82069.54"),
            capex: number("-37993.69"),
            financing_cash_flow: number("-1538.85"),
            net_cash_flow: number("-21082.46"),
            ebitda: number("72117.40"),
            reported_roe: number("0.03"),
            reported_roa: number("0.03"),
            reported_current_ratio: number("3.19"),
            reported_quick_ratio: number("3.19"),
            reported_debt_service_ratio: number("144.81"),
            extra_fields: Default::default(),
        }
    );
}

#[test]
fn parses_all_twenty_screenshot_fields_without_changing_sign_or_scale() {
    assert_eq!(
        parse_fundamentals(KGHM).unwrap(),
        Fundamentals {
            report_period: "I-II kw. 2026".into(),
            currency: "PLN".into(),
            unit_multiplier: 1_000,
            consolidated: true,
            revenue: number("24711000.00"),
            sales_profit: number("6057000.00"),
            other_operating_income: number("2357000.00"),
            operating_profit: number("8065000.00"),
            pretax_profit: number("7988000.00"),
            net_income_parent: number("5579000.00"),
            depreciation_amortization: number("1033000.00"),
            assets: number("65862000.00"),
            non_current_assets: number("48253000.00"),
            current_assets: number("17609000.00"),
            equity_parent: number("39742000.00"),
            share_capital: number("2000000.00"),
            long_term_liabilities: number("13201000.00"),
            short_term_liabilities: number("12847000.00"),
            operating_cash_flow: number("2556000.00"),
            investing_cash_flow: number("-1618000.00"),
            capex: number("-2467000.00"),
            financing_cash_flow: number("-152000.00"),
            net_cash_flow: number("786000.00"),
            ebitda: number("9098000.00"),
            ..Default::default()
        }
    );
}

#[test]
fn missing_bank_fields_are_null_and_zero_remains_zero() {
    let html = report(
        "<tr><th>AKTYWA</th><td>440 000 000,00</td></tr>\
         <tr><th>Przychody ze sprzedaży</th><td>—</td></tr>\
         <tr><th>Przepływy operacyjne</th><td></td></tr>\
         <tr><th>EBITDA</th><td>0,00</td></tr>",
    );
    let parsed = parse_fundamentals(&html).unwrap();
    assert_eq!(parsed.assets, number("440000000.00"));
    assert_eq!(parsed.revenue, None);
    assert_eq!(parsed.operating_cash_flow, None);
    assert_eq!(parsed.long_term_liabilities, None);
    assert_eq!(parsed.ebitda, number("0.00"));
    let json = serde_json::to_value(parsed).unwrap();
    assert!(json["revenue"].is_null());
    assert!(json["long_term_liabilities"].is_null());
    assert_eq!(json["assets"], "440000000.00");
}

#[test]
fn reads_eur_and_explicit_unit_scales_from_metadata() {
    for (unit, expected_multiplier) in [("tys. EUR", 1_000), ("mln EUR", 1_000_000), ("EUR", 1)] {
        let html = KGHM.replace("tys. PLN", unit);
        let parsed = parse_fundamentals(&html).unwrap();
        assert_eq!(parsed.currency, "EUR");
        assert_eq!(parsed.unit_multiplier, expected_multiplier);
        assert_eq!(parsed.capex, number("-2467000.00"));
    }
}

#[test]
fn handles_cosmetic_layout_changes_and_unicode_numeric_whitespace() {
    let html = report(
        "<tr><td><strong>Przychody ze sprzedaży</strong></td>\
         <td><span>12&nbsp;345&#8239;678,90</span></td></tr>\
         <tr><td>Przepływy inwestycyjne</td><td>−1&nbsp;000,25</td></tr>",
    );
    let parsed = parse_fundamentals(&html).unwrap();
    assert_eq!(parsed.revenue, number("12345678.90"));
    assert_eq!(parsed.investing_cash_flow, number("-1000.25"));
}

#[test]
fn preserves_unknown_rows_without_confusing_accounting_categories() {
    let html = report(
        "<tr><th>AKTYWA</th><td>100,00</td></tr>\
         <tr><th>Zysk/strata netto</th><td>10,00</td></tr>\
         <tr><th>Kapitał własny</th><td>30,00</td></tr>\
         <tr><th>Nowa miara</th><td>tekst źródłowy</td></tr>",
    );
    let parsed = parse_fundamentals(&html).unwrap();
    assert_eq!(parsed.net_income_parent, None);
    assert_eq!(parsed.equity_parent, None);
    assert_eq!(parsed.extra_fields["Zysk/strata netto"], "10,00");
    assert_eq!(parsed.extra_fields["Kapitał własny"], "30,00");
    assert_eq!(parsed.extra_fields["Nowa miara"], "tekst źródłowy");
}

#[test]
fn rejects_malformed_or_unrepresentable_known_numbers() {
    for value in [
        "12 34,00", "1,2,3", "NaN", "1.234,00", "10%", "12 PLN", "--1,00",
        "0,12345678901234567890123456789", "999999999999999999999999999999999999",
    ] {
        let html = report(&format!("<tr><th>AKTYWA</th><td>{value}</td></tr>"));
        assert!(
            matches!(parse_fundamentals(&html), Err(ParseError::InvalidNumber { .. })),
            "accepted invalid number: {value}"
        );
    }
}

#[test]
fn rejects_missing_or_ambiguous_metadata() {
    for html in [
        KGHM.replace("<h3>I-II kw. 2026</h3>", ""),
        KGHM.replace("Dane finansowe w tys. PLN", ""),
        KGHM.replace("Dane skonsolidowane", ""),
        format!("{KGHM}<h3>I-IV kw. 2025</h3>"),
        format!("{KGHM}<p>Dane finansowe w mln EUR</p>"),
        format!("{KGHM}<p>Dane jednostkowe</p>"),
    ] {
        assert!(matches!(parse_fundamentals(&html), Err(ParseError::InvalidMetadata(_))));
    }
}

#[test]
fn rejects_error_pages_and_multiple_reports() {
    assert_eq!(parse_fundamentals("<h1>Sprawdź przeglądarkę</h1>"), Err(ParseError::MissingTable));
    assert_eq!(parse_fundamentals(&format!("{KGHM}{ELEVEN_BIT}")), Err(ParseError::MultipleTables));
}

#[test]
fn rejects_conflicting_duplicates_including_blank_then_present() {
    for first in ["10,00", "-"] {
        let html = report(&format!(
            "<tr><th>AKTYWA</th><td>{first}</td></tr>\
             <tr><th>Aktywa razem</th><td>20,00</td></tr>"
        ));
        assert!(matches!(parse_fundamentals(&html), Err(ParseError::ConflictingField(_))));
    }
}

#[test]
fn rejects_changed_multi_period_row_layout_instead_of_picking_a_column() {
    let html = report("<tr><th>AKTYWA</th><td>100,00</td><td>200,00</td></tr>");
    assert!(matches!(parse_fundamentals(&html), Err(ParseError::InvalidRow(_))));
}

#[test]
fn decimal_values_round_trip_through_json_without_float_conversion() {
    let html = report("<tr><th>AKTYWA</th><td>9 007 199 254 740 993,01</td></tr>");
    let parsed = parse_fundamentals(&html).unwrap();
    let json = serde_json::to_value(&parsed).unwrap();
    assert_eq!(json["assets"], "9007199254740993.01");
    assert_eq!(serde_json::from_value::<Fundamentals>(json).unwrap(), parsed);
}
