use rankr_import::fundamental_ratios::{
    FundamentalRatioInput, FundamentalRatios, RatioError, calculate,
};
use rust_decimal::Decimal;
use serde_json::{Value, json};

fn fixture() -> FundamentalRatioInput {
    let snapshot: Value =
        serde_json::from_str(include_str!("fixtures/fundamental_company.json")).unwrap();
    serde_json::from_value(snapshot["fundamentals"].clone()).unwrap()
}

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).unwrap()
}

#[test]
fn calculates_five_dimensionless_ratios_from_the_thousand_unit_fixture() {
    let mut input = fixture();
    assert_eq!(input.unit_multiplier, 1000);
    let expected = FundamentalRatios {
        net_margin: Some(decimal("0.1")),
        operating_margin: Some(decimal("0.125")),
        equity_ratio: Some(decimal("0.4")),
        debt_to_equity: Some(decimal("0.75")),
        ocf_to_net_income: Some(decimal("1.5")),
    };
    assert_eq!(calculate(&input).unwrap(), expected);
    // Scaling both monetary operands leaves the dimensionless ratio unchanged.
    input.unit_multiplier = 1;
    assert_eq!(calculate(&input).unwrap(), expected);
}

#[test]
fn missing_inputs_stay_null_and_actual_zero_numerators_stay_zero() {
    let mut input = fixture();
    input.net_income_parent = None;
    input.equity_parent = None;
    input.operating_profit = Some(Decimal::ZERO);
    let result = calculate(&input).unwrap();
    assert_eq!(result.net_margin, None);
    assert_eq!(result.operating_margin, Some(Decimal::ZERO));
    assert_eq!(result.equity_ratio, None);
    assert_eq!(result.debt_to_equity, None);
    assert_eq!(result.ocf_to_net_income, None);
    let json = serde_json::to_value(result).unwrap();
    assert!(json["net_margin"].is_null());
    assert_eq!(json["operating_margin"], "0");

    let mut input = fixture();
    input.revenue = None;
    input.assets = None;
    input.operating_cash_flow = None;
    let result = calculate(&input).unwrap();
    assert_eq!(result.net_margin, None);
    assert_eq!(result.operating_margin, None);
    assert_eq!(result.equity_ratio, None);
    assert_eq!(result.ocf_to_net_income, None);
    assert_eq!(result.debt_to_equity, Some(decimal("0.75")));
}

#[test]
fn debt_to_equity_requires_both_liability_amounts() {
    for missing_long_term in [true, false] {
        let mut input = fixture();
        if missing_long_term {
            input.long_term_liabilities = None;
        } else {
            input.short_term_liabilities = None;
        }
        let result = calculate(&input).unwrap();
        assert_eq!(result.debt_to_equity, None);
        assert_eq!(result.equity_ratio, Some(decimal("0.4")));
    }
    let mut input = fixture();
    input.long_term_liabilities = Some(Decimal::ZERO);
    input.short_term_liabilities = Some(Decimal::ZERO);
    assert_eq!(
        calculate(&input).unwrap().debt_to_equity,
        Some(Decimal::ZERO)
    );
}

#[test]
fn zero_denominators_yield_null_in_all_five_ratios() {
    let mut input = fixture();
    input.revenue = Some(Decimal::ZERO);
    input.assets = Some(Decimal::ZERO);
    input.equity_parent = Some(Decimal::ZERO);
    input.net_income_parent = Some(Decimal::ZERO);
    let result = calculate(&input).unwrap();
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        json!({
            "net_margin": null, "operating_margin": null, "equity_ratio": null,
            "debt_to_equity": null, "ocf_to_net_income": null
        })
    );
}

#[test]
fn multiplying_monetary_amounts_checks_decimal_range() {
    let mut input = fixture();
    input.assets = Some(Decimal::MAX);
    assert_eq!(calculate(&input), Err(RatioError::Arithmetic));
    // This value is representable before monetary scaling, so scaling caused the error.
    input.unit_multiplier = 1;
    assert!(calculate(&input).is_ok());
}
