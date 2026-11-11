//! Local, server-rendered ranking and company scorecard.

use std::{fmt::Display, path::PathBuf, sync::Arc};

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::get,
};
use chrono::{Datelike, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{
    company_archive::ArchiveIndex,
    model::ShortsSnapshot,
    ranking::{RankingRow, rank_directory},
    scorecard::{self, CompanyCard},
};

#[derive(Clone)]
struct PageState {
    directory: PathBuf,
    archives: Arc<ArchiveIndex>,
    company_codes: Arc<Vec<String>>,
}

fn router(directory: PathBuf) -> std::io::Result<Router> {
    let archives = Arc::new(ArchiveIndex::read(&directory)?);
    let company_codes = Arc::new(
        crate::fundamental_points_table::gics_map()
            .map_err(std::io::Error::other)?
            .into_keys()
            .collect(),
    );
    Ok(Router::new()
        .route("/", get(index))
        .route("/{code}", get(company))
        .route("/sezonowosc", get(seasonality))
        .route("/historia", get(history))
        .route("/shorty", get(shorts))
        .route("/{code}/wykres", get(legacy_chart))
        .route("/{code}/shorty", get(legacy_shorts))
        .with_state(PageState {
            directory,
            archives,
            company_codes,
        }))
}

pub async fn serve(directory: PathBuf) -> std::io::Result<()> {
    let app = tokio::task::spawn_blocking(move || router(directory)).await??;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("rankr: http://127.0.0.1:3000/");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(State(state): State<PageState>) -> Result<Html<String>, (StatusCode, &'static str)> {
    let as_of = Utc::now().date_naive();
    let ranking = tokio::task::spawn_blocking(move || rank_directory(&state.directory, as_of))
        .await
        .map_err(server_error)?
        .map_err(server_error)?;
    for skipped in &ranking.skipped {
        eprintln!(
            "Skipped {}: {}",
            skipped.source_file.display(),
            skipped.reason
        );
    }
    Ok(Html(render(&ranking.rows, as_of)))
}

async fn company_data(
    state: PageState,
    code: String,
) -> Result<(CompanyCard, NaiveDate), (StatusCode, &'static str)> {
    let as_of = Utc::now().date_naive();
    let card = tokio::task::spawn_blocking(move || scorecard::load(&state.archives, &code, as_of))
        .await
        .map_err(server_error)?
        .map_err(server_error)?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki o podanym kodzie.",
        ))?;
    Ok((card, as_of))
}

async fn company(
    State(state): State<PageState>,
    Path(code): Path<String>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let (card, as_of) = company_data(state, code).await?;
    Ok(Html(render_company(&card, as_of)))
}

#[derive(Deserialize)]
struct CompanyQuery {
    spolka: Option<String>,
}

fn selected_code(
    state: &PageState,
    query: CompanyQuery,
) -> Result<String, (StatusCode, &'static str)> {
    let code = query
        .spolka
        .filter(|code| !code.trim().is_empty())
        .map(|code| code.trim().to_ascii_uppercase())
        .or_else(|| {
            state
                .company_codes
                .iter()
                .find(|code| {
                    state
                        .archives
                        .company(code)
                        .is_some_and(|files| !files.histories.is_empty())
                })
                .cloned()
        })
        .or_else(|| state.company_codes.first().cloned())
        .ok_or_else(|| server_error("Brak mapy spółek"))?;
    if !state.company_codes.contains(&code) {
        return Err((
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki o podanym kodzie.",
        ));
    }
    Ok(code)
}

async fn history(
    State(state): State<PageState>,
    Query(query): Query<CompanyQuery>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let code = selected_code(&state, query)?;
    let codes = state.company_codes.clone();
    let (card, as_of) = company_data(state, code).await?;
    Ok(Html(render_chart(&card, as_of, &codes)))
}

async fn seasonality(
    State(state): State<PageState>,
    Query(query): Query<CompanyQuery>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let code = selected_code(&state, query)?;
    let codes = state.company_codes.clone();
    let (card, as_of) = company_data(state, code).await?;
    Ok(Html(
        render_seasonality(&card, as_of, &codes).map_err(server_error)?,
    ))
}

async fn shorts(
    State(state): State<PageState>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let board = tokio::task::spawn_blocking(move || {
        crate::weekly_shorts::board(&state.archives, &state.company_codes)
    })
    .await
    .map_err(server_error)?;
    Ok(Html(render_shorts(&board, Utc::now().date_naive())))
}

async fn legacy_chart(
    State(state): State<PageState>,
    Path(code): Path<String>,
) -> Result<Redirect, (StatusCode, &'static str)> {
    let code = selected_code(&state, CompanyQuery { spolka: Some(code) })?;
    Ok(Redirect::permanent(&format!("/historia?spolka={code}")))
}

async fn legacy_shorts(
    State(state): State<PageState>,
    Path(code): Path<String>,
) -> Result<Redirect, (StatusCode, &'static str)> {
    selected_code(&state, CompanyQuery { spolka: Some(code) })?;
    Ok(Redirect::permanent("/shorty"))
}

fn server_error(error: impl Display) -> (StatusCode, &'static str) {
    eprintln!("Ranking error: {error}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Nie udało się odczytać danych rankingu.",
    )
}

fn document(title: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="pl"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{} — rankr</title><style>{}</style></head>
<body><main><nav><a class="brand" href="/">rankr<span>.</span></a></nav>
{body}
</main></body></html>"#,
        escape(title),
        include_str!("page.css")
    )
}

fn render(rows: &[RankingRow], _as_of: NaiveDate) -> String {
    let mut body = String::new();
    for row in rows {
        body.push_str(&format!(
            r#"<tr><th scope="row"><a class="row-link company-name" href="/{}">{}</a></th>
<td class="sector">{}</td><td>{}</td><td>{}</td><td>{}</td><td class="final-score">{}</td></tr>"#,
            escape(&row.code),
            escape(&row.code),
            escape(&row.gics_sector),
            score(row.fundamental_score, false),
            score(row.technical_score, false),
            score(row.sentiment_score, false),
            score(row.final_score, false)
        ));
    }
    if rows.is_empty() {
        body.push_str(
            "<tr><td colspan=\"6\" class=\"empty\">Brak zapisanych danych spółek.</td></tr>",
        );
    }
    document(
        "Ranking spółek GPW",
        &format!(
            r#"
<header class="page-heading"><h1>Ranking spółek</h1></header>
{buttons}<div class="table-panel"><table class="ranking"><thead><tr>
<th scope="col">Spółka</th><th scope="col">Sektor</th><th scope="col">Fundament</th><th scope="col">Technika</th><th scope="col">Sentyment</th><th scope="col">Wynik</th>
</tr></thead><tbody>{body}</tbody></table></div>"#,
            buttons = global_links("")
        ),
    )
}

fn company_header(card: &CompanyCard, _as_of: NaiveDate, view: &str) -> String {
    format!(
        r#"<a class="back" href="/">Ranking</a>
<header class="page-heading company-heading"><div><p class="eyebrow">{view}</p><h1 class="company-name">{}</h1><p class="sector">{}</p></div>
</header>"#,
        escape(&card.row.code),
        escape(&card.row.gics_sector)
    )
}

fn global_links(current: &str) -> String {
    let mut links = String::from("<nav class=\"page-buttons\" aria-label=\"Analizy\">");
    for (view, path) in [
        ("Sezonowość", "/sezonowosc"),
        ("Historia punktów", "/historia"),
        ("Shorty KNF", "/shorty"),
    ] {
        let active = if current == path {
            " aria-current=\"page\""
        } else {
            ""
        };
        links.push_str(&format!("<a href=\"{path}\"{active}>{view}</a>"));
    }
    links.push_str("</nav>");
    links
}

fn view_links(code: &str) -> String {
    let code = escape(code);
    format!(
        "<nav class=\"view-links\" aria-label=\"Analizy spółki\"><a href=\"/sezonowosc?spolka={code}\">Sezonowość</a><a href=\"/historia?spolka={code}\">Historia punktów</a><a href=\"/shorty\">Shorty KNF</a></nav>"
    )
}

fn page_heading(title: &str, _as_of: NaiveDate) -> String {
    format!(
        "<a class=\"back\" href=\"/\">Ranking</a><header class=\"page-heading\"><h1>{title}</h1></header>"
    )
}

fn company_selector(action: &str, selected: &str, codes: &[String]) -> String {
    let mut html = format!(
        "<form class=\"company-selector\" action=\"{action}\" method=\"get\"><label for=\"company\">Spółka</label><select id=\"company\" name=\"spolka\">"
    );
    for code in codes {
        let option = if code == selected { " selected" } else { "" };
        html.push_str(&format!(
            "<option value=\"{}\"{option}>{}</option>",
            escape(code),
            escape(code)
        ));
    }
    html.push_str("</select><button type=\"submit\">Pokaż</button></form>");
    html
}

fn render_company(card: &CompanyCard, as_of: NaiveDate) -> String {
    let mut body = company_header(card, as_of, "Karta wyników");
    body.push_str(&view_links(&card.row.code));
    body.push_str(&format!(
        "<div class=\"total\"><span>Wynik</span><div class=\"final-score\">{}</div></div>",
        score(card.row.final_score, true)
    ));
    body.push_str(&band(
        "01",
        "Fundament",
        card.row.fundamental_score,
        &render_fundamental(card),
    ));
    body.push_str(&band(
        "02",
        "Technika",
        card.row.technical_score,
        &render_technical(card),
    ));
    body.push_str(&band(
        "03",
        "Sentyment",
        card.row.sentiment_score,
        &render_sentiment(card),
    ));
    document(&format!("{} — wyniki", card.row.code), &body)
}

fn render_chart(card: &CompanyCard, as_of: NaiveDate, codes: &[String]) -> String {
    let mut body = page_heading("Historia punktów", as_of);
    body.push_str(&global_links("/historia"));
    body.push_str(&company_selector("/historia", &card.row.code, codes));
    body.push_str(&crate::charts::render(
        &card.candles,
        &card.currency,
        &card.saved_scores,
    ));
    document(&format!("{} — historia punktów", card.row.code), &body)
}

fn render_seasonality(
    card: &CompanyCard,
    as_of: NaiveDate,
    codes: &[String],
) -> Result<String, crate::technical::TechnicalError> {
    let mut body = page_heading("Sezonowość", as_of);
    body.push_str(&global_links("/sezonowosc"));
    body.push_str(&company_selector("/sezonowosc", &card.row.code, codes));
    if card.closes.is_empty() {
        body.push_str("<p class=\"missing\">Brak historii cen.</p>");
    } else {
        let data = crate::seasonality::calculate(&card.closes, as_of)?;
        body.push_str(&format!("<div class=\"season-summary\"><h2>{} {} · średni zwrot</h2><strong>{}</strong><span>{} lat</span></div>", crate::seasonality::month_name(as_of.month()), as_of.year(), decimal(data.mean, true), data.years));
        body.push_str(&crate::charts::render_seasonality(&data, as_of.year()));
    }
    Ok(document(&format!("{} — sezonowość", card.row.code), &body))
}

fn render_shorts(board: &crate::weekly_shorts::KnfBoard, as_of: NaiveDate) -> String {
    let mut body = page_heading("Shorty KNF", as_of);
    body.push_str(&global_links("/shorty"));
    body.push_str(r#"<section class="knf-guide" aria-label="Jak czytać dane KNF">
<div><h2>Co oznaczają liczby?</h2><p>To suma ujawnionych pozycji krótkich netto posiadaczy, w procentach wyemitowanego kapitału spółki. Źródło: <a href="https://rss.knf.gov.pl/rss_pub/">rejestr KNF</a>. To publiczne wpisy, a nie wszystkie shorty ani opinie wszystkich inwestorów.</p></div>
<div><h2>Punkt sentymentu: porównanie dni</h2><p>Najnowszy odczyt porównujemy z ostatnim odczytem z wcześniejszego dnia UTC. Spadek sumy daje +1, wzrost −1, brak zmiany 0. Obok punktu widać obie daty, sumy i różnicę. Pusty punkt oznacza brak pary odczytów.</p></div>
<div><h2>Historia tygodniowa: inny okres</h2><p>Każda kolumna to tydzień ISO, od poniedziałku. Pokazuje ostatni zapisany odczyt z tego tygodnia; bieżący tydzień jest niepełny. Daty oznaczają pobranie rejestru (UTC). Spadek między tygodniami nie musi dawać dzisiaj +1. W tej części 0 oznacza zapisany odczyt z sumą 0%, także pustą listę wpisów. Puste pole oznacza brak zapisu.</p></div>
</section>"#);
    if board.weeks.is_empty() {
        body.push_str("<p class=\"missing\">brak zapisanych odczytów rejestru</p>");
    } else {
        body.push_str(&format!("<div class=\"table-panel\"><table class=\"knf-board\"><thead><tr class=\"column-groups\"><th class=\"company-column\" scope=\"col\" rowspan=\"2\">Spółka</th><th scope=\"colgroup\" colspan=\"3\">Aktualny sentyment · porównanie dni UTC</th><th scope=\"colgroup\" colspan=\"{}\">Historia tygodniowa · suma (%)</th></tr><tr class=\"columns\"><th scope=\"col\">Punkt</th><th scope=\"col\">Porównywane odczyty (UTC)</th><th scope=\"col\">Zmiana (p.p.)</th>", board.weeks.len()));
        for week in &board.weeks {
            let end = week
                .checked_add_signed(chrono::Duration::days(6))
                .unwrap_or(*week);
            body.push_str(&format!("<th class=\"weekly-heading\" scope=\"col\" title=\"{week} — {end}\">{}-W{:02}</th>", week.iso_week().year(), week.iso_week().week()));
        }
        body.push_str("</tr></thead><tbody>");
        for row in &board.rows {
            let change = row
                .older
                .as_ref()
                .zip(row.newer.as_ref())
                .and_then(|(older, newer)| newer.total.checked_sub(older.total));
            let change = change
                .map(|value| {
                    format!(
                        "{}{}",
                        if value > Decimal::ZERO { "+" } else { "" },
                        decimal(Some(value), false)
                    )
                })
                .unwrap_or_default();
            body.push_str(&format!("<tr data-code=\"{}\"><th scope=\"row\"><a class=\"company-name\" href=\"/{}\">{}</a></th><td class=\"sentiment-point\">{}</td><td class=\"sentiment-readings\">{}{}</td><td class=\"sentiment-change\">{change}</td>",
                escape(&row.code), escape(&row.code), escape(&row.code), score(row.sentiment_score, false), short_reading("Poprzedni", row.older.as_ref()), short_reading("Najnowszy", row.newer.as_ref())));
            for week in &board.weeks {
                let observation = row.weekly.get(week);
                let title = observation
                    .map(|observation| {
                        format!(
                            "Ostatni odczyt: {} UTC",
                            observation.fetched_at.format("%Y-%m-%d %H:%M")
                        )
                    })
                    .unwrap_or_else(|| "Brak zapisanego odczytu".into());
                body.push_str(&format!(
                    "<td class=\"weekly-total\" title=\"{title}\">{}</td>",
                    decimal(observation.map(|observation| observation.total), false)
                ));
            }
            body.push_str("</tr>");
        }
        body.push_str("</tbody></table></div>");
    }
    document("Shorty KNF", &body)
}

fn short_reading(label: &str, reading: Option<&crate::charts::ShortTotal>) -> String {
    let Some(reading) = reading else {
        return String::new();
    };
    format!(
        "<span class=\"reading-line\"><span class=\"reading-label\">{label}</span><time datetime=\"{}\">{}</time><span class=\"reading-value\">{}%</span></span>",
        reading.fetched_at.to_rfc3339(),
        reading.fetched_at.format("%Y-%m-%d %H:%M"),
        decimal(Some(reading.total), false)
    )
}

fn band(number: &str, title: &str, points: Option<i8>, components: &str) -> String {
    format!(
        r#"<section class="band"><header class="band-heading"><span class="band-number">{number}</span>
<h2>{title}</h2>{}</header><div class="components">{components}</div></section>"#,
        score(points, true)
    )
}

fn render_fundamental(card: &CompanyCard) -> String {
    let Some((p, snapshot)) = &card.fundamental else {
        return String::new();
    };
    let f = &snapshot.fundamentals;
    let unit = format!("{} × {}", f.currency, f.unit_multiplier);
    let mut body = String::new();
    // This is the same Financials template selected by fundamental_points::score.
    if card.row.gics_sector != "Financials" {
        body.push_str(&metric(
            "Marża netto",
            "net_margin",
            p.net_margin_points,
            p.ratios.net_margin,
            true,
            &unit,
            &[
                ("Zysk netto", f.net_income_parent),
                ("Przychody", f.revenue),
            ],
        ));
        body.push_str(&metric(
            "Marża operacyjna",
            "operating_margin",
            p.operating_margin_points,
            p.ratios.operating_margin,
            true,
            &unit,
            &[
                ("Zysk operacyjny", f.operating_profit),
                ("Przychody", f.revenue),
            ],
        ));
    }
    body.push_str(&metric(
        "Kapitał",
        "equity_ratio",
        p.equity_ratio_points,
        p.ratios.equity_ratio,
        true,
        &unit,
        &[
            ("Kapitał własny (equity_parent)", f.equity_parent),
            ("Aktywa", f.assets),
        ],
    ));
    if card.row.gics_sector != "Financials" {
        body.push_str(&metric(
            "Dług",
            "debt_to_equity",
            p.debt_to_equity_points,
            p.ratios.debt_to_equity,
            false,
            &unit,
            &[
                ("Zobowiązania długoterminowe", f.long_term_liabilities),
                ("Zobowiązania krótkoterminowe", f.short_term_liabilities),
                ("Kapitał własny (equity_parent)", f.equity_parent),
            ],
        ));
        body.push_str(&metric(
            "Gotówka",
            "ocf_to_net_income",
            p.ocf_to_net_income_points,
            p.ratios.ocf_to_net_income,
            false,
            &unit,
            &[
                ("Przepływy operacyjne", f.operating_cash_flow),
                ("Zysk netto", f.net_income_parent),
            ],
        ));
    }
    body
}

fn metric(
    title: &str,
    key: &str,
    points: Option<i8>,
    ratio: Option<Decimal>,
    percent: bool,
    unit: &str,
    amounts: &[(&str, Option<Decimal>)],
) -> String {
    let mut details = String::new();
    for (label, value) in amounts {
        details.push_str(&datum(
            &format!("{label} ({unit})"),
            &decimal(*value, false),
        ));
    }
    details.push_str(&datum(key, &decimal(ratio, percent)));
    format!(
        r#"<article class="metric" data-metric="{key}"><div class="metric-heading"><h3>{title}</h3>{}</div>
<dl>{details}</dl></article>"#,
        score(points, true)
    )
}

fn render_technical(card: &CompanyCard) -> String {
    let Some(t) = &card.technical else {
        return String::new();
    };
    let e = &t.explanation;
    format!(
        r#"<article class="metric" data-metric="seasonality_points"><div class="metric-heading"><h3>Sezonowość</h3>{}</div><dl>{}{}</dl></article>
<article class="metric" data-metric="trend_points"><div class="metric-heading"><h3>Trend SMA200</h3>{}</div><dl>{}{}{}</dl></article>"#,
        score(t.seasonality_points, true),
        datum("Średni zwrot R", &decimal(e.r, true)),
        datum("Liczba lat", &e.years.to_string()),
        score(Some(t.trend_points), true),
        datum("Ostatni close", &decimal(e.close, false)),
        datum("SMA200", &decimal(e.sma200, false)),
        datum("Odchylenie d", &decimal(e.d, true))
    )
}

fn render_sentiment(card: &CompanyCard) -> String {
    let mut body = String::from("<dl class=\"short-totals\">");
    for (label, snapshot) in [
        ("Suma nowszego odczytu", &card.newer_shorts),
        ("Suma starszego odczytu", &card.older_shorts),
    ] {
        let value = snapshot
            .as_ref()
            .map(|snapshot| format!("{}%", decimal(Some(short_total(snapshot)), false)))
            .unwrap_or_default();
        body.push_str(&datum(label, &value));
        if let Some(snapshot) = snapshot {
            body.push_str(&datum(
                "Data odczytu UTC",
                &snapshot.fetched_at.to_rfc3339(),
            ));
        }
    }
    body.push_str("</dl>");
    body
}

fn short_total(snapshot: &ShortsSnapshot) -> Decimal {
    snapshot
        .positions
        .iter()
        .map(|p| p.net_short_position_percent)
        .sum()
}

fn datum(label: &str, value: &str) -> String {
    format!(
        "<div><dt>{}</dt><dd>{}</dd></div>",
        escape(label),
        escape(value)
    )
}

fn decimal(value: Option<Decimal>, percent: bool) -> String {
    value
        .and_then(|value| {
            if percent {
                value.checked_mul(Decimal::from(100))
            } else {
                Some(value)
            }
        })
        .map(|value| {
            format!(
                "{}{}",
                value.round_dp(4).normalize(),
                if percent { "%" } else { "" }
            )
        })
        .unwrap_or_default()
}

fn score(value: Option<i8>, with_arrow: bool) -> String {
    let Some(value) = value else {
        return "<span class=\"score\" aria-label=\"brak danych\"></span>".into();
    };
    let (class, arrow) = match value.cmp(&0) {
        std::cmp::Ordering::Greater => ("positive", "↑"),
        std::cmp::Ordering::Less => ("negative", "↓"),
        std::cmp::Ordering::Equal => ("neutral", "—"),
    };
    let arrow = if with_arrow {
        format!("<span class=\"arrow\" aria-hidden=\"true\">{arrow}</span>")
    } else {
        String::new()
    };
    let number = if value == 0 && with_arrow {
        String::new()
    } else if value < 0 {
        format!("−{}", -i16::from(value))
    } else {
        value.to_string()
    };
    format!("<span class=\"score {class}\">{arrow}<span>{number}</span></span>")
}

pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
#[path = "page_tests.rs"]
mod tests;
