use rankr_import::{fundamental_points::score, fundamental_ratios::FundamentalRatios};
use rust_decimal::Decimal;

fn value(text: &str) -> Option<Decimal> {
    Some(Decimal::from_str_exact(text).unwrap())
}

fn empty() -> FundamentalRatios {
    FundamentalRatios {
        net_margin: None,
        operating_margin: None,
        equity_ratio: None,
        debt_to_equity: None,
        ocf_to_net_income: None,
    }
}

#[test]
fn nonfinancial_boundaries_are_neutral() {
    for (margin, equity, debt, cash) in [
        ("-0.02", "0.30", "0.50", "0"),
        ("0.02", "0.50", "1.50", "1"),
    ] {
        let result = score(
            FundamentalRatios {
                net_margin: value(margin),
                operating_margin: value(margin),
                equity_ratio: value(equity),
                debt_to_equity: value(debt),
                ocf_to_net_income: value(cash),
            },
            15,
        );
        assert_eq!(result.net_margin_points, Some(0));
        assert_eq!(result.operating_margin_points, Some(0));
        assert_eq!(result.equity_ratio_points, Some(0));
        assert_eq!(result.debt_to_equity_points, Some(0));
        assert_eq!(result.ocf_to_net_income_points, Some(0));
        assert_eq!(result.fundamental_score, Some(0));
    }
}

#[test]
fn values_just_outside_the_bands_follow_each_metrics_direction() {
    for (margin, equity, debt, cash, expected) in [
        (
            "0.020000001",
            "0.500000001",
            "0.499999999",
            "1.000000001",
            1,
        ),
        (
            "-0.020000001",
            "0.299999999",
            "1.500000001",
            "-0.000000001",
            -1,
        ),
    ] {
        let result = score(
            FundamentalRatios {
                net_margin: value(margin),
                operating_margin: value(margin),
                equity_ratio: value(equity),
                debt_to_equity: value(debt),
                ocf_to_net_income: value(cash),
            },
            15,
        );
        assert_eq!(result.net_margin_points, Some(expected));
        assert_eq!(result.operating_margin_points, Some(expected));
        assert_eq!(result.equity_ratio_points, Some(expected));
        assert_eq!(result.debt_to_equity_points, Some(expected));
        assert_eq!(result.ocf_to_net_income_points, Some(expected));
        assert_eq!(result.fundamental_score, Some(5 * expected));
    }
}

#[test]
fn financials_only_scores_equity_with_inclusive_neutral_boundaries() {
    for (equity, expected) in [
        ("0.079999999", -1),
        ("0.08", 0),
        ("0.09", 0),
        ("0.10", 0),
        ("0.100000001", 1),
    ] {
        let result = score(
            FundamentalRatios {
                net_margin: value("0.5"),
                operating_margin: value("-0.5"),
                equity_ratio: value(equity),
                debt_to_equity: value("2"),
                ocf_to_net_income: value("3"),
            },
            40,
        );
        assert_eq!(result.ratios.net_margin, None);
        assert_eq!(result.ratios.operating_margin, None);
        assert_eq!(result.ratios.debt_to_equity, None);
        assert_eq!(result.ratios.ocf_to_net_income, None);
        assert_eq!(result.net_margin_points, None);
        assert_eq!(result.operating_margin_points, None);
        assert_eq!(result.debt_to_equity_points, None);
        assert_eq!(result.ocf_to_net_income_points, None);
        assert_eq!(result.equity_ratio_points, Some(expected));
        assert_eq!(result.fundamental_score, Some(expected));
    }
}

#[test]
fn kghm_materials_does_not_use_financials_equity_thresholds() {
    let mut ratios = empty();
    ratios.equity_ratio = value("0.11");
    let result = score(ratios, 15);
    assert_eq!(result.equity_ratio_points, Some(-1));
    assert_eq!(result.fundamental_score, Some(-1));
}

#[test]
fn missing_points_are_excluded_and_an_available_zero_is_a_real_score() {
    let mut ratios = empty();
    ratios.equity_ratio = value("0.40");
    let result = score(ratios, 15);
    assert_eq!(result.net_margin_points, None);
    assert_eq!(result.operating_margin_points, None);
    assert_eq!(result.equity_ratio_points, Some(0));
    assert_eq!(result.fundamental_score, Some(0));
    assert_eq!(score(empty(), 15).fundamental_score, None);

    let mut ratios = empty();
    ratios.net_margin = value("0.5");
    assert_eq!(score(ratios, 40).fundamental_score, None);
}
