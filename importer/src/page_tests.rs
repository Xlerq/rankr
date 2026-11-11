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

fn history_page(card: &CompanyCard, as_of: NaiveDate) -> String {
    render_chart(card, as_of, &[card.row.code.clone()])
}

fn board_page(root: &FsPath) -> String {
    let index = ArchiveIndex::read(root).unwrap();
    render_shorts(
        &crate::weekly_shorts::board(&index, &["KGHM".into()]),
        as_of(),
    )
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
        ["1", "−1", "0", ""]
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
    assert!(text(&html, ".missing").is_empty());
    assert!(text(&html, "svg").is_empty());
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
        text(&html, ".metric dt")
            .iter()
            .any(|s| s.contains("PLN × 1000"))
    );
}

#[test]
fn sentiment_details_use_the_rankings_pair_without_a_holder_list() {
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
    assert!(text(&html, ".holders").is_empty());
    assert!(text(&html, "svg").is_empty());
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
        ["—"]
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
        ("/KGHM", StatusCode::OK, "Karta wyników"),
        ("/KGHM/wykres", StatusCode::OK, "Historia punktów"),
        ("/historia?spolka=KGHM", StatusCode::OK, "Historia punktów"),
        ("/historia", StatusCode::OK, "Historia punktów"),
        ("/sezonowosc?spolka=KGHM", StatusCode::OK, "Sezonowość"),
        ("/sezonowosc", StatusCode::OK, "Sezonowość"),
        ("/shorty", StatusCode::OK, "Shorty KNF"),
        (
            "/historia?spolka=UNKNOWN",
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki",
        ),
        (
            "/sezonowosc?spolka=UNKNOWN",
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki",
        ),
        ("/KGHM/shorty", StatusCode::OK, "Shorty"),
        (
            "/UNKNOWN/wykres",
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki",
        ),
        (
            "/UNKNOWN/shorty",
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki",
        ),
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
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    for (old, target) in [
        ("/KGHM/wykres", "/historia?spolka=KGHM"),
        ("/KGHM/shorty", "/shorty"),
    ] {
        let response = client
            .get(format!("http://{address}{old}"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(response.headers()["location"], target);
    }
    let response = client
        .get(format!("http://{address}/historia?spolka=PGE"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let html = ParsedHtml::parse_document(&response.text().await.unwrap());
    assert_eq!(text(&html, "select option[selected]"), ["PGE"]);
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
    let html = ParsedHtml::parse_document(&history_page(&company, as_of()));
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
    assert!(text(&html, ".chart-panel .missing").is_empty());
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
    let html = ParsedHtml::parse_document(&history_page(&company, as_of()));
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
        .map(|mark| mark.value().attr("data-x").unwrap().parse().unwrap())
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
    let html = ParsedHtml::parse_document(&history_page(&card(root, "KGHM"), as_of()));
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
    assert!(x(saturday) > x(friday));
    for mark in [friday, saturday] {
        assert!(mark.value().attr("y").unwrap().parse::<f64>().unwrap() >= 640.0);
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
    let html = ParsedHtml::parse_document(&history_page(&card(directory.path(), "KGHM"), as_of()));
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
fn weekly_shorts_select_the_latest_reading_of_each_iso_week_including_zero() {
    for newest_empty in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        write_json(root, "history.json", &ohlc_history("KGHM"));
        for (path, date, total) in [
            (
                "a/shorts.json",
                "2026-09-18T12:00:00Z",
                if newest_empty { None } else { Some("1.3") },
            ),
            ("z/shorts.json", "2026-09-14T08:00:00Z", Some("9.9")),
            ("prior/shorts.json", "2026-09-08T08:00:00Z", Some("1.5")),
        ] {
            let positions = total
                .map(|total| {
                    json!([{"holder":"Holder", "net_short_position_percent":total,
                "position_date":"2026-09-07", "modify_date":"2026-09-07"}])
                })
                .unwrap_or(json!([]));
            write_json(
                root,
                path,
                &json!({"code":"KGHM", "isin":"PLKGHM000017", "fetched_at":date, "positions":positions}),
            );
        }
        let html = ParsedHtml::parse_document(&board_page(directory.path()));
        assert_eq!(text(&html, ".knf-board tbody tr").len(), 1);
        assert_eq!(
            text(&html, ".knf-board thead .weekly-heading"),
            ["2026-W37", "2026-W38"]
        );
        assert_eq!(text(&html, ".knf-board tbody th"), ["KGHM"]);
        assert_eq!(
            text(&html, ".knf-board tbody .weekly-total"),
            if newest_empty {
                vec!["1.5", "0"]
            } else {
                vec!["1.5", "1.3"]
            }
        );
        assert!(text(&html, "svg").is_empty());
    }
}

#[test]
fn company_without_short_files_has_a_message_without_a_zero_chart() {
    let directory = tempfile::tempdir().unwrap();
    write_json(directory.path(), "history.json", &ohlc_history("KGHM"));
    let html = ParsedHtml::parse_document(&board_page(directory.path()));
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

#[test]
fn ranking_is_numeric_and_has_only_the_requested_columns() {
    let rows = [RankingRow {
        code: "KGHM".into(),
        gics_sector: "Materials".into(),
        fundamental_score: Some(1),
        technical_score: Some(-1),
        sentiment_score: Some(0),
        final_score: None,
    }];
    let rendered = render(&rows, as_of());
    let html = ParsedHtml::parse_document(&rendered);
    assert_eq!(
        text(&html, "thead th"),
        [
            "Spółka",
            "Sektor",
            "Fundament",
            "Technika",
            "Sentyment",
            "Wynik"
        ]
    );
    for arrow in ['↑', '↓'] {
        assert!(!rendered.contains(arrow));
    }
    assert!(!rendered.contains("Wagi rodzin"));
    assert!(text(&html, "footer, .legend, .source-note").is_empty());
    assert!(text(&html, ".date").is_empty());
}

#[test]
fn scorecard_has_points_and_navigation_without_charts_or_explanations() {
    let directory = tempfile::tempdir().unwrap();
    write_json(directory.path(), "fundamentals.json", &report("KGHM"));
    write_json(directory.path(), "history.json", &ohlc_history("KGHM"));
    let html =
        ParsedHtml::parse_document(&render_company(&card(directory.path(), "KGHM"), as_of()));
    assert!(text(&html, ".arrow").contains(&"↑".into()));
    assert!(
        text(
            &html,
            "svg, .holders, .missing, .source-note, footer, .legend"
        )
        .is_empty()
    );
    let links: Vec<_> = html
        .select(&Selector::parse(".view-links a").unwrap())
        .map(|link| {
            (
                link.text().collect::<String>(),
                link.value().attr("href").unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        links,
        [
            ("Sezonowość".into(), "/sezonowosc?spolka=KGHM".into()),
            ("Historia punktów".into(), "/historia?spolka=KGHM".into()),
            ("Shorty KNF".into(), "/shorty".into())
        ]
    );
    assert_eq!(
        text(&html, "[data-metric=net_margin] dd"),
        ["100", "1000", "10%"]
    );
}

#[test]
fn charts_fill_the_container_and_keep_a_tall_pixel_height() {
    let directory = tempfile::tempdir().unwrap();
    write_json(directory.path(), "history.json", &ohlc_history("KGHM"));
    let html = ParsedHtml::parse_document(&history_page(&card(directory.path(), "KGHM"), as_of()));
    let svgs: Vec<_> = html.select(&Selector::parse("svg").unwrap()).collect();
    assert_eq!(svgs.len(), 1);
    assert_eq!(svgs[0].value().attr("width"), Some("100%"));
    assert_eq!(svgs[0].value().attr("height"), Some("900"));
    assert!(svgs[0].value().attr("viewBox").is_none());
    assert!(!include_str!("page.css").contains("max-width: 1200px"));
    assert!(text(&html, ".source-note, footer, .legend, .missing").is_empty());
}

#[test]
fn weekly_grouping_handles_iso_year_boundaries_and_missing_weeks() {
    use crate::charts::ShortTotal;
    let observation = |at: &str, total| ShortTotal {
        fetched_at: at.parse().unwrap(),
        total: Decimal::from(total),
    };
    let readings = [
        observation("2021-01-18T10:00:00Z", 6),
        observation("2021-01-04T10:00:00Z", 4),
        observation("2020-12-31T10:00:00Z", 2),
        observation("2021-01-01T10:00:00Z", 3),
    ];
    let weeks = crate::weekly_shorts::group(&readings);
    assert_eq!(
        weeks
            .iter()
            .map(|week| week.week.to_string())
            .collect::<Vec<_>>(),
        ["2020-12-28", "2021-01-04", "2021-01-18"]
    );
    assert_eq!(weeks[0].total, Decimal::from(3));
    assert_eq!(weeks[1].total, Decimal::from(4));
    assert_eq!(weeks[2].total, Decimal::from(6));
}

fn seasonal_history(years: i32) -> Value {
    let mut candles = Vec::new();
    for year in (2026 - years)..2026 {
        candles.extend([
            json!({"date":format!("{year}-08-31"), "close":100}),
            json!({"date":format!("{year}-09-02"), "close":100}),
            json!({"date":format!("{year}-09-28"), "close":if year == 2026 - years { 120 } else { 110 }}),
        ]);
    }
    candles.extend([
        json!({"date":"2026-09-01", "close":100}),
        json!({"date":"2026-09-18", "close":105}),
        json!({"date":"2026-09-30", "close":1000}),
    ]);
    json!({"instrument":{"code":"KGHM"}, "fetched_at":"2026-09-19T10:00:00Z", "candles":candles})
}

#[test]
fn six_years_show_the_full_mean_and_five_year_path_without_ten_years() {
    let directory = tempfile::tempdir().unwrap();
    write_json(directory.path(), "history.json", &seasonal_history(6));
    let company = card(directory.path(), "KGHM");
    let data = crate::seasonality::calculate(&company.closes, as_of()).unwrap();
    assert_eq!(data.years, 6);
    assert_eq!(data.mean, company.technical.as_ref().unwrap().explanation.r);
    assert_eq!(data.current, [Decimal::ZERO, Decimal::new(5, 2)]);
    assert_eq!(
        data.five.as_ref().unwrap(),
        &[Decimal::ZERO, Decimal::new(1, 1)]
    );
    assert!(data.ten.is_none());
    let html = ParsedHtml::parse_document(
        &render_seasonality(&company, as_of(), &["KGHM".into(), "PGE".into()]).unwrap(),
    );
    assert_eq!(text(&html, ".season-summary strong"), ["11.6667%"]);
    assert!(text(&html, ".season-summary h2")[0].contains("Wrzesień 2026"));
    for kind in ["current", "all", "five"] {
        assert_eq!(text(&html, &format!("svg [data-series={kind}]")).len(), 1);
    }
    assert!(text(&html, "svg [data-series=ten]").is_empty());
    assert_eq!(text(&html, ".seasonality-chart").len(), 1);
    assert!(text(&html, "svg")[0].contains("Sesja miesiąca"));
    assert_eq!(text(&html, "select option[selected]"), ["KGHM"]);
    assert!(text(&html, "footer, .legend, .price-score-chart").is_empty());
}

#[test]
fn seasonal_path_thresholds_use_completed_years_only_and_start_at_zero() {
    for years in [0, 4, 5, 9, 10] {
        let history: crate::technical::TechnicalHistory =
            serde_json::from_value(seasonal_history(years)).unwrap();
        let data = crate::seasonality::calculate(&history.candles, as_of()).unwrap();
        assert_eq!(data.years, years as usize);
        assert_eq!(data.five.is_some(), years >= 5);
        assert_eq!(data.ten.is_some(), years >= 10);
        assert_eq!(data.current.first(), Some(&Decimal::ZERO));
        if years > 0 {
            assert_eq!(data.all.first(), Some(&Decimal::ZERO));
        } else {
            assert!(data.all.is_empty());
            assert!(data.mean.is_none());
        }
    }
}

#[test]
fn seasonal_mean_preserves_the_scorers_adjusted_price_and_skips_missing_previous_months() {
    let history: crate::technical::TechnicalHistory = serde_json::from_value(json!({"candles":[
        {"date":"2024-09-02", "close":100}, {"date":"2024-09-30", "close":500},
        {"date":"2025-08-29", "close":100, "adjusted_close":100},
        {"date":"2025-09-02", "close":100, "adjusted_close":100},
        {"date":"2025-09-30", "close":80, "adjusted_close":110}
    ]}))
    .unwrap();
    let data = crate::seasonality::calculate(&history.candles, as_of()).unwrap();
    assert_eq!(data.mean, Some(Decimal::new(1, 1)));
    assert_eq!(data.years, 1);
    assert_eq!(data.all, [Decimal::ZERO, Decimal::new(1, 1)]);
    assert!(data.current.is_empty());
}

#[test]
fn seasonality_in_january_uses_december_of_the_preceding_year() {
    let history: crate::technical::TechnicalHistory = serde_json::from_value(json!({"candles":[
        {"date":"2024-12-31", "close":100}, {"date":"2025-01-02", "close":100},
        {"date":"2025-01-31", "close":110}, {"date":"2026-01-02", "close":200}
    ]}))
    .unwrap();
    let data = crate::seasonality::calculate(
        &history.candles,
        NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
    )
    .unwrap();
    assert_eq!(data.mean, Some(Decimal::new(1, 1)));
    assert_eq!(data.years, 1);
    assert_eq!(data.current, [Decimal::ZERO]);
}

#[test]
fn knf_board_has_multiple_companies_and_keeps_missing_observations_blank() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for (code, name, at, value) in [
        ("KGHM", "old", "2026-09-08T12:00:00Z", Some("1.5")),
        ("KGHM", "new", "2026-09-18T12:00:00Z", None),
        ("PGE", "old", "2026-09-08T12:00:00Z", Some("0.7")),
    ] {
        let positions = value
            .map(|value| {
                json!([{"holder":"Holder", "net_short_position_percent":value,
            "position_date":"2026-09-07", "modify_date":"2026-09-07"}])
            })
            .unwrap_or(json!([]));
        write_json(
            root,
            &format!("{code}/{name}/shorts.json"),
            &json!({"code":code, "isin":format!("fixture-{code}"), "fetched_at":at, "positions":positions}),
        );
    }
    let index = ArchiveIndex::read(root).unwrap();
    let board = crate::weekly_shorts::board(&index, &["KGHM".into(), "PGE".into(), "PKOBP".into()]);
    let html = ParsedHtml::parse_document(&render_shorts(&board, as_of()));
    assert_eq!(text(&html, ".knf-board tbody th"), ["KGHM", "PGE", "PKOBP"]);
    assert_eq!(
        text(&html, ".knf-board thead .weekly-heading"),
        ["2026-W37", "2026-W38"]
    );
    assert_eq!(
        text(&html, ".knf-board tbody .weekly-total"),
        ["1.5", "0", "0.7", "", "", ""]
    );
    assert!(text(&html, "svg, .holders").is_empty());
}

#[test]
fn ranking_has_three_analysis_buttons_above_the_numeric_table() {
    let rendered = render(&[], as_of());
    let html = ParsedHtml::parse_document(&rendered);
    let buttons: Vec<_> = html
        .select(&Selector::parse(".page-buttons a").unwrap())
        .map(|a| {
            (
                a.text().collect::<String>(),
                a.value().attr("href").unwrap(),
            )
        })
        .collect();
    assert_eq!(
        buttons,
        [
            ("Sezonowość".into(), "/sezonowosc"),
            ("Historia punktów".into(), "/historia"),
            ("Shorty KNF".into(), "/shorty")
        ]
    );
    assert!(
        rendered.find("page-buttons\" aria-label").unwrap()
            < rendered.find("<table class=\"ranking\"").unwrap()
    );
    assert!(text(&html, ".arrow, .legend, footer").is_empty());
    assert!(!rendered.contains("Wagi rodzin"));
}

#[test]
fn a_known_company_without_archives_has_one_short_seasonality_message() {
    let directory = tempfile::tempdir().unwrap();
    let company = card(directory.path(), "KGHM");
    let html = ParsedHtml::parse_document(
        &render_seasonality(&company, as_of(), &["KGHM".into()]).unwrap(),
    );
    assert_eq!(text(&html, ".missing"), ["Brak historii cen."]);
    assert!(text(&html, "svg, .season-summary").is_empty());
}

#[test]
fn knf_daily_points_show_the_actual_pair_even_when_weekly_changes_have_another_direction() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    for (code, observations) in [
        ("KRUK", ["2.34", "2.30", "2.81", "2.75"]),
        ("ALLEGRO", ["0.76", "0.66", "0.66", "0.66"]),
    ] {
        for (index, (at, total)) in [
            "2026-11-06T10:00:00Z",
            "2026-11-09T19:00:00Z",
            "2026-11-10T20:00:00Z",
            "2026-11-11T16:00:00Z",
        ]
        .into_iter()
        .zip(observations)
        .enumerate()
        {
            write_json(
                root,
                &format!("{code}/{index}/shorts.json"),
                &json!({
                    "code":code, "isin":format!("fixture-{code}"), "fetched_at":at,
                    "positions":[{"holder":"Holder", "net_short_position_percent":total, "position_date":"2026-11-05", "modify_date":"2026-11-06"}]
                }),
            );
        }
    }
    let index = ArchiveIndex::read(root).unwrap();
    let board = crate::weekly_shorts::board(&index, &["KRUK".into(), "ALLEGRO".into()]);
    let date = NaiveDate::from_ymd_opt(2026, 11, 11).unwrap();
    let ranking = rank_directory(root, date).unwrap();
    for row in &board.rows {
        assert_eq!(
            row.sentiment_score,
            ranking
                .rows
                .iter()
                .find(|ranked| ranked.code == row.code)
                .unwrap()
                .sentiment_score
        );
    }
    let html = ParsedHtml::parse_document(&render_shorts(&board, date));
    assert_eq!(text(&html, "tr[data-code=KRUK] .sentiment-point"), ["1"]);
    assert_eq!(
        text(&html, "tr[data-code=KRUK] .sentiment-change"),
        ["-0.06"]
    );
    assert_eq!(
        text(&html, "tr[data-code=KRUK] .reading-value"),
        ["2.81%", "2.75%"]
    );
    assert_eq!(
        text(&html, "tr[data-code=KRUK] time"),
        ["2026-11-10 20:00", "2026-11-11 16:00"]
    );
    assert_eq!(
        text(&html, "tr[data-code=KRUK] .weekly-total"),
        ["2.34", "2.75"]
    );
    assert_eq!(text(&html, "tr[data-code=ALLEGRO] .sentiment-point"), ["0"]);
    assert_eq!(
        text(&html, "tr[data-code=ALLEGRO] .sentiment-change"),
        ["0"]
    );
    assert_eq!(
        text(&html, "tr[data-code=ALLEGRO] .weekly-total"),
        ["0.76", "0.66"]
    );
    let weekly = html
        .select(&Selector::parse("tr[data-code=KRUK] .weekly-total").unwrap())
        .last()
        .unwrap();
    assert_eq!(
        weekly.value().attr("title"),
        Some("Ostatni odczyt: 2026-11-11 16:00 UTC")
    );
    let explanation = text(&html, ".knf-guide").join(" ");
    for fragment in [
        "procentach",
        "wcześniejszego dnia UTC",
        "Spadek sumy daje +1",
        "tydzień ISO",
        "bieżący tydzień jest niepełny",
        "Puste pole oznacza brak zapisu",
    ] {
        assert!(explanation.contains(fragment), "{fragment}");
    }
    assert!(text(&html, "svg").is_empty());
}

#[test]
fn missing_comparison_does_not_become_a_zero_point_on_the_knf_board() {
    let directory = tempfile::tempdir().unwrap();
    write_json(
        directory.path(),
        "shorts.json",
        &json!({
            "code":"KGHM", "isin":"PLKGHM000017", "fetched_at":"2026-09-19T12:00:00Z", "positions":[]
        }),
    );
    let html = ParsedHtml::parse_document(&board_page(directory.path()));
    assert_eq!(text(&html, ".sentiment-point"), [""]);
    assert_eq!(text(&html, ".sentiment-change"), [""]);
    assert_eq!(text(&html, ".reading-value"), ["0%"]);
    assert_eq!(text(&html, ".weekly-total"), ["0"]);
}

#[test]
fn the_top_right_market_badge_and_ranking_date_are_removed_from_every_page() {
    let directory = tempfile::tempdir().unwrap();
    let company = card(directory.path(), "KGHM");
    for rendered in [
        render(&[], as_of()),
        render_company(&company, as_of()),
        history_page(&company, as_of()),
        render_seasonality(&company, as_of(), &["KGHM".into()]).unwrap(),
        board_page(directory.path()),
    ] {
        assert!(!rendered.contains("GPW / WIG20"));
        assert!(!rendered.contains("Data rankingu"));
        assert!(text(&ParsedHtml::parse_document(&rendered), ".date").is_empty());
    }
}
