//! SVG geometry only. Scoring continues to use Decimal and its existing formulas.

use std::fmt::Write;

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::{Decimal, prelude::ToPrimitive};

use crate::{company_archive::SavedScore, page::escape, weekly_shorts::WeeklyShort};

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
    let mut html = String::from("<section class=\"chart-panel\">");
    if candles.is_empty() {
        html.push_str("<p class=\"missing\">Brak zapisanej historii OHLC.</p>");
    }
    if let Some(axis) = DateAxis::new(
        candles
            .iter()
            .map(|c| c.date)
            .chain(scores.iter().map(|s| s.date)),
    ) {
        let mut svg = svg_start("price-score-chart", "Cena i zapisane wyniki", 900.0);
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
        svg.push_str("</g>");
        svg_end(&mut svg);
        html.push_str(&svg);
    }
    html.push_str("</section>");
    html
}

fn svg_start(class: &str, label: &str, height: f64) -> String {
    format!(
        "<div class=\"plot\"><svg xmlns=\"http://www.w3.org/2000/svg\" class=\"{class}\" role=\"img\" aria-label=\"{label}\" width=\"100%\" height=\"{height}\">"
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
            "<g class=\"candle {class}\" data-date=\"{}\" data-x=\"{x:.6}\"><title>{}: O {} · H {} · L {} · C {}</title><line x1=\"{x:.6}%\" y1=\"{:.2}\" x2=\"{x:.6}%\" y2=\"{:.2}\"/><rect x=\"{:.6}%\" y=\"{body_top:.2}\" width=\"{body_width:.6}%\" height=\"{body_height:.2}\"/></g>",
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

pub(crate) fn render_shorts(weeks: &[WeeklyShort]) -> String {
    let Some(axis) = DateAxis::new(weeks.iter().map(|week| week.week)) else {
        return String::new();
    };
    let max_total = weeks
        .iter()
        .map(|week| number(week.total))
        .fold(0.0, f64::max)
        .max(0.1);
    let y = |total: Decimal| 660.0 - number(total) / max_total * 612.0;
    let mut svg = String::from("<section class=\"chart-panel\">");
    svg.push_str(&svg_start("shorts-chart", "Tygodniowa suma shortów", 720.0));
    svg.push_str("<text class=\"panel-label\" x=\"6%\" y=\"26\">Suma (%)</text>");
    for tick in 0..=4 {
        let value = f64::from(tick) / 4.0 * max_total;
        horizontal_tick(
            &mut svg,
            660.0 - f64::from(tick) / 4.0 * 612.0,
            &format!("{value:.2}%"),
        );
    }
    for pair in weeks.windows(2) {
        let _ = write!(
            svg,
            "<line class=\"short-line\" x1=\"{:.6}%\" y1=\"{:.2}\" x2=\"{:.6}%\" y2=\"{:.2}\"/>",
            axis.x(pair[0].week),
            y(pair[0].total),
            axis.x(pair[1].week),
            y(pair[1].total)
        );
    }
    for week in weeks {
        let _ = write!(
            svg,
            "<circle class=\"short-point\" data-week=\"{}\" data-total=\"{}\" cx=\"{:.6}%\" cy=\"{:.2}\" r=\"3\"><title>{}: {}%</title></circle>",
            week.week,
            week.total.normalize(),
            axis.x(week.week),
            y(week.total),
            week.week,
            week.total.normalize()
        );
    }
    axis.draw(&mut svg, 700.0);
    svg_end(&mut svg);
    svg.push_str("</section>");
    svg
}

fn number(value: Decimal) -> f64 {
    value.to_f64().expect("Decimal fits in f64")
}
