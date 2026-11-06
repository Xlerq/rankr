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

async fn company(
    State(state): State<PageState>,
    Path(code): Path<String>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let as_of = Utc::now().date_naive();
    // The URL is only a lookup key; it is never joined to a filesystem path.
    let card = tokio::task::spawn_blocking(move || scorecard::load(&state.archives, &code, as_of))
        .await
        .map_err(server_error)?
        .map_err(server_error)?
        .ok_or((
            StatusCode::NOT_FOUND,
            "Nie znaleziono spółki o podanym kodzie.",
        ))?;
    Ok(Html(render_company(&card, as_of)))
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
{body}<footer>Wagi rodzin 1 / 1 / 1 <span>Fundament + technika + sentyment</span></footer>
</main></body></html>"#,
        escape(title),
        include_str!("page.css")
    )
}

fn render(rows: &[RankingRow], as_of: NaiveDate) -> String {
    let mut body = String::new();
    for row in rows {
        body.push_str(&format!(
            r#"<tr>
<td class="final-score">{}</td>
<th scope="row"><a class="row-link company-name" href="/{}">{}</a></th>
<td class="sector">{}</td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
            score(row.final_score, false),
            escape(&row.code),
            escape(&row.code),
            escape(&row.gics_sector),
            score(row.fundamental_score, true),
            score(row.technical_score, true),
            score(row.sentiment_score, true)
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
<header class="page-heading"><div><p class="eyebrow">Potencjał wzrostu</p><h1>Ranking spółek</h1></div>
<p class="date">{as_of}<span>Data rankingu · UTC</span></p></header>
<div class="table-panel"><table class="ranking"><thead><tr>
<th scope="col">Wynik</th><th scope="col">Spółka</th><th scope="col">Sektor GICS</th>
<th scope="col">Fundament</th><th scope="col">Technika</th><th scope="col">Sentyment</th>
</tr></thead><tbody>{body}</tbody></table></div>
<p class="legend">↑ dodatni punkt <span>↓ ujemny punkt</span> — zero <span>Puste pole: brak wyniku</span></p>"#
        ),
    )
}

fn render_company(card: &CompanyCard, as_of: NaiveDate) -> String {
    let row = &card.row;
    let mut body = format!(
        r#"<a class="back" href="/">← Ranking spółek</a>
<header class="page-heading company-heading"><div><p class="eyebrow">Karta spółki · {as_of} UTC</p>
<h1 class="company-name">{}</h1><p class="sector">Sektor GICS · {}</p></div>
<div class="total"><span class="eyebrow">Wynik końcowy</span><div class="final-score">{}</div></div></header>"#,
        escape(&row.code),
        escape(&row.gics_sector),
        score(row.final_score, false)
    );
    if row.final_score.is_none() {
        body.push_str("<p class=\"missing\">Brak wyniku co najmniej jednej rodziny.</p>");
    }
    body.push_str(&crate::charts::render(
        &card.candles,
        &card.currency,
        &card.saved_scores,
    ));
    body.push_str(&band(
        "01",
        "Fundament",
        row.fundamental_score,
        &render_fundamental(card),
    ));
    body.push_str(&band(
        "02",
        "Technika",
        row.technical_score,
        &render_technical(card),
    ));
    body.push_str(&band(
        "03",
        "Sentyment",
        row.sentiment_score,
        &render_sentiment(card),
    ));
    document(&format!("{} — karta spółki", row.code), &body)
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
        return "<p class=\"missing\">Brak raportu pozwalającego obliczyć wskaźniki.</p>".into();
    };
    let f = &snapshot.fundamentals;
    let mut body = format!(
        "<p class=\"source-note\">Raport: {} · Kwoty raportu: {} × {} · Pobranie: {}</p>",
        escape(&f.report_period),
        escape(&f.currency),
        f.unit_multiplier,
        snapshot.fetched_at
    );
    // This is the same Financials template selected by fundamental_points::score.
    if card.row.gics_sector != "Financials" {
        body.push_str(&metric(
            "Marża netto",
            "net_margin",
            p.net_margin_points,
            p.ratios.net_margin,
            true,
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
    amounts: &[(&str, Option<Decimal>)],
) -> String {
    let mut details = String::new();
    for (label, value) in amounts {
        details.push_str(&datum(label, &decimal(*value, false)));
    }
    details.push_str(&datum(key, &decimal(ratio, percent)));
    let missing = if points.is_none() {
        "<p class=\"missing\">Brak danych lub poprawnego mianownika.</p>"
    } else {
        ""
    };
    format!(
        r#"<article class="metric" data-metric="{key}"><div class="metric-heading"><h3>{title}</h3>{}</div>
<dl>{details}</dl>{missing}</article>"#,
        score(points, true)
    )
}

fn render_technical(card: &CompanyCard) -> String {
    let Some(t) = &card.technical else {
        return "<p class=\"missing\">Brak historii pozwalającej obliczyć wynik techniczny.</p>"
            .into();
    };
    let e = &t.explanation;
    let mut body = format!(
        r#"<article class="metric" data-metric="seasonality_points">
<div class="metric-heading"><h3>Sezonowość</h3>{}</div><dl>{}{}</dl>{}</article>
<article class="metric" data-metric="trend_points"><div class="metric-heading"><h3>Trend SMA200</h3>{}</div>
<dl>{}{}{}</dl><p class="source-note">Sesje: {} / 200 · Ostatnia sesja: {}</p></article>"#,
        score(t.seasonality_points, true),
        datum("Średni zwrot R", &decimal(e.r, true)),
        datum("Liczba lat", &e.years.to_string()),
        if t.seasonality_points.is_none() {
            "<p class=\"missing\">Brak porównywalnych miesięcy z poprzednich lat.</p>"
        } else {
            ""
        },
        score(Some(t.trend_points), true),
        datum("Ostatni close", &decimal(e.close, false)),
        datum("SMA200", &decimal(e.sma200, false)),
        datum("Odchylenie d", &decimal(e.d, true)),
        e.trend_sessions,
        e.last_session
            .map(|date| date.to_string())
            .unwrap_or_default()
    );
    if t.technical_score.is_none() && t.seasonality_points.is_some() {
        body.push_str("<p class=\"missing\">Historia cen jest nieaktualna.</p>");
    }
    body
}

fn render_sentiment(card: &CompanyCard) -> String {
    let mut body = String::from(
        "<p class=\"source-note\">Rejestr krótkiej sprzedaży KNF · ujawnione pozycje netto</p>",
    );
    if card.older_shorts.is_none() || card.newer_shorts.is_none() {
        body.push_str("<p class=\"missing\">brak drugiego odczytu rejestru</p>");
    }
    body.push_str("<dl class=\"short-totals\">");
    for (label, snapshot) in [
        ("Suma nowszego odczytu", &card.newer_shorts),
        ("Suma starszego odczytu", &card.older_shorts),
    ] {
        // KNF stores percentages already; do not multiply by 100 here.
        let total = snapshot.as_ref().map(short_total);
        let value = total
            .map(|value| format!("{}%", decimal(Some(value), false)))
            .unwrap_or_default();
        body.push_str(&datum(label, &value));
        if let Some(snapshot) = snapshot {
            body.push_str(&datum(
                "Data odczytu UTC",
                &snapshot.fetched_at.to_rfc3339(),
            ));
        }
    }
    body.push_str("</dl><h3 class=\"holders-title\">Posiadacze w nowszym odczycie</h3><div class=\"table-panel\"><table class=\"holders\"><thead><tr><th scope=\"col\">Posiadacz</th><th scope=\"col\">Pozycja netto</th><th scope=\"col\">Data pozycji</th></tr></thead><tbody>");
    if let Some(snapshot) = &card.newer_shorts {
        for position in &snapshot.positions {
            body.push_str(&format!(
                "<tr><td>{}</td><td class=\"number\">{}%</td><td class=\"number\">{}</td></tr>",
                escape(&position.holder),
                decimal(Some(position.net_short_position_percent), false),
                position.position_date
            ));
        }
    }
    body.push_str("</tbody></table></div>");
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
    let number = if value < 0 {
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
