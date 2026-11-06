use super::*;
use chrono::Duration;
use scraper::{Html as ParsedHtml, Selector};
use serde_json::{Value, json};
use std::{fs, path::Path as FsPath};

fn as_of() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 9, 19).unwrap()
}

fn write_json(root: &FsPath, relative: &str, value: &Value) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
}

fn report(code: &str) -> Value {
    let mut report: Value =
        serde_json::from_str(include_str!("../tests/fixtures/fundamental_company.json")).unwrap();
    report["instrument"]["code"] = json!(code);
    report
}

fn card(root: &FsPath, code: &str) -> CompanyCard {
    let index = ArchiveIndex::read(root).unwrap();
    scorecard::load(&index, code, as_of()).unwrap().unwrap()
}

fn text(html: &ParsedHtml, selector: &str) -> Vec<String> {
    html.select(&Selector::parse(selector).unwrap())
        .map(|element| element.text().collect::<String>())
        .collect()
}

#[test]
fn ranking_preserves_order_and_fields_with_empty_null_and_numeric_zero() {
    let row = |code: &str, final_score| RankingRow {
        code: code.into(),
        gics_sector: "Materials".into(),
        fundamental_score: Some(1),
        technical_score: Some(-1),
        sentiment_score: Some(0),
        final_score,
    };
    let html =
        ParsedHtml::parse_document(&render(&[row("PGE", None), row("KGHM", Some(0))], as_of()));
    assert_eq!(text(&html, "tbody .company-name"), ["PGE", "KGHM"]);
    assert_eq!(text(&html, "tbody .final-score"), ["", "0"]);
    assert_eq!(
        text(&html, "tbody tr:first-child .score"),
        ["", "↑1", "↓−1", "—0"]
    );
    let links: Vec<_> = html
        .select(&Selector::parse("tbody a").unwrap())
        .map(|a| a.value().attr("href").unwrap())
        .collect();
    assert_eq!(links, ["/PGE", "/KGHM"]);
}

#[test]
fn null_on_card_never_becomes_zero_an_arrow_or_a_state_color() {
    let directory = tempfile::tempdir().unwrap();
    let mut input = report("KGHM");
    for field in [
        "revenue",
        "net_income_parent",
        "operating_profit",
        "equity_parent",
        "assets",
        "long_term_liabilities",
        "short_term_liabilities",
        "operating_cash_flow",
    ] {
        input["fundamentals"][field] = Value::Null;
    }
    write_json(directory.path(), "fundamentals.json", &input);
    let html =
        ParsedHtml::parse_document(&render_company(&card(directory.path(), "KGHM"), as_of()));
    let scores: Vec<_> = html.select(&Selector::parse(".score").unwrap()).collect();
    assert_eq!(scores.len(), 9); // final, three families, five fundamental metrics
    for score in scores {
        assert_eq!(score.text().collect::<String>(), "");
        assert_eq!(score.value().attr("class"), Some("score"));
        assert!(
            score
                .select(&Selector::parse(".arrow").unwrap())
                .next()
                .is_none()
        );
    }
    assert!(
        text(&html, ".missing")
            .iter()
            .any(|s| s == "brak drugiego odczytu rejestru")
    );
}

#[test]
fn financials_only_shows_equity_even_when_report_contains_positive_margins() {
    for code in ["PKOBP", "PZU", "KRUK"] {
        let directory = tempfile::tempdir().unwrap();
        write_json(directory.path(), "fundamentals.json", &report(code));
        let html =
            ParsedHtml::parse_document(&render_company(&card(directory.path(), code), as_of()));
        assert_eq!(text(&html, ".metric h3"), ["Kapitał"]);
        assert!(text(&html, ".metric dt").contains(&"equity_ratio".into()));
        assert_eq!(text(&html, ".metric .arrow"), ["↑"]);
        assert!(
            text(&html, ".metric dt")
                .iter()
                .all(|s| !s.contains("margin"))
        );
    }
}

#[test]
fn card_exposes_report_units_amounts_ratios_and_technical_explanation() {
    let directory = tempfile::tempdir().unwrap();
    write_json(directory.path(), "fundamentals.json", &report("KGHM"));
    let mut candles = vec![
        json!({"date":"2025-08-29", "close":100}),
        json!({"date":"2025-09-30", "close":103}),
    ];
    for days in (1..=200).rev() {
        candles.push(json!({"date":as_of() - Duration::days(days), "close":if days == 1 { 110 } else { 100 }}));
    }
    write_json(
        directory.path(),
        "history.json",
        &json!({
            "instrument":{"code":"KGHM"}, "fetched_at":"2026-09-19T10:00:00Z", "candles":candles
        }),
    );
    let company = card(directory.path(), "KGHM");
    let technical = company.technical.as_ref().unwrap();
    assert_eq!(technical.technical_score, company.row.technical_score);
    let html = ParsedHtml::parse_document(&render_company(&company, as_of()));
    assert_eq!(
        text(&html, "[data-metric=net_margin] dd"),
        ["100", "1000", "10%"]
    );
    assert_eq!(
        text(&html, "[data-metric=debt_to_equity] dd"),
        ["200", "100", "400", "0.75"]
    );
    assert_eq!(
        text(&html, "[data-metric=seasonality_points] dd"),
        ["3%", "1"]
    );
    assert_eq!(
        text(&html, "[data-metric=trend_points] dd"),
        ["110", "100.05", "9.945%"]
    );
    assert!(
        text(&html, ".source-note")
            .iter()
            .any(|s| s.contains("PLN × 1000"))
    );
}

#[test]
fn sentiment_details_use_the_rankings_pair_and_escape_holder_names() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let snapshot = |fetched_at: &str, percent| {
        json!({
            "code":"KGHM", "isin":"PLKGHM000017", "fetched_at":fetched_at,
            "positions":[{"holder":"A <script> & B", "net_short_position_percent":percent,
                "position_date":"2026-09-17", "modify_date":"2026-09-18"}]
        })
    };
    write_json(
        root,
        "old/shorts.json",
        &snapshot("2026-09-18T18:00:00Z", "1.5"),
    );
    write_json(
        root,
        "same-day/shorts.json",
        &snapshot("2026-09-19T08:00:00Z", "5"),
    );
    write_json(
        root,
        "new/shorts.json",
        &snapshot("2026-09-19T12:00:00Z", "1.2"),
    );
    let company = card(root, "KGHM");
    assert_eq!(company.row.sentiment_score, Some(1));
    let rendered = render_company(&company, as_of());
    let html = ParsedHtml::parse_document(&rendered);
    assert_eq!(
        text(&html, ".short-totals dd"),
        [
            "1.2%",
            "2026-09-19T12:00:00+00:00",
            "1.5%",
            "2026-09-18T18:00:00+00:00"
        ]
    );
    assert_eq!(
        text(&html, ".holders tbody td"),
        ["A <script> & B", "1.2%", "2026-09-17"]
    );
    assert!(!rendered.contains("<script>"));
}

#[test]
fn empty_register_has_no_fabricated_holders_and_real_zero_stays_zero() {
    let directory = tempfile::tempdir().unwrap();
    for (path, date) in [
        ("old/shorts.json", "2026-09-18T12:00:00Z"),
        ("new/shorts.json", "2026-09-19T12:00:00Z"),
    ] {
        write_json(
            directory.path(),
            path,
            &json!({"code":"KGHM", "isin":"PLKGHM000017", "fetched_at":date, "positions":[]}),
        );
    }
    let company = card(directory.path(), "KGHM");
    assert_eq!(company.row.sentiment_score, Some(0));
    let html = ParsedHtml::parse_document(&render_company(&company, as_of()));
    assert!(text(&html, ".holders tbody tr").is_empty());
    assert_eq!(text(&html, ".short-totals dd")[0], "0%");
    assert_eq!(
        text(&html, ".band:last-of-type .band-heading .score"),
        ["—0"]
    );
}

#[tokio::test]
async fn axum_serves_ranking_card_and_polish_not_found() {
    let directory = tempfile::tempdir().unwrap();
    write_json(directory.path(), "fundamentals.json", &report("KGHM"));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = router(directory.path().to_path_buf()).unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let client = reqwest::Client::new();
    for (path, expected_status, expected_text) in [
        ("/", StatusCode::OK, "Ranking spółek"),
        ("/KGHM", StatusCode::OK, "Karta spółki"),
        (
            "/UNKNOWN",
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki o podanym kodzie.",
        ),
    ] {
        let response = client
            .get(format!("http://{address}{path}"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), expected_status);
        assert!(response.text().await.unwrap().contains(expected_text));
    }
    server.abort();
}

fn ohlc_history(code: &str) -> Value {
    json!({
        "instrument":{"code":code}, "currency":"PLN", "fetched_at":"2026-09-19T12:00:00Z",
        "candles":[
            {"date":"2020-01-02", "open":10, "high":13, "low":9, "close":12},
            {"date":"2026-09-18", "open":12, "high":13, "low":10, "close":11}
        ]
    })
}

fn saved_ranking(root: &FsPath, name: &str, date: &str, time: &str, value: Value) {
    write_json(
        root,
        &format!("rankings/{name}/ranking.json"),
        &json!({
            "as_of":date, "computed_at":time,
            "rows":[{"code":"KGHM", "final_score":value}, {"code":"PGE", "final_score":9}]
        }),
    );
}

#[test]
fn company_without_saved_rankings_gets_real_candles_and_empty_score_panel() {
    let directory = tempfile::tempdir().unwrap();
    write_json(
        directory.path(),
        "2026-09-19_KGHM_history/history.json",
        &ohlc_history("KGHM"),
    );
    let company = card(directory.path(), "KGHM");
    assert!(company.saved_scores.is_empty());
    let html = ParsedHtml::parse_document(&render_company(&company, as_of()));
    assert_eq!(
        text(&html, ".price-chart .candle title"),
        [
            "2020-01-02: O 10 · H 13 · L 9 · C 12",
            "2026-09-18: O 12 · H 13 · L 10 · C 11"
        ]
    );
    assert_eq!(text(&html, ".price-chart .positive").len(), 1);
    assert_eq!(text(&html, ".price-chart .negative").len(), 1);
    assert_eq!(text(&html, ".score-chart"), [""]);
    assert_eq!(text(&html, ".chart-panel svg").len(), 1);
    assert_eq!(
        text(&html, ".price-score-chart .price-chart .candle").len(),
        2
    );
    assert!(
        html.select(&Selector::parse(".price-score-chart .score-chart *").unwrap())
            .next()
            .is_none()
    );
    assert!(
        text(&html, ".chart-panel .missing")
            .iter()
            .any(|value| value.contains("Brak zapisanych snapshotów"))
    );
}

#[test]
fn score_bars_use_saved_company_values_only_preserving_null_zero_and_date_gaps() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_json(root, "history.json", &ohlc_history("KGHM"));
    saved_ranking(
        root,
        "positive",
        "2026-09-10",
        "2026-09-10T12:00:00Z",
        json!(3),
    );
    saved_ranking(
        root,
        "negative",
        "2026-09-13",
        "2026-09-13T12:00:00Z",
        json!(-2),
    );
    saved_ranking(root, "zero", "2026-09-16", "2026-09-16T12:00:00Z", json!(0));
    saved_ranking(
        root,
        "early",
        "2026-09-18",
        "2026-09-18T08:00:00Z",
        json!(5),
    );
    saved_ranking(
        root,
        "late",
        "2026-09-18",
        "2026-09-18T12:00:00Z",
        Value::Null,
    );
    let company = card(root, "KGHM");
    assert_eq!(company.saved_scores.len(), 4);
    assert!(company.saved_scores.last().unwrap().score.is_none());
    let html = ParsedHtml::parse_document(&render_company(&company, as_of()));
    assert_eq!(
        text(&html, ".score-chart .score-bar title"),
        ["2026-09-10: 3", "2026-09-13: -2"]
    );
    assert_eq!(
        text(&html, ".score-chart .score-zero title"),
        ["2026-09-16: 0"]
    );
    let marks: Vec<_> = html
        .select(&Selector::parse(".score-chart [data-score]").unwrap())
        .collect();
    assert_eq!(marks.len(), 3);
    let positions: Vec<f64> = marks
        .iter()
        .map(|mark| {
            mark.value()
                .attr("x")
                .or_else(|| mark.value().attr("x1"))
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    assert!((positions[1] - positions[0] - (positions[2] - positions[1])).abs() < 0.02);
    assert!(
        html.select(&Selector::parse(".score-chart [data-date='2026-09-18']").unwrap())
            .next()
            .is_none()
    );
    assert_eq!(text(&html, ".chart-panel svg").len(), 1);
    assert!(
        marks
            .iter()
            .all(|mark| mark.value().attr("data-date").unwrap() >= "2026-09-10")
    );
    assert!(
        !text(&html, ".score-chart title")
            .iter()
            .any(|s| s.contains(": 9"))
    );
}

#[test]
fn company_loading_is_independent_of_other_company_archives_and_matches_ranking() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_json(
        root,
        "2026-09-19_KGHM_fundamentals/fundamentals.json",
        &report("KGHM"),
    );
    let mut history = ohlc_history("KGHM");
    history["candles"].as_array_mut().unwrap().extend([
        json!({"date":"2025-08-29", "open":100, "high":101, "low":99, "close":100}),
        json!({"date":"2025-09-30", "open":100, "high":104, "low":99, "close":103}),
    ]);
    write_json(root, "2026-09-19_KGHM_history/history.json", &history);
    write_json(
        root,
        "2026-09-19_PGE_fundamentals/fundamentals.json",
        &report("PGE"),
    );
    write_json(
        root,
        "2026-09-19_PGE_history/history.json",
        &ohlc_history("PGE"),
    );
    for (name, date, percent) in [
        ("older", "2026-09-18T12:00:00Z", "1.5"),
        ("newer", "2026-09-19T12:00:00Z", "1.2"),
    ] {
        write_json(
            root,
            &format!("{name}/shorts.json"),
            &json!({
                "code":"KGHM", "isin":"PLKGHM000017", "fetched_at":date,
                "positions":[{"holder":"Holder", "net_short_position_percent":percent,
                    "position_date":"2026-09-17", "modify_date":"2026-09-18"}]
            }),
        );
    }
    let expected = rank_directory(root, as_of())
        .unwrap()
        .rows
        .into_iter()
        .find(|row| row.code == "KGHM")
        .unwrap();
    assert_eq!(expected.final_score, Some(5));
    let index = ArchiveIndex::read(root).unwrap();
    assert_eq!(index.company("KGHM").unwrap().histories.len(), 1);
    // The indexed card needs no walk of the collection directory and no foreign sources.
    fs::remove_dir_all(root.join("2026-09-19_PGE_fundamentals")).unwrap();
    fs::remove_dir_all(root.join("2026-09-19_PGE_history")).unwrap();
    let company = scorecard::load(&index, "KGHM", as_of()).unwrap().unwrap();
    assert_eq!(
        serde_json::to_value(company.row).unwrap(),
        serde_json::to_value(expected).unwrap()
    );
}

#[test]
fn candles_and_scores_share_one_svg_and_the_same_calendar_positions() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    write_json(root, "history.json", &ohlc_history("KGHM"));
    saved_ranking(
        root,
        "friday",
        "2026-09-18",
        "2026-09-18T12:00:00Z",
        json!(3),
    );
    // A saved Saturday ranking has its own calendar position even without a candle.
    saved_ranking(
        root,
        "saturday",
        "2026-09-19",
        "2026-09-19T12:00:00Z",
        json!(-1),
    );
    let html = ParsedHtml::parse_document(&render_company(&card(root, "KGHM"), as_of()));
    assert_eq!(text(&html, ".chart-panel svg").len(), 1);
    assert_eq!(text(&html, ".price-score-chart .candle").len(), 2);
    assert_eq!(text(&html, ".price-score-chart .score-bar").len(), 2);
    let candle = html
        .select(&Selector::parse(".candle[data-date='2026-09-18']").unwrap())
        .next()
        .unwrap();
    let friday = html
        .select(&Selector::parse(".score-bar[data-date='2026-09-18']").unwrap())
        .next()
        .unwrap();
    let saturday = html
        .select(&Selector::parse(".score-bar[data-date='2026-09-19']").unwrap())
        .next()
        .unwrap();
    assert_eq!(candle.value().attr("data-x"), friday.value().attr("data-x"));
    let x = |element: scraper::ElementRef<'_>| {
        element
            .value()
            .attr("data-x")
            .unwrap()
            .parse::<f64>()
            .unwrap()
    };
    assert!((x(saturday) - x(friday) - 6.0).abs() < 0.01);
    for mark in [friday, saturday] {
        assert!(mark.value().attr("y").unwrap().parse::<f64>().unwrap() >= 350.0);
    }
    assert!(
        text(&html, ".price-score-chart .time-axis .axis-label")
            .iter()
            .any(|s| s.starts_with("2020"))
    );
}

#[test]
fn missing_prices_show_an_explanation_while_saved_scores_remain_visible() {
    let directory = tempfile::tempdir().unwrap();
    saved_ranking(
        directory.path(),
        "saved",
        "2026-09-18",
        "2026-09-18T12:00:00Z",
        json!(2),
    );
    let html =
        ParsedHtml::parse_document(&render_company(&card(directory.path(), "KGHM"), as_of()));
    assert!(text(&html, ".candle").is_empty());
    assert_eq!(
        text(&html, ".price-score-chart .score-bar title"),
        ["2026-09-18: 2"]
    );
    assert!(
        text(&html, ".chart-panel .missing")
            .iter()
            .any(|s| s == "Brak zapisanej historii OHLC.")
    );
}

#[test]
fn short_chart_keeps_every_reading_in_time_order_including_empty_registers() {
    for newest_empty in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_json(root, "history.json", &ohlc_history("KGHM"));
        let positions = json!([
            {"holder":"Holder A", "net_short_position_percent":"0.6", "position_date":"2026-09-18", "modify_date":"2026-09-18"},
            {"holder":"Holder B", "net_short_position_percent":"0.7", "position_date":"2026-09-18", "modify_date":"2026-09-18"}
        ]);
        // Same UTC day: both observations belong on the chart, without daily reduction.
        for (path, date, empty) in [
            ("a/shorts.json", "2026-09-19T12:00:00Z", newest_empty),
            ("z/shorts.json", "2026-09-19T08:00:00Z", !newest_empty),
        ] {
            write_json(
                root,
                path,
                &json!({"code":"KGHM", "isin":"PLKGHM000017", "fetched_at":date,
                "positions":if empty { json!([]) } else { positions.clone() }}),
            );
        }
        let company = card(root, "KGHM");
        assert_eq!(company.short_totals.len(), 2);
        let html = ParsedHtml::parse_document(&render_company(&company, as_of()));
        let points: Vec<_> = html
            .select(&Selector::parse(".shorts-chart .short-point").unwrap())
            .collect();
        assert_eq!(points.len(), 2);
        assert_eq!(
            points
                .iter()
                .map(|p| p.value().attr("data-total").unwrap())
                .collect::<Vec<_>>(),
            if newest_empty {
                vec!["1.3", "0"]
            } else {
                vec!["0", "1.3"]
            }
        );
        assert_eq!(
            points[0].value().attr("data-fetched-at"),
            Some("2026-09-19T08:00:00+00:00")
        );
        assert_eq!(
            points[1].value().attr("data-fetched-at"),
            Some("2026-09-19T12:00:00+00:00")
        );
        assert!(
            points[0]
                .value()
                .attr("cx")
                .unwrap()
                .parse::<f64>()
                .unwrap()
                < points[1]
                    .value()
                    .attr("cx")
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
        );
        assert_eq!(
            text(&html, ".holders tbody tr").len(),
            if newest_empty { 0 } else { 2 }
        );
        assert_eq!(text(&html, ".shorts-chart .short-line").len(), 1);
        assert!(text(&html, ".price-score-chart .short-point").is_empty());
        assert_eq!(text(&html, "svg").len(), 2);
    }
}

#[test]
fn company_without_short_files_has_a_message_without_a_zero_chart() {
    let directory = tempfile::tempdir().unwrap();
    write_json(directory.path(), "history.json", &ohlc_history("KGHM"));
    let company = card(directory.path(), "KGHM");
    assert!(company.short_totals.is_empty());
    let html = ParsedHtml::parse_document(&render_company(&company, as_of()));
    assert!(text(&html, ".shorts-chart").is_empty());
    assert!(text(&html, ".short-point").is_empty());
    assert!(text(&html, ".short-line").is_empty());
    assert!(text(&html, ".holders tbody tr").is_empty());
    assert!(
        text(&html, ".missing")
            .iter()
            .any(|s| s == "brak zapisanych odczytów rejestru")
    );
}
