use chrono::{Datelike, NaiveDate};
use rankr_import::technical::{CloseCandle, TechnicalError, TechnicalHistory, score};
use rust_decimal::Decimal;

const HISTORY: &str = include_str!("fixtures/technical_history.json");

fn fixture() -> TechnicalHistory {
    serde_json::from_str(HISTORY).unwrap()
}

fn date(value: &str) -> NaiveDate {
    value.parse().unwrap()
}

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).unwrap()
}

fn set_close(history: &mut TechnicalHistory, day: &str, close: &str) {
    history
        .candles
        .iter_mut()
        .find(|candle| candle.date == date(day))
        .unwrap()
        .close = decimal(close);
}

#[test]
fn uses_month_ends_raw_close_and_all_available_years() {
    let mut history = fixture();
    // September: +10% in 2024 and -4% in 2025; average +3%.
    // Intra-month extremes, July closes and adjusted_close must not participate.
    set_close(&mut history, "2024-09-30", "110");
    set_close(&mut history, "2025-09-30", "192");
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.seasonality_points, Some(1));
    assert_eq!(result.explanation.r, Some(decimal("0.03")));
    assert_eq!(result.explanation.years, 2);
    assert_eq!(result.trend_points, 0);
    assert_eq!(result.explanation.close, Some(decimal("100")));
    assert_eq!(result.explanation.sma200, Some(decimal("100")));
    assert_eq!(result.explanation.d, Some(Decimal::ZERO));
    assert_eq!(result.explanation.trend_sessions, 200);
    assert_eq!(result.technical_score, Some(1));
}

#[test]
fn excludes_as_of_and_future_candles_before_validation_and_scoring() {
    let mut history = fixture();
    let as_of = date("2026-09-19");
    let past: Vec<_> = history
        .candles
        .iter()
        .filter(|candle| candle.date < as_of)
        .cloned()
        .collect();
    let expected = score(&past, as_of).unwrap();
    // A duplicate and invalid future close cannot affect a past ranking.
    history.candles.push(CloseCandle {
        date: as_of,
        close: Decimal::ZERO,
    });
    history.candles.push(CloseCandle {
        date: date("2027-09-30"),
        close: decimal("-10"),
    });
    assert_eq!(score(&history.candles, as_of).unwrap(), expected);
    assert_eq!(expected.explanation.last_session, Some(date("2026-09-18")));
}

#[test]
fn excludes_the_current_calendar_month_from_seasonality() {
    let mut history = fixture();
    // September 2026 is open: even a large move cannot add a third seasonal year.
    set_close(&mut history, "2026-09-18", "200");
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.explanation.years, 2);
    assert_eq!(result.explanation.r, Some(decimal("0.03")));
    assert_eq!(result.seasonality_points, Some(1));
}

#[test]
fn gives_zero_points_to_a_sideways_season() {
    let mut history = fixture();
    set_close(&mut history, "2024-09-30", "104");
    set_close(&mut history, "2025-09-30", "196");
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.explanation.r, Some(decimal("0.01")));
    assert_eq!(result.seasonality_points, Some(0));
    assert_eq!(result.technical_score, Some(0));
}

#[test]
fn seasonality_thresholds_are_strict_and_symmetric() {
    for (first, second, points, expected_r) in [
        ("102", "204", 0, "0.02"),
        ("98", "196", 0, "-0.02"),
        ("103", "206", 1, "0.03"),
        ("97", "194", -1, "-0.03"),
    ] {
        let mut history = fixture();
        set_close(&mut history, "2024-09-30", first);
        set_close(&mut history, "2025-09-30", second);
        let result = score(&history.candles, date("2026-09-19")).unwrap();
        assert_eq!(result.seasonality_points, Some(points));
        assert_eq!(result.explanation.r, Some(decimal(expected_r)));
    }
}

#[test]
fn one_completed_seasonal_month_is_enough() {
    let mut history = fixture();
    history.candles.retain(|candle| candle.date.year() != 2024);
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.seasonality_points, Some(1));
    assert_eq!(result.explanation.years, 1);
    assert_eq!(result.explanation.r, Some(decimal("0.03")));
}

#[test]
fn january_uses_december_of_the_previous_year() {
    let history: TechnicalHistory =
        serde_json::from_str(include_str!("fixtures/technical_january.json")).unwrap();
    let result = score(&history.candles, date("2025-01-19")).unwrap();
    assert_eq!(result.explanation.r, Some(decimal("0.03")));
    assert_eq!(result.explanation.years, 1);
    assert_eq!(result.seasonality_points, Some(1));
    assert_eq!(result.trend_points, 0);
}

#[test]
fn trend_band_is_three_percent_and_its_boundary_is_directional() {
    for (close, previous_close, points, distance) in [
        ("102.999", "97.001", 0, "0.02999"),
        ("97.001", "102.999", 0, "0.02999"),
        ("103", "97", 1, "0.03"),
        ("97", "103", -1, "0.03"),
        ("104", "96", 1, "0.04"),
        ("96", "104", -1, "0.04"),
    ] {
        let mut history = fixture();
        // 198 closes at 100 + these two closes = 20,000; SMA200 is exactly 100.
        set_close(&mut history, "2026-09-18", close);
        set_close(&mut history, "2026-09-17", previous_close);
        let result = score(&history.candles, date("2026-09-19")).unwrap();
        assert_eq!(result.explanation.sma200, Some(decimal("100")));
        assert_eq!(result.explanation.d, Some(decimal(distance)));
        assert_eq!(result.trend_points, points, "close={close}");
        assert_eq!(result.technical_score, Some(1 + points));
    }
}

#[test]
fn fewer_than_200_sessions_produces_neutral_trend() {
    let mut history = fixture();
    history
        .candles
        .retain(|candle| candle.date < date("2025-12-15"));
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.trend_points, 0);
    assert_eq!(result.explanation.sma200, None);
    assert_eq!(result.explanation.d, None);
    assert_eq!(result.seasonality_points, Some(1));
    assert_eq!(result.technical_score, Some(1));

    let mut history = fixture();
    history
        .candles
        .retain(|candle| candle.date > date("2025-12-15") && candle.date < date("2026-09-19"));
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.explanation.trend_sessions, 199);
    assert_eq!(result.trend_points, 0);
    assert_eq!(result.explanation.sma200, None);
}

#[test]
fn missing_seasonal_month_is_null_instead_of_zero() {
    let mut history = fixture();
    history
        .candles
        .retain(|candle| candle.date >= date("2025-12-15"));
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.seasonality_points, None);
    assert_eq!(result.technical_score, None);
    assert_eq!(result.explanation.r, None);
    assert_eq!(result.explanation.years, 0);
    assert_eq!(result.trend_points, 0);
    let json = serde_json::to_value(result).unwrap();
    assert!(json["seasonality_points"].is_null());
    assert!(json["technical_score"].is_null());
}

#[test]
fn missing_previous_month_does_not_use_an_older_month_as_the_base() {
    let mut history = fixture();
    history
        .candles
        .retain(|candle| candle.date != date("2024-08-30") && candle.date != date("2025-08-29"));
    let result = score(&history.candles, date("2026-09-19")).unwrap();
    assert_eq!(result.seasonality_points, None);
    assert_eq!(result.explanation.years, 0);
}

#[test]
fn empty_and_unsorted_series_have_deterministic_results() {
    let empty = score(&[], date("2026-09-19")).unwrap();
    assert_eq!(empty.trend_points, 0);
    assert_eq!(empty.seasonality_points, None);
    assert_eq!(empty.explanation.close, None);
    let mut history = fixture();
    let expected = score(&history.candles, date("2026-09-19")).unwrap();
    history.candles.reverse();
    assert_eq!(
        score(&history.candles, date("2026-09-19")).unwrap(),
        expected
    );
}

#[test]
fn rejects_invalid_or_duplicate_past_closes_without_panicking() {
    let mut history = fixture();
    set_close(&mut history, "2026-09-18", "0");
    assert_eq!(
        score(&history.candles, date("2026-09-19")),
        Err(TechnicalError::InvalidClose(date("2026-09-18")))
    );
    let mut history = fixture();
    history.candles.push(history.candles[0].clone());
    assert_eq!(
        score(&history.candles, date("2026-09-19")),
        Err(TechnicalError::DuplicateDate(date("2024-08-30")))
    );
}
