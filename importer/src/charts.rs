//! SVG geometry only. Scoring continues to use Decimal and its existing formulas.

use std::fmt::Write;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::{company_archive::SavedScore, page::escape, seasonality::Seasonality};

pub(crate) struct OhlcCandle {
    pub date: NaiveDate,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
}

pub(crate) struct ShortTotal {
    pub fetched_at: DateTime<Utc>,
    /// KNF values are already percentages: 0.83 means 0.83%.
    pub total: Decimal,
}

// Horizontal positions are percentages. Vertical positions and font sizes stay
// in CSS pixels, so resizing a wide window never flattens the image or its text.
const LEFT: f64 = 6.0;
const RIGHT: f64 = 94.0;
const PRICE_TOP: f64 = 48.0;
const PRICE_BOTTOM: f64 = 560.0;
const SCORE_TOP: f64 = 640.0;
const SCORE_BOTTOM: f64 = 840.0;

struct DateAxis {
    start: NaiveDate,
    days: i64,
    step: f64,
}

impl DateAxis {
    fn new(dates: impl Iterator<Item = NaiveDate>) -> Option<Self> {
        let (start, end) = dates.fold(None, |range, date| match range {
            None => Some((date, date)),
            Some((start, end)) => Some((date.min(start), date.max(end))),
        })?;
        let days = (end - start).num_days() + 1;
        Some(Self {
            start,
            days,
            step: (RIGHT - LEFT) / days as f64,
        })
    }

    fn x(&self, date: NaiveDate) -> f64 {
        LEFT + ((date - self.start).num_days() as f64 + 0.5) * self.step
    }

    fn draw(&self, svg: &mut String, y: f64) {
        let ticks = (self.days - 1).min(5);
        for tick in 0..=ticks {
            let offset = if ticks == 0 {
                0
            } else {
                (self.days - 1) * tick / ticks
            };
            if let Some(date) = self
                .start
                .checked_add_signed(chrono::Duration::days(offset))
            {
                let anchor = if tick == 0 {
                    "start"
                } else if tick == ticks {
                    "end"
                } else {
                    "middle"
                };
                let _ = write!(
                    svg,
                    "<text class=\"axis-label\" x=\"{:.4}%\" y=\"{y}\" text-anchor=\"{anchor}\">{date}</text>",
                    self.x(date)
                );
            }
        }
    }
}

pub(crate) fn render(candles: &[OhlcCandle], currency: &str, scores: &[SavedScore]) -> String {
    let first_score = scores
        .iter()
        .find(|score| score.score.is_some())
        .map(|score| score.date.to_string())
        .unwrap_or_default();
    let mut html = format!(
        "<section class=\"chart-panel\" data-history-chart data-first-score=\"{first_score}\">"
    );
    if candles.is_empty() {
        html.push_str("<p class=\"missing\">Brak zapisanej historii OHLC.</p>");
    }
    if let Some(axis) = DateAxis::new(
        candles
            .iter()
            .map(|c| c.date)
            .chain(scores.iter().map(|s| s.date)),
    ) {
        html.push_str(r#"<div class="chart-controls" hidden aria-label="Zakres historii">
<button type="button" data-range="scores">Od pierwszej oceny</button><button type="button" data-range="90">3 miesiące</button><button type="button" data-range="365">Rok</button><button type="button" data-range="all">Całość</button>
<button type="button" data-zoom="0.5" aria-label="Powiększ wykres">+</button><button type="button" data-zoom="2" aria-label="Pomniejsz wykres">−</button>
<button type="button" data-pan="-0.5">Wcześniej</button><button type="button" data-pan="0.5">Później</button>
<label>Od <input type="date" data-from></label><label>Do <input type="date" data-to></label><button type="button" data-apply>Pokaż</button><output class="chart-range" data-range-label></output></div>"#);
        let end = axis.start + chrono::Duration::days(axis.days - 1);
        let attrs = format!(
            "data-domain-start=\"{}\" data-domain-end=\"{end}\" tabindex=\"0\"",
            axis.start
        );
        let mut svg = svg_start("price-score-chart", "Cena i zapisane wyniki", 900.0, &attrs);
        let _ = write!(
            svg,
            "<text class=\"panel-label\" x=\"6%\" y=\"26\">Cena {}</text><g class=\"price-chart\">",
            escape(currency)
        );
        draw_prices(&mut svg, candles, &axis);
        svg.push_str("</g><text class=\"panel-label\" x=\"6%\" y=\"610\">Wynik</text><g class=\"score-chart\">");
        draw_scores(&mut svg, scores, &axis);
        svg.push_str("</g><g class=\"time-axis\">");
        axis.draw(&mut svg, 878.0);
        svg.push_str("</g><line class=\"chart-cursor\" x1=\"6%\" x2=\"6%\" y1=\"48\" y2=\"840\" visibility=\"hidden\"/>");
        svg_end(&mut svg);
        html.push_str(&svg);
        html.push_str("<div class=\"chart-tooltip\" data-tooltip aria-live=\"off\"></div>");
    }
    html.push_str("</section>");
    html
}

fn svg_start(class: &str, label: &str, height: f64, attrs: &str) -> String {
    format!(
        "<div class=\"plot\"><svg xmlns=\"http://www.w3.org/2000/svg\" class=\"{class}\" role=\"img\" aria-label=\"{label}\" width=\"100%\" height=\"{height}\" {attrs}>"
    )
}

fn svg_end(svg: &mut String) {
    svg.push_str("</svg></div>");
}

fn horizontal_tick(svg: &mut String, y: f64, value: &str) {
    let _ = write!(
        svg,
        "<line class=\"grid-line\" x1=\"{LEFT}%\" y1=\"{y:.2}\" x2=\"{RIGHT}%\" y2=\"{y:.2}\"/><text class=\"axis-label\" x=\"5%\" y=\"{:.2}\" text-anchor=\"end\">{value}</text><text class=\"axis-label\" x=\"95%\" y=\"{:.2}\">{value}</text>",
        y + 5.0,
        y + 5.0
    );
}

fn draw_prices(svg: &mut String, candles: &[OhlcCandle], axis: &DateAxis) {
    if candles.is_empty() {
        return;
    }
    let low = candles
        .iter()
        .map(|c| number(c.low))
        .fold(f64::INFINITY, f64::min);
    let high = candles
        .iter()
        .map(|c| number(c.high))
        .fold(f64::NEG_INFINITY, f64::max);
    let padding = ((high - low) * 0.06).max(high * 0.005).max(0.01);
    let bottom = (low - padding).max(0.0);
    let top = high + padding;
    let y = |price: Decimal| {
        PRICE_TOP + (top - number(price)) / (top - bottom) * (PRICE_BOTTOM - PRICE_TOP)
    };
    for tick in 0..=4 {
        let fraction = f64::from(tick) / 4.0;
        let value = top - fraction * (top - bottom);
        horizontal_tick(
            svg,
            PRICE_TOP + fraction * (PRICE_BOTTOM - PRICE_TOP),
            &format!("{value:.2}"),
        );
    }
    for candle in candles {
        let x = axis.x(candle.date);
        let class = if candle.close > candle.open {
            "positive"
        } else if candle.close < candle.open {
            "negative"
        } else {
            "neutral"
        };
        let body_top = y(candle.open.max(candle.close));
        let body_height = (y(candle.open) - y(candle.close)).abs().max(1.0);
        let body_width = (axis.step * 0.65).min(0.8);
        let _ = write!(
            svg,
            "<g class=\"candle {class}\" data-date=\"{}\" data-x=\"{x:.6}\" data-open=\"{}\" data-high=\"{}\" data-low=\"{}\" data-close=\"{}\"><title>{}: O {} · H {} · L {} · C {}</title><line x1=\"{x:.6}%\" y1=\"{:.2}\" x2=\"{x:.6}%\" y2=\"{:.2}\"/><rect x=\"{:.6}%\" y=\"{body_top:.2}\" width=\"{body_width:.6}%\" height=\"{body_height:.2}\"/></g>",
            candle.date,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            candle.date,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            y(candle.high),
            y(candle.low),
            x - body_width / 2.0
        );
    }
}

fn draw_scores(svg: &mut String, scores: &[SavedScore], axis: &DateAxis) {
    if scores.iter().all(|score| score.score.is_none()) {
        return;
    }
    let max_score = scores
        .iter()
        .filter_map(|s| s.score)
        .map(|s| f64::from(s).abs())
        .fold(1.0, f64::max);
    let middle = (SCORE_TOP + SCORE_BOTTOM) / 2.0;
    let scale = (SCORE_BOTTOM - SCORE_TOP) / 2.0 / max_score;
    for value in [-max_score, 0.0, max_score] {
        horizontal_tick(svg, middle - value * scale, &format!("{value:.0}"));
    }
    for saved in scores {
        let Some(value) = saved.score else { continue };
        let x = axis.x(saved.date);
        let y = middle - f64::from(value) * scale;
        let bar_width = (axis.step * 0.65).min(1.5);
        if value == 0 {
            let _ = write!(
                svg,
                "<line class=\"score-zero\" data-date=\"{}\" data-score=\"0\" data-x=\"{x:.6}\" x1=\"{:.6}%\" y1=\"{middle:.2}\" x2=\"{:.6}%\" y2=\"{middle:.2}\"><title>{}: 0</title></line>",
                saved.date,
                x - bar_width / 2.0,
                x + bar_width / 2.0,
                saved.date
            );
        } else {
            let class = if value > 0 { "positive" } else { "negative" };
            let _ = write!(
                svg,
                "<rect class=\"score-bar {class}\" data-date=\"{}\" data-score=\"{value}\" data-x=\"{x:.6}\" x=\"{:.6}%\" y=\"{:.2}\" width=\"{bar_width:.6}%\" height=\"{:.2}\"><title>{}: {value}</title></rect>",
                saved.date,
                x - bar_width / 2.0,
                y.min(middle),
                (y - middle).abs(),
                saved.date
            );
        }
    }
}

pub(crate) fn render_seasonality(data: &Seasonality, year: i32) -> String {
    let mut series = Vec::new();
    if !data.current.is_empty() {
        series.push(("current", year.to_string(), data.current.as_slice()));
    }
    if !data.all.is_empty() {
        series.push((
            "all",
            "Średnia · wszystkie lata".into(),
            data.all.as_slice(),
        ));
    }
    if let Some(path) = &data.five {
        series.push(("five", "Średnia · 5 lat".into(), path.as_slice()));
    }
    if let Some(path) = &data.ten {
        series.push(("ten", "Średnia · 10 lat".into(), path.as_slice()));
    }
    if series.is_empty() {
        return "<p class=\"missing\">Brak historii dla tego miesiąca.</p>".into();
    }
    let sessions = series
        .iter()
        .map(|(_, _, path)| path.len())
        .max()
        .unwrap_or(1)
        .max(2);
    let low = series
        .iter()
        .flat_map(|(_, _, path)| *path)
        .map(|value| number(*value) * 100.0)
        .fold(0.0, f64::min);
    let high = series
        .iter()
        .flat_map(|(_, _, path)| *path)
        .map(|value| number(*value) * 100.0)
        .fold(0.0, f64::max);
    let padding = ((high - low) * 0.08).max(0.1);
    let top = high + padding;
    let bottom = low - padding;
    let x = |session: usize| LEFT + session as f64 / (sessions - 1) as f64 * (RIGHT - LEFT);
    let y = |value: Decimal| 48.0 + (top - number(value) * 100.0) / (top - bottom) * 712.0;
    let mut html = String::from("<section class=\"chart-panel\"><div class=\"series-labels\">");
    for (kind, label, _) in &series {
        let _ = write!(
            html,
            "<span class=\"series-label season-{kind}\">{label}</span>"
        );
    }
    html.push_str("</div>");
    html.push_str(&svg_start(
        "seasonality-chart",
        "Sezonowość bieżącego miesiąca",
        840.0,
        "",
    ));
    for tick in 0..=4 {
        let fraction = f64::from(tick) / 4.0;
        let value = top - fraction * (top - bottom);
        horizontal_tick(&mut html, 48.0 + fraction * 712.0, &format!("{value:.2}%"));
    }
    for (kind, label, path) in &series {
        let _ = write!(
            html,
            "<g class=\"season-series season-{kind}\" data-series=\"{kind}\"><title>{label}</title>"
        );
        for (session, pair) in path.windows(2).enumerate() {
            let _ = write!(
                html,
                "<line x1=\"{:.4}%\" y1=\"{:.2}\" x2=\"{:.4}%\" y2=\"{:.2}\"/>",
                x(session),
                y(pair[0]),
                x(session + 1),
                y(pair[1])
            );
        }
        for (session, value) in path.iter().enumerate() {
            let _ = write!(
                html,
                "<circle cx=\"{:.4}%\" cy=\"{:.2}\" r=\"2\"><title>Sesja {}: {:.4}%</title></circle>",
                x(session),
                y(*value),
                session + 1,
                number(*value) * 100.0
            );
        }
        html.push_str("</g>");
    }
    let ticks = (sessions - 1).min(5);
    for tick in 0..=ticks {
        let session = (sessions - 1) * tick / ticks;
        let _ = write!(
            html,
            "<text class=\"axis-label\" x=\"{:.4}%\" y=\"792\" text-anchor=\"middle\">{}</text>",
            x(session),
            session + 1
        );
    }
    html.push_str("<text class=\"axis-label\" x=\"50%\" y=\"822\" text-anchor=\"middle\">Sesja miesiąca</text>");
    svg_end(&mut html);
    html.push_str("</section>");
    html
}

fn number(value: Decimal) -> f64 {
    value.to_f64().expect("Decimal fits in f64")
}
