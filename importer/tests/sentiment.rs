use rankr::{model::ShortsSnapshot, sentiment::score};
use rust_decimal::Decimal;

fn snapshots() -> (ShortsSnapshot, ShortsSnapshot) {
    (
        serde_json::from_str(include_str!("fixtures/knf_snapshot_older.json")).unwrap(),
        serde_json::from_str(include_str!("fixtures/knf_snapshot_newer.json")).unwrap(),
    )
}

#[test]
fn increasing_total_scores_minus_one() {
    let (older, mut newer) = snapshots();
    newer.positions[0].holder = "New Holder".into();
    assert_eq!(score(Some(&older), Some(&newer)), Some(-1));
}

#[test]
fn decreasing_total_scores_plus_one() {
    let (older, mut newer) = snapshots();
    for position in &mut newer.positions {
        position.net_short_position_percent = Decimal::new(2, 1);
    }
    assert_eq!(score(Some(&older), Some(&newer)), Some(1));
}

#[test]
fn unchanged_total_scores_zero() {
    let (older, mut newer) = snapshots();
    // Swap the holders' shares: 0.5 + 0.7 remains 0.7 + 0.5.
    newer.positions[0].net_short_position_percent = Decimal::new(5, 1);
    newer.positions[1].net_short_position_percent = Decimal::new(7, 1);
    assert_eq!(score(Some(&older), Some(&newer)), Some(0));
}

#[test]
fn missing_pair_yields_null_and_two_empty_lists_score_zero() {
    let (mut older, mut newer) = snapshots();
    assert_eq!(score(Some(&older), None), None);
    assert_eq!(score(None, Some(&newer)), None);
    assert_eq!(score(None, None), None);
    older.positions.clear();
    newer.positions.clear();
    assert_eq!(score(Some(&older), Some(&newer)), Some(0));
}
