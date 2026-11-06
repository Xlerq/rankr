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
    scorecard::load(root, code, as_of()).unwrap().unwrap()
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
    let app = router(directory.path().to_path_buf());
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
