//! Local, server-rendered ranking and company scorecard.

use std::{fmt::Display, path::PathBuf, sync::Arc};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;

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
}

fn router(directory: PathBuf) -> std::io::Result<Router> {
    let archives = Arc::new(ArchiveIndex::read(&directory)?);
    Ok(Router::new()
        .route("/", get(index))
        .route("/{code}", get(company))
        .route("/{code}/wykres", get(chart))
        .route("/{code}/shorty", get(shorts))
        .with_state(PageState {
            directory,
            archives,
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

async fn chart(
    State(state): State<PageState>,
    Path(code): Path<String>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let (card, as_of) = company_data(state, code).await?;
    Ok(Html(render_chart(&card, as_of)))
}

async fn shorts(
    State(state): State<PageState>,
    Path(code): Path<String>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let (card, as_of) = company_data(state, code).await?;
    Ok(Html(render_shorts(&card, as_of)))
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
<body><main><nav><a class="brand" href="/">rankr<span>.</span></a><span>GPW / WIG20</span></nav>
{body}
</main></body></html>"#,
        escape(title),
        include_str!("page.css")
    )
}

fn render(rows: &[RankingRow], as_of: NaiveDate) -> String {
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
<header class="page-heading"><h1>Ranking spółek</h1><p class="date">{as_of}<span>Data rankingu · UTC</span></p></header>
<div class="table-panel"><table class="ranking"><thead><tr>
<th scope="col">Spółka</th><th scope="col">Sektor</th><th scope="col">Fundament</th><th scope="col">Technika</th><th scope="col">Sentyment</th><th scope="col">Wynik</th>
</tr></thead><tbody>{body}</tbody></table></div>"#
        ),
    )
}

fn company_header(card: &CompanyCard, as_of: NaiveDate, view: &str) -> String {
    format!(
        r#"<a class="back" href="/">Ranking</a>
<header class="page-heading company-heading"><div><p class="eyebrow">{view}</p><h1 class="company-name">{}</h1><p class="sector">{}</p></div>
<p class="date">{as_of}<span>Data rankingu · UTC</span></p></header>"#,
        escape(&card.row.code),
        escape(&card.row.gics_sector)
    )
}

fn view_links(code: &str, current: &str) -> String {
    let mut links = String::from("<nav class=\"view-links\" aria-label=\"Widoki spółki\">");
    for (view, suffix) in [("Wyniki", ""), ("Wykres", "/wykres"), ("Shorty", "/shorty")] {
        if view != current {
            links.push_str(&format!("<a href=\"/{}{suffix}\">{view}</a>", escape(code)));
        }
    }
    links.push_str("</nav>");
    links
}

fn render_company(card: &CompanyCard, as_of: NaiveDate) -> String {
    let mut body = company_header(card, as_of, "Karta wyników");
    body.push_str(&view_links(&card.row.code, "Wyniki"));
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

fn render_chart(card: &CompanyCard, as_of: NaiveDate) -> String {
    let mut body = company_header(card, as_of, "Wykres");
    body.push_str(&view_links(&card.row.code, "Wykres"));
    body.push_str(&crate::charts::render(
        &card.candles,
        &card.currency,
        &card.saved_scores,
    ));
    document(&format!("{} — wykres", card.row.code), &body)
}

fn render_shorts(card: &CompanyCard, as_of: NaiveDate) -> String {
    let mut body = company_header(card, as_of, "Shorty");
    body.push_str(&view_links(&card.row.code, "Shorty"));
    let weeks = crate::weekly_shorts::group(&card.short_totals);
    if weeks.is_empty() {
        body.push_str("<p class=\"missing\">brak zapisanych odczytów rejestru</p>");
    } else {
        body.push_str("<div class=\"table-panel\"><table class=\"weekly-shorts\"><thead><tr><th scope=\"col\">Tydzień</th><th scope=\"col\">Suma (%)</th><th scope=\"col\">Zmiana (p.p.)</th></tr></thead><tbody>");
        for week in &weeks {
            let change = week
                .change
                .map(|value| {
                    let sign = if value > Decimal::ZERO { "+" } else { "" };
                    format!("{sign}{}", decimal(Some(value), false))
                })
                .unwrap_or_default();
            body.push_str(&format!(
                "<tr><th scope=\"row\">{}</th><td>{}</td><td>{change}</td></tr>",
                week.week,
                decimal(Some(week.total), false)
            ));
        }
        body.push_str("</tbody></table></div>");
        body.push_str(&crate::charts::render_shorts(&weeks));
    }
    document(&format!("{} — shorty", card.row.code), &body)
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
