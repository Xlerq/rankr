//! Standalone SVG geometry. Floating point is used only for drawing, never scoring.

use std::fmt::Write;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::{company_archive::SavedScore, page::escape};

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

const LEFT: f64 = 72.0;
const RIGHT: f64 = 84.0;
const PRICE_TOP: f64 = 48.0;
const PRICE_BOTTOM: f64 = 280.0;
const SCORE_TOP: f64 = 350.0;
const SCORE_BOTTOM: f64 = 486.0;
const MIN_WIDTH: f64 = 860.0;

/// Calendar days are shared by candles and saved rankings, including non-session days.
struct DateAxis {
    start: NaiveDate,
    days: i64,
    width: f64,
    step: f64,
}

impl DateAxis {
    fn new(dates: impl Iterator<Item = NaiveDate>) -> Option<Self> {
        let (start, end) = dates.fold(None, |range, date| match range {
            None => Some((date, date)),
            Some((start, end)) => Some((date.min(start), date.max(end))),
        })?;
        let days = (end - start).num_days() + 1;
        let width = MIN_WIDTH.max(LEFT + RIGHT + days as f64 * 6.0);
        Some(Self {
            start,
            days,
            width,
            step: (width - LEFT - RIGHT) / days as f64,
        })
    }

    fn x(&self, date: NaiveDate) -> f64 {
        LEFT + ((date - self.start).num_days() as f64 + 0.5) * self.step
    }

    fn draw(&self, svg: &mut String, y: f64) {
        let every = (160.0 / self.step).ceil().max(1.0) as usize;
        for offset in (0..self.days).step_by(every) {
            if let Some(date) = self
                .start
                .checked_add_signed(chrono::Duration::days(offset))
            {
                let _ = write!(
                    svg,
                    "<text class=\"axis-label\" x=\"{:.2}\" y=\"{y}\" text-anchor=\"middle\">{date}</text>",
                    self.x(date)
                );
            }
        }
    }
}

pub(crate) fn render(candles: &[OhlcCandle], currency: &str, scores: &[SavedScore]) -> String {
    let mut html =
        String::from("<section class=\"chart-panel\"><h2>Historia ceny i zapisanych ocen</h2>");
    if candles.is_empty() {
        html.push_str("<p class=\"missing\">Brak zapisanej historii OHLC.</p>");
    } else {
        let _ = write!(
            html,
            "<p class=\"source-note\">Ceny: {} — {} · {} sesji · {}. Przewiń wykres poziomo, aby zobaczyć wcześniejsze daty.</p>",
            candles.first().unwrap().date,
            candles.last().unwrap().date,
            candles.len(),
            escape(currency)
        );
    }
    if scores.is_empty() {
        html.push_str(
            "<p class=\"missing\">Brak zapisanych snapshotów rankingu dla tej spółki.</p>",
        );
    } else {
        let _ = write!(
            html,
            "<p class=\"source-note\">Oceny: {} — {} · ostatni zapis z każdego dnia UTC. Wspólna oś dat ze świecami; kreska oznacza zapisane zero. Dni bez wyniku i lata przed pierwszym zapisem pozostają bez słupków.</p>",
            scores.first().unwrap().date,
            scores.last().unwrap().date
        );
        if scores.iter().all(|score| score.score.is_none()) {
            html.push_str(
                "<p class=\"missing\">Zapisane snapshoty nie zawierają dostępnego final_score.</p>",
            );
        }
    }
    if let Some(axis) = DateAxis::new(
        candles
            .iter()
            .map(|c| c.date)
            .chain(scores.iter().map(|s| s.date)),
    ) {
        let mut svg = svg_start(
            "price-score-chart",
            "Świece OHLC i zapisane oceny końcowe na wspólnej osi dat",
            axis.width,
            548.0,
        );
        svg.push_str("<text class=\"panel-label\" x=\"72\" y=\"26\">Cena · świece OHLC</text><g class=\"price-chart\">");
        draw_prices(&mut svg, candles, &axis);
        svg.push_str("</g><text class=\"panel-label\" x=\"72\" y=\"326\">Zapisana ocena końcowa · final_score</text><g class=\"score-chart\">");
        draw_scores(&mut svg, scores, &axis);
        svg.push_str("</g><g class=\"time-axis\">");
        axis.draw(&mut svg, 524.0);
        svg.push_str("</g>");
        svg_end(&mut svg);
        html.push_str(&svg);
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

fn horizontal_tick(svg: &mut String, width: f64, y: f64, value: &str) {
    let _ = write!(
        svg,
        "<line class=\"grid-line\" x1=\"{LEFT}\" y1=\"{y:.2}\" x2=\"{:.2}\" y2=\"{y:.2}\"/><text class=\"axis-label\" x=\"62\" y=\"{:.2}\" text-anchor=\"end\">{value}</text><text class=\"axis-label\" x=\"{:.2}\" y=\"{:.2}\">{value}</text>",
        width - RIGHT,
        y + 5.0,
        width - RIGHT + 10.0,
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
        let level = PRICE_TOP + fraction * (PRICE_BOTTOM - PRICE_TOP);
        let value = top - fraction * (top - bottom);
        horizontal_tick(svg, axis.width, level, &format!("{value:.2}"));
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
        let body_width = (axis.step * 0.65).min(12.0);
        let _ = write!(
            svg,
            "<g class=\"candle {class}\" data-date=\"{}\" data-x=\"{x:.2}\"><title>{}: O {} · H {} · L {} · C {}</title><line x1=\"{x:.2}\" y1=\"{:.2}\" x2=\"{x:.2}\" y2=\"{:.2}\"/><rect x=\"{:.2}\" y=\"{body_top:.2}\" width=\"{body_width:.2}\" height=\"{body_height:.2}\"/></g>",
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
        horizontal_tick(
            svg,
            axis.width,
            middle - value * scale,
            &format!("{value:.0}"),
        );
    }
    for saved in scores {
        let Some(value) = saved.score else { continue };
        let x = axis.x(saved.date);
        let y = middle - f64::from(value) * scale;
        let bar_width = (axis.step * 0.65).min(24.0);
        if value == 0 {
            let _ = write!(
                svg,
                "<line class=\"score-zero\" data-date=\"{}\" data-score=\"0\" data-x=\"{x:.2}\" x1=\"{:.2}\" y1=\"{middle:.2}\" x2=\"{:.2}\" y2=\"{middle:.2}\"><title>{}: 0</title></line>",
                saved.date,
                x - bar_width / 2.0,
                x + bar_width / 2.0,
                saved.date
            );
        } else {
            let class = if value > 0 { "positive" } else { "negative" };
            let _ = write!(
                svg,
                "<rect class=\"score-bar {class}\" data-date=\"{}\" data-score=\"{value}\" data-x=\"{x:.2}\" x=\"{:.2}\" y=\"{:.2}\" width=\"{bar_width:.2}\" height=\"{:.2}\"><title>{}: {value}</title></rect>",
                saved.date,
                x - bar_width / 2.0,
                y.min(middle),
                (y - middle).abs(),
                saved.date
            );
        }
    }
}

pub(crate) fn render_shorts(totals: &[ShortTotal]) -> String {
    if totals.is_empty() {
        return "<p class=\"missing\">brak zapisanych odczytów rejestru</p>".into();
    }
    let mut html = String::from(
        "<section class=\"short-history\"><h3>Suma ujawnionych shortów KNF</h3><p class=\"source-note\">Suma procentów każdego zapisanego odczytu. Zero oznacza brak publicznego wpisu.</p>",
    );
    let first = totals.first().unwrap().fetched_at;
    let last = totals.last().unwrap().fetched_at;
    let elapsed = (last - first).num_milliseconds() as f64;
    let width = MIN_WIDTH.max(LEFT + RIGHT + (last - first).num_days() as f64 * 12.0);
    let max_total = totals
        .iter()
        .map(|point| number(point.total))
        .fold(0.0, f64::max)
        .max(0.1);
    let y = |total: Decimal| PRICE_BOTTOM - number(total) / max_total * (PRICE_BOTTOM - PRICE_TOP);
    let x = |at: DateTime<Utc>| {
        if elapsed == 0.0 {
            (LEFT + width - RIGHT) / 2.0
        } else {
            LEFT + (at - first).num_milliseconds() as f64 / elapsed * (width - LEFT - RIGHT)
        }
    };
    let mut svg = svg_start(
        "shorts-chart",
        "Suma ujawnionych shortów KNF w kolejnych odczytach",
        width,
        322.0,
    );
    for tick in 0..=4 {
        let value = f64::from(tick) / 4.0 * max_total;
        let level = PRICE_BOTTOM - f64::from(tick) / 4.0 * (PRICE_BOTTOM - PRICE_TOP);
        horizontal_tick(&mut svg, width, level, &format!("{value:.2}%"));
    }
    if totals.len() > 1 {
        let mut points = String::new();
        for total in totals {
            let _ = write!(points, "{:.2},{:.2} ", x(total.fetched_at), y(total.total));
        }
        let _ = write!(svg, "<polyline class=\"short-line\" points=\"{points}\"/>");
    }
    for total in totals {
        let _ = write!(
            svg,
            "<circle class=\"short-point\" data-fetched-at=\"{}\" data-total=\"{}\" cx=\"{:.2}\" cy=\"{:.2}\" r=\"3\"><title>{}: {}%</title></circle>",
            total.fetched_at.to_rfc3339(),
            total.total.normalize(),
            x(total.fetched_at),
            y(total.total),
            total.fetched_at,
            total.total.normalize()
        );
    }
    for tick in 0..=4 {
        if tick > 0 && elapsed == 0.0 {
            break;
        }
        let at = first + chrono::Duration::milliseconds((elapsed * f64::from(tick) / 4.0) as i64);
        let label = if (last - first).num_days() < 2 {
            at.format("%Y-%m-%d %H:%M").to_string()
        } else {
            at.format("%Y-%m-%d").to_string()
        };
        let anchor = if tick == 0 {
            "start"
        } else if tick == 4 {
            "end"
        } else {
            "middle"
        };
        let _ = write!(
            svg,
            "<text class=\"axis-label\" x=\"{:.2}\" y=\"307\" text-anchor=\"{anchor}\">{label}</text>",
            x(at)
        );
    }
    svg_end(&mut svg);
    html.push_str(&svg);
    html.push_str("</section>");
    html
}

fn number(value: Decimal) -> f64 {
    // Every finite Decimal fits in f64's range; conversion affects SVG coordinates only.
    value.to_f64().expect("Decimal fits in f64")
}
