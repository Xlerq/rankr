//! Standalone SVG geometry. Floating point is used only for drawing, never scoring.

use std::fmt::Write;

use chrono::NaiveDate;
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::{company_archive::SavedScore, page::escape};

pub(crate) struct OhlcCandle {
    pub date: NaiveDate,
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
}

const LEFT: f64 = 72.0;
const RIGHT: f64 = 84.0;
const TOP: f64 = 24.0;
const PRICE_BOTTOM: f64 = 280.0;
const SCORE_BOTTOM: f64 = 156.0;
const MIN_WIDTH: f64 = 860.0;

pub(crate) fn render(candles: &[OhlcCandle], currency: &str, scores: &[SavedScore]) -> String {
    let mut html = String::from(
        "<section class=\"chart-panel\"><h2>Historia ceny i zapisanych ocen</h2><h3>Cena · świece OHLC</h3>",
    );
    if candles.is_empty() {
        html.push_str("<p class=\"missing\">Brak zapisanej historii OHLC.</p>");
    } else {
        let _ = write!(
            html,
            "<p class=\"source-note\">{} — {} · {} sesji · {}. Przewiń wykres poziomo, aby zobaczyć wcześniejsze sesje.</p>",
            candles.first().unwrap().date,
            candles.last().unwrap().date,
            candles.len(),
            escape(currency)
        );
        html.push_str(&price_svg(candles));
    }
    html.push_str("<h3>Zapisana ocena końcowa · final_score</h3>");
    html.push_str(&score_svg(scores));
    if scores.is_empty() {
        html.push_str(
            "<p class=\"missing\">Brak zapisanych snapshotów rankingu dla tej spółki.</p>",
        );
    } else {
        let _ = write!(
            html,
            "<p class=\"source-note\">{} — {} · ostatni zapis z każdego dnia UTC. Puste dni nie mają słupków; kreska oznacza zapisane zero. Historia ocen zaczyna się od pierwszego zapisu rankingu.</p>",
            scores.first().unwrap().date,
            scores.last().unwrap().date
        );
        if scores.iter().all(|score| score.score.is_none()) {
            html.push_str(
                "<p class=\"missing\">Zapisane snapshoty nie zawierają dostępnego final_score.</p>",
            );
        }
    }
    html.push_str("</section>");
    html
}

fn svg_start(class: &str, label: &str, width: f64, height: f64) -> String {
    format!(
        "<div class=\"plot\" dir=\"rtl\"><svg xmlns=\"http://www.w3.org/2000/svg\" class=\"{class}\" dir=\"ltr\" role=\"img\" aria-label=\"{label}\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">"
    )
}

fn svg_end(svg: &mut String) {
    svg.push_str("</svg></div>");
}

fn price_svg(candles: &[OhlcCandle]) -> String {
    let width = MIN_WIDTH.max(LEFT + RIGHT + candles.len() as f64 * 6.0);
    let step = (width - LEFT - RIGHT) / candles.len() as f64;
    let low = candles
        .iter()
        .map(|c| number(c.low))
        .fold(f64::INFINITY, f64::min);
    let high = candles
        .iter()
        .map(|c| number(c.high))
        .fold(f64::NEG_INFINITY, f64::max);
    let padding = ((high - low) * 0.06).max(high * 0.005).max(0.01);
    let bottom = low - padding;
    let top = high + padding;
    let y = |price: Decimal| TOP + (top - number(price)) / (top - bottom) * (PRICE_BOTTOM - TOP);
    let mut svg = svg_start("price-chart", "Historia cen: świece OHLC", width, 322.0);
    for tick in 0..=4 {
        let fraction = f64::from(tick) / 4.0;
        let level = TOP + fraction * (PRICE_BOTTOM - TOP);
        let value = top - fraction * (top - bottom);
        let _ = write!(
            svg,
            "<line class=\"grid-line\" x1=\"{LEFT}\" y1=\"{level:.2}\" x2=\"{}\" y2=\"{level:.2}\"/><text class=\"axis-label\" x=\"62\" y=\"{:.2}\" text-anchor=\"end\">{value:.2}</text>",
            width - RIGHT,
            level + 5.0
        );
        let _ = write!(
            svg,
            "<text class=\"axis-label\" x=\"{:.2}\" y=\"{:.2}\">{value:.2}</text>",
            width - RIGHT + 10.0,
            level + 5.0
        );
    }
    let label_every = (160.0 / step).ceil().max(1.0) as usize;
    for (index, candle) in candles.iter().enumerate() {
        let x = LEFT + (index as f64 + 0.5) * step;
        let class = if candle.close > candle.open {
            "positive"
        } else if candle.close < candle.open {
            "negative"
        } else {
            "neutral"
        };
        let body_top = y(candle.open.max(candle.close));
        let body_height = (y(candle.open) - y(candle.close)).abs().max(1.0);
        let body_width = (step * 0.65).min(12.0);
        let _ = write!(
            svg,
            "<g class=\"candle {class}\" data-date=\"{}\"><title>{}: O {} · H {} · L {} · C {}</title><line x1=\"{x:.2}\" y1=\"{:.2}\" x2=\"{x:.2}\" y2=\"{:.2}\"/><rect x=\"{:.2}\" y=\"{body_top:.2}\" width=\"{body_width:.2}\" height=\"{body_height:.2}\"/></g>",
            candle.date,
            candle.date,
            candle.open,
            candle.high,
            candle.low,
            candle.close,
            y(candle.high),
            y(candle.low),
            x - body_width / 2.0
        );
        if index % label_every == 0 {
            let _ = write!(
                svg,
                "<text class=\"axis-label\" x=\"{x:.2}\" y=\"307\" text-anchor=\"middle\">{}</text>",
                candle.date
            );
        }
    }
    svg_end(&mut svg);
    svg
}

fn score_svg(scores: &[SavedScore]) -> String {
    let days = scores
        .first()
        .zip(scores.last())
        .map(|(first, last)| (last.date - first.date).num_days() + 1)
        .unwrap_or(1)
        .max(1);
    let width = MIN_WIDTH.max(LEFT + RIGHT + days as f64 * 12.0);
    let step = (width - LEFT - RIGHT) / days as f64;
    let max_score = scores
        .iter()
        .filter_map(|s| s.score)
        .map(|s| f64::from(s).abs())
        .fold(1.0, f64::max);
    let middle = (TOP + SCORE_BOTTOM) / 2.0;
    let scale = (SCORE_BOTTOM - TOP) / 2.0 / max_score;
    let mut svg = svg_start(
        "score-chart",
        "Zapisane wyniki końcowe rankingu",
        width,
        198.0,
    );
    if scores.is_empty() {
        svg_end(&mut svg);
        return svg;
    }
    for value in [-max_score, 0.0, max_score] {
        let y = middle - value * scale;
        let _ = write!(
            svg,
            "<line class=\"grid-line\" x1=\"{LEFT}\" y1=\"{y:.2}\" x2=\"{}\" y2=\"{y:.2}\"/><text class=\"axis-label\" x=\"62\" y=\"{:.2}\" text-anchor=\"end\">{value:.0}</text>",
            width - RIGHT,
            y + 5.0
        );
        let _ = write!(
            svg,
            "<text class=\"axis-label\" x=\"{:.2}\" y=\"{:.2}\">{value:.0}</text>",
            width - RIGHT + 10.0,
            y + 5.0
        );
    }
    if let Some(first) = scores.first() {
        for saved in scores {
            let Some(value) = saved.score else { continue };
            let x = LEFT + ((saved.date - first.date).num_days() as f64 + 0.5) * step;
            let y = middle - f64::from(value) * scale;
            let bar_width = (step * 0.65).min(24.0);
            if value == 0 {
                let _ = write!(
                    svg,
                    "<line class=\"score-zero\" data-date=\"{}\" data-score=\"0\" x1=\"{:.2}\" y1=\"{middle:.2}\" x2=\"{:.2}\" y2=\"{middle:.2}\"><title>{}: 0</title></line>",
                    saved.date,
                    x - bar_width / 2.0,
                    x + bar_width / 2.0,
                    saved.date
                );
            } else {
                let class = if value > 0 { "positive" } else { "negative" };
                let _ = write!(
                    svg,
                    "<rect class=\"score-bar {class}\" data-date=\"{}\" data-score=\"{value}\" x=\"{:.2}\" y=\"{:.2}\" width=\"{bar_width:.2}\" height=\"{:.2}\"><title>{}: {value}</title></rect>",
                    saved.date,
                    x - bar_width / 2.0,
                    y.min(middle),
                    (y - middle).abs(),
                    saved.date
                );
            }
        }
        // The score axis has its own date range; it is never stretched back to the price archive.
        let label_every = (160.0 / step).ceil().max(1.0) as usize;
        for offset in (0..days).step_by(label_every) {
            if let Some(date) = first
                .date
                .checked_add_signed(chrono::Duration::days(offset))
            {
                let x = LEFT + (offset as f64 + 0.5) * step;
                let _ = write!(
                    svg,
                    "<text class=\"axis-label\" x=\"{x:.2}\" y=\"183\" text-anchor=\"middle\">{date}</text>"
                );
            }
        }
    }
    svg_end(&mut svg);
    svg
}

fn number(value: Decimal) -> f64 {
    // Every finite Decimal fits in f64's range; conversion affects SVG coordinates only.
    value.to_f64().expect("Decimal fits in f64")
}
