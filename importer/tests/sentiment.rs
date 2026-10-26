use rankr_import::{model::ShortsSnapshot, sentiment::score};
use rust_decimal::Decimal;

fn snapshots() -> (ShortsSnapshot, ShortsSnapshot) {
    (
        serde_json::from_str(include_str!("fixtures/knf_snapshot_older.json")).unwrap(),
        serde_json::from_str(include_str!("fixtures/knf_snapshot_newer.json")).unwrap(),
    )
}

fn decimal(value: &str) -> Decimal {
    Decimal::from_str_exact(value).unwrap()
}

#[test]
fn increasing_total_scores_minus_one_for_the_same_holders_in_any_order() {
    let (older, newer) = snapshots();
    assert_eq!(score(Some(&older), Some(&newer)), Some(-1));
    let mut reordered = newer.clone();
    reordered.positions.reverse();
    assert_eq!(score(Some(&older), Some(&reordered)), Some(-1));
}

#[test]
fn decreasing_total_scores_plus_one_and_accepts_exactly_half_a_percent() {
    let (older, mut newer) = snapshots();
    for position in &mut newer.positions {
        position.net_short_position_percent = decimal("0.5");
    }
    assert_eq!(score(Some(&older), Some(&newer)), Some(1));
}

#[test]
fn unchanged_total_scores_zero_even_when_positions_change_individually() {
    let (older, mut newer) = snapshots();
    // Swap the holders' shares: 0.5 + 0.7 remains 0.7 + 0.5.
    newer.positions[0].net_short_position_percent = decimal("0.5");
    newer.positions[1].net_short_position_percent = decimal("0.7");
    assert_eq!(score(Some(&older), Some(&newer)), Some(0));
    assert_eq!(
        serde_json::to_value(score(Some(&older), Some(&newer))).unwrap(),
        0
    );
}

#[test]
fn compares_calendar_days_of_fetch_dates_instead_of_elapsed_hours_or_position_dates() {
    let (older, mut newer) = snapshots();
    // The fixtures are 30 calendar days apart, but less than 30 * 24 hours.
    assert!((newer.fetched_at - older.fetched_at).num_hours() < 30 * 24);
    assert_eq!(score(Some(&older), Some(&newer)), Some(-1));

    newer.fetched_at = "2026-09-30T23:59:59Z".parse().unwrap();
    assert_eq!(score(Some(&older), Some(&newer)), None);
    newer.fetched_at = "2026-10-02T00:00:00Z".parse().unwrap();
    // Old position dates do not change the gap between observations.
    for position in &mut newer.positions {
        position.position_date = "2020-01-01".parse().unwrap();
        position.modify_date = "2020-01-02".parse().unwrap();
    }
    assert_eq!(score(Some(&older), Some(&newer)), Some(-1));
    assert_eq!(score(Some(&newer), Some(&older)), None);
    assert_eq!(score(Some(&older), Some(&older)), None);
}

#[test]
fn missing_pairs_and_empty_lists_are_null_including_two_empty_snapshots() {
    let (older, newer) = snapshots();
    for (old, new) in [(None, None), (Some(&older), None), (None, Some(&newer))] {
        let result = score(old, new);
        assert_eq!(result, None);
        assert!(serde_json::to_value(result).unwrap().is_null());
    }
    let mut empty_older = older.clone();
    empty_older.positions.clear();
    let mut empty_newer = newer.clone();
    empty_newer.positions.clear();
    assert_eq!(score(Some(&empty_older), Some(&newer)), None);
    assert_eq!(score(Some(&older), Some(&empty_newer)), None);
    assert_eq!(score(Some(&empty_older), Some(&empty_newer)), None);
}

#[test]
fn changed_holder_sets_are_null_even_when_holder_count_and_totals_are_equal() {
    let (older, mut newer) = snapshots();
    newer.positions[0].net_short_position_percent = decimal("0.7");
    newer.positions[1].net_short_position_percent = decimal("0.5");
    let mut replaced = newer.clone();
    replaced.positions[0].holder = "Different Holder".into();
    assert_eq!(score(Some(&older), Some(&replaced)), None);
    let mut removed = newer.clone();
    removed.positions.pop();
    assert_eq!(score(Some(&older), Some(&removed)), None);
    let mut added = newer;
    let mut extra = added.positions[0].clone();
    extra.holder = "Additional Holder".into();
    added.positions.push(extra);
    assert_eq!(score(Some(&older), Some(&added)), None);
}

#[test]
fn any_position_below_half_a_percent_invalidates_either_snapshot() {
    for percent in ["0.499999999999999999", "0", "-0.5"] {
        let (mut older, mut newer) = snapshots();
        newer.positions[0].net_short_position_percent = decimal(percent);
        assert_eq!(score(Some(&older), Some(&newer)), None);
        let (_, newer) = snapshots();
        older.positions[0].net_short_position_percent = decimal(percent);
        assert_eq!(score(Some(&older), Some(&newer)), None);
    }
}

#[test]
fn matches_by_isin_and_requires_the_same_nonempty_isin() {
    let (older, mut newer) = snapshots();
    newer.code = "RENAMED".into();
    assert_eq!(score(Some(&older), Some(&newer)), Some(-1));
    newer.isin = "PLKGHM000017".into();
    assert_eq!(score(Some(&older), Some(&newer)), None);
    let mut older = older;
    older.isin.clear();
    newer.isin.clear();
    assert_eq!(score(Some(&older), Some(&newer)), None);
}

#[test]
fn decimal_comparison_keeps_changes_smaller_than_float_precision() {
    let (mut older, mut newer) = snapshots();
    older.positions[0].net_short_position_percent = decimal("0.500000000000000001");
    older.positions[1].net_short_position_percent = decimal("0.5");
    newer.positions[0].net_short_position_percent = decimal("0.5");
    newer.positions[1].net_short_position_percent = decimal("0.500000000000000002");
    assert_eq!(score(Some(&older), Some(&newer)), Some(-1));
}

#[test]
fn follows_importer_holder_normalization_and_rejects_duplicates_or_blank_names() {
    let (older, mut newer) = snapshots();
    newer.positions[0].holder = " beta   & partners ".into();
    assert_eq!(score(Some(&older), Some(&newer)), Some(-1));
    let mut duplicate = newer.positions[0].clone();
    duplicate.holder = "BETA & PARTNERS".into();
    newer.positions.push(duplicate);
    assert_eq!(score(Some(&older), Some(&newer)), None);
    let (_, mut newer) = snapshots();
    newer.positions[0].holder = " \t ".into();
    assert_eq!(score(Some(&older), Some(&newer)), None);
}
