//! Shared local HTML page for the existing ranking function.

use std::{fmt::Display, path::PathBuf};

use crate::ranking::{RankingRow, rank_directory};
use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    routing::get,
};
use chrono::{NaiveDate, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
struct RankingQuery {
    as_of: Option<NaiveDate>,
}

pub async fn serve(directory: PathBuf) -> std::io::Result<()> {
    let app = Router::new().route("/", get(index)).with_state(directory);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    eprintln!("rankr: http://127.0.0.1:3000/");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(
    State(directory): State<PathBuf>,
    Query(query): Query<RankingQuery>,
) -> Result<Html<String>, (StatusCode, &'static str)> {
    let as_of = query.as_of.unwrap_or_else(|| Utc::now().date_naive());
    // Reading and scoring JSON archives is synchronous work.
    let ranking = tokio::task::spawn_blocking(move || rank_directory(&directory, as_of))
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

fn server_error(error: impl Display) -> (StatusCode, &'static str) {
    eprintln!("Ranking error: {error}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Nie udało się odczytać danych rankingu.",
    )
}

fn render(rows: &[RankingRow], as_of: NaiveDate) -> String {
    let mut body = String::new();
    for row in rows {
        body.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            escape(&row.code),
            escape(&row.gics_sector),
            cell(row.fundamental_score),
            cell(row.technical_score),
            cell(row.sentiment_score),
            cell(row.final_score),
        ));
    }
    format!(
        r#"<!doctype html>
<html lang="pl">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>rankr — ranking spółek GPW</title>
<style>
body {{ font-family: system-ui, sans-serif; margin: 2rem; color: #1e293b; background: #f8fafc; }}
main {{ max-width: 75rem; margin: auto; }}
.table {{ overflow-x: auto; }}
table {{ border-collapse: collapse; width: 100%; background: white; }}
th, td {{ padding: 0.75rem; border-bottom: 1px solid #cbd5e1; text-align: left; }}
th {{ background: #e2e8f0; }}
th:nth-child(n+3), td:nth-child(n+3) {{ text-align: right; font-variant-numeric: tabular-nums; }}
</style>
</head>
<body>
<main>
<h1>rankr — ranking spółek GPW</h1>
<p>Data rankingu: {as_of}</p>
<div class="table">
<table>
<thead><tr><th scope="col">code</th><th scope="col">gics_sector</th><th scope="col">fundamental_score</th><th scope="col">technical_score</th><th scope="col">sentiment_score</th><th scope="col">final_score</th></tr></thead>
<tbody>
{body}</tbody>
</table>
</div>
</main>
</body>
</html>
"#
    )
}

fn cell(score: Option<i8>) -> String {
    score.map(|value| value.to_string()).unwrap_or_default()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_family_renders_an_empty_cell_while_zero_remains_zero() {
        let row = RankingRow {
            code: "KGHM".into(),
            gics_sector: "Materials".into(),
            fundamental_score: Some(0),
            technical_score: None,
            sentiment_score: Some(1),
            final_score: None,
        };
        let html = render(&[row], NaiveDate::from_ymd_opt(2026, 10, 26).unwrap());
        assert!(html.contains(
            "<tr><td>KGHM</td><td>Materials</td><td>0</td><td></td><td>1</td><td></td></tr>"
        ));
    }
}
