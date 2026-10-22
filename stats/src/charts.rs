//! Hand-drawn SVG charts.
//!
//! No chart library: every chart here is a handful of rects, a polyline and a
//! few labels, so a dependency would cost more than it saves. The geometry is
//! computed by plain functions that take numbers and return coordinates, which
//! keeps the drawing testable without a browser.

use leptos::prelude::*;

use crate::format::{compact, pct, thousands};
use crate::{DailyPoint, HourlyPoint};

/// Chart box, in SVG user units. The element scales to its container, so these
/// numbers only set the aspect ratio and the padding budget.
const W: f64 = 640.0;
const H: f64 = 220.0;
/// Room on the left for the axis labels and at the bottom for the dates.
const PAD_L: f64 = 46.0;
const PAD_R: f64 = 10.0;
const PAD_T: f64 = 10.0;
const PAD_B: f64 = 26.0;

/// Plot area width, after the label gutters are removed.
fn plot_w() -> f64 {
    W - PAD_L - PAD_R
}

/// Plot area height, after the label gutters are removed.
fn plot_h() -> f64 {
    H - PAD_T - PAD_B
}

/// Largest value in a series, floored at one so an all-zero day still draws an
/// empty baseline instead of dividing by zero.
fn scale_max(values: &[u64]) -> u64 {
    values.iter().copied().max().unwrap_or(0).max(1)
}

/// Height in pixels for a value, clamped to the plot area.
fn bar_height(value: u64, max: u64) -> f64 {
    if max == 0 {
        return 0.0;
    }
    let h = (value as f64 / max as f64) * plot_h();
    h.clamp(0.0, plot_h())
}

/// Left edge of the slot a bar occupies, and the width of the bar inside it.
///
/// A single point fills the plot, and many points still leave a hairline gap
/// so a dense range reads as bars rather than a solid block.
fn slot(index: usize, count: usize) -> (f64, f64) {
    if count == 0 {
        return (PAD_L, 0.0);
    }
    let slot_w = plot_w() / count as f64;
    let gap = if count > 1 {
        (slot_w * 0.18).min(3.0)
    } else {
        0.0
    };
    (PAD_L + slot_w * index as f64, (slot_w - gap).max(0.6))
}

/// Y coordinate of a point on the value axis.
fn y_for(value: u64, max: u64) -> f64 {
    PAD_T + plot_h() - bar_height(value, max)
}

/// `MM-DD` from an ISO date, for the x axis.
fn short_date(date: &str) -> String {
    match date.get(5..10) {
        Some(d) => d.to_string(),
        None => date.to_string(),
    }
}

/// A legend swatch plus its label, one entry per series.
#[component]
fn Legend(items: Vec<(&'static str, String)>) -> impl IntoView {
    view! {
        <div class="stats-legend">
            {items.into_iter().map(|(color, label)| view! {
                <span>
                    <i style=format!("background:{color}")></i>
                    {label}
                </span>
            }).collect_view()}
        </div>
    }
}

/// Horizontal gridlines with their value labels.
#[component]
fn Grid(max: u64, format: fn(u64) -> String) -> impl IntoView {
    // Four lines: the baseline, two thirds, one third and the top.
    let steps = [0.0, 1.0 / 3.0, 2.0 / 3.0, 1.0];
    view! {
        <g>
            {steps.into_iter().map(|f| {
                let y = PAD_T + plot_h() - plot_h() * f;
                let label_y = y + 3.5;
                let value = (max as f64 * f) as u64;
                view! {
                    <line x1=PAD_L x2={W - PAD_R} y1=y y2=y
                        stroke="rgba(255,255,255,0.09)" stroke-width="1"/>
                    <text x={PAD_L - 8.0} y=label_y
                        text-anchor="end" font-size="10" fill="rgba(255,255,255,0.45)">
                        {format(value)}
                    </text>
                }
            }).collect_view()}
        </g>
    }
}

/// The x axis: the first, middle and last dates of the range.
#[component]
fn DateAxis(points: Vec<DailyPoint>) -> impl IntoView {
    let count = points.len();
    let marks = if count == 0 {
        Vec::new()
    } else {
        let mut v = vec![0];
        if count > 2 {
            v.push(count / 2);
        }
        if count > 1 {
            v.push(count - 1);
        }
        v
    };
    view! {
        <g>
            {marks.into_iter().map(|i| {
                let (x, width) = slot(i, count);
                let label = points.get(i).map(|p| short_date(&p.date)).unwrap_or_default();
                let cx = x + width / 2.0;
                view! {
                    <text x=cx y={H - 8.0} text-anchor="middle"
                        font-size="10" fill="rgba(255,255,255,0.45)">{label}</text>
                }
            }).collect_view()}
        </g>
    }
}

/// Requests per day.
#[component]
pub fn RequestsBars(points: Vec<DailyPoint>) -> impl IntoView {
    let values: Vec<u64> = points.iter().map(|p| p.requests).collect();
    let max = scale_max(&values);
    let count = values.len();
    let total: u64 = values.iter().sum();
    let peak = values.iter().copied().max().unwrap_or(0);
    let axis = points.clone();

    view! {
        <svg viewBox=format!("0 0 {W} {H}") role="img"
            aria-label=format!("Requests per day, {} in total, peak {}", thousands(total), thousands(peak))>
            <Grid max=max format=compact/>
            {points.into_iter().enumerate().map(|(i, p)| {
                let (x, w) = slot(i, count);
                let h = bar_height(p.requests, max);
                let y = PAD_T + plot_h() - h;
                view! {
                    <rect x=x y=y width=w height=h fill="#d76414" rx="1.5">
                        <title>{format!("{} requests", thousands(p.requests))}</title>
                    </rect>
                }
            }).collect_view()}
            <DateAxis points=axis/>
        </svg>
        <Legend items=vec![("#d76414", "requests".to_string())]/>
    }
}

/// Input and output tokens per day, stacked.
#[component]
pub fn TokenBars(points: Vec<DailyPoint>) -> impl IntoView {
    let inputs: Vec<u64> = points.iter().map(|p| p.input_tokens).collect();
    let outputs: Vec<u64> = points.iter().map(|p| p.output_tokens).collect();
    // One shared scale for both series, otherwise the output bars would look
    // as tall as the input bars and the chart would lie about the ratio.
    let max = scale_max(&inputs).max(scale_max(&outputs));
    let count = points.len();
    let total_in: u64 = inputs.iter().sum();
    let total_out: u64 = outputs.iter().sum();
    let axis = points.clone();

    view! {
        <svg viewBox=format!("0 0 {W} {H}") role="img"
            aria-label=format!("Tokens per day, {} in and {} out", compact(total_in), compact(total_out))>
            <Grid max=max format=compact/>
            {points.into_iter().enumerate().map(|(i, p)| {
                let (x, w) = slot(i, count);
                let h_in = bar_height(p.input_tokens, max);
                let h_out = bar_height(p.output_tokens, max);
                let y_in = PAD_T + plot_h() - h_in;
                let y_out = y_in - h_out;
                view! {
                    <rect x=x y=y_in width=w height=h_in fill="#d76414" rx="1.5">
                        <title>{format!("{} input", compact(p.input_tokens))}</title>
                    </rect>
                    <rect x=x y=y_out width=w height=h_out fill="#f0a35e" rx="1.5">
                        <title>{format!("{} output", compact(p.output_tokens))}</title>
                    </rect>
                }
            }).collect_view()}
            <DateAxis points=axis/>
        </svg>
        <Legend items=vec![("#d76414", "input".to_string()), ("#f0a35e", "output".to_string())]/>
    }
}

/// Grid label for a percentage axis.
fn pct_label(v: u64) -> String {
    format!("{v}%")
}

/// Cache hit rate per day, as a line with the points marked.
#[component]
pub fn CacheLine(points: Vec<DailyPoint>) -> impl IntoView {
    let count = points.len();
    // Percentages, so the axis is fixed at 0..100 and a good day is visibly
    // good rather than relative to the worst day in the window.
    let max = 100u64;
    let coords: Vec<(f64, f64, f64)> = points
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let denom = p.input_tokens + p.cached_tokens;
            let hit = if denom == 0 {
                0.0
            } else {
                p.cached_tokens as f64 * 100.0 / denom as f64
            };
            let (x, w) = slot(i, count);
            (x + w / 2.0, y_for(hit.round() as u64, max), hit)
        })
        .collect();

    let path = coords
        .iter()
        .map(|(x, y, _)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ");
    let latest = coords.last().map(|(_, _, v)| *v).unwrap_or(0.0);
    let axis = points.clone();

    view! {
        <svg viewBox=format!("0 0 {W} {H}") role="img"
            aria-label=format!("Cache hit rate per day, latest {}", pct(latest))>
            <Grid max=max format=pct_label/>
            {if count > 1 {
                Some(view! {
                    <polyline points=path fill="none" stroke="#6fd08c"
                        stroke-width="2" stroke-linejoin="round"/>
                })
            } else {
                None
            }}
            {coords.into_iter().map(|(x, y, v)| view! {
                <circle cx=x cy=y r="3" fill="#6fd08c">
                    <title>{pct(v)}</title>
                </circle>
            }).collect_view()}
            <DateAxis points=axis/>
        </svg>
        <Legend items=vec![("#6fd08c", "cache hit rate".to_string())]/>
    }
}

/// Requests by hour of day, lifetime, in UTC.
#[component]
pub fn HourBars(points: Vec<HourlyPoint>) -> impl IntoView {
    let mut values = [0u64; 24];
    for p in &points {
        if let Some(bucket) = values.get_mut(p.hour as usize) {
            *bucket = p.requests;
        }
    }
    let max = scale_max(&values);
    let total: u64 = values.iter().sum();
    let peak_hour = values
        .iter()
        .enumerate()
        .max_by_key(|(_, v)| **v)
        .map(|(h, _)| h)
        .unwrap_or(0);

    view! {
        <svg viewBox=format!("0 0 {W} {H}") role="img"
            aria-label=format!("Requests by hour, busiest at {peak_hour}:00 UTC")>
            <Grid max=max format=compact/>
            {values.into_iter().enumerate().map(|(hour, v)| {
                let (x, w) = slot(hour, 24);
                let h = bar_height(v, max);
                let y = PAD_T + plot_h() - h;
                view! {
                    <rect x=x y=y width=w height=h fill="#d76414" rx="1.5">
                        <title>{format!("{hour:02}:00 UTC, {} requests", thousands(v))}</title>
                    </rect>
                }
            }).collect_view()}
            {[0usize, 6, 12, 18, 23].into_iter().map(|hour| {
                let (x, w) = slot(hour, 24);
                let cx = x + w / 2.0;
                view! {
                    <text x=cx y={H - 8.0} text-anchor="middle"
                        font-size="10" fill="rgba(255,255,255,0.45)">
                        {format!("{hour:02}")}
                    </text>
                }
            }).collect_view()}
        </svg>
        <Legend items=vec![("#d76414", format!("requests, {} total", thousands(total)))]/>
    }
}

/// One bar per day for a single count series: distinct IPs or distinct clients.
///
/// The two charts differ only in which field they read and what they say, so
/// they share this builder instead of being copy-pasted and drifting apart.
fn count_bars(
    points: Vec<DailyPoint>,
    pick: fn(&DailyPoint) -> u64,
    color: &'static str,
    legend: &'static str,
    aria: &'static str,
) -> impl IntoView {
    let values: Vec<u64> = points.iter().map(pick).collect();
    let max = scale_max(&values);
    let count = values.len();
    let peak = values.iter().copied().max().unwrap_or(0);
    let axis = points.clone();

    view! {
        <svg viewBox=format!("0 0 {W} {H}") role="img"
            aria-label=format!("{aria}, peak {}", thousands(peak))>
            <Grid max=max format=compact/>
            {points.into_iter().enumerate().map(|(i, p)| {
                let value = pick(&p);
                let (x, w) = slot(i, count);
                let h = bar_height(value, max);
                let y = PAD_T + plot_h() - h;
                view! {
                    <rect x=x y=y width=w height=h fill=color rx="1.5">
                        <title>{format!("{} {}", thousands(value), legend)}</title>
                    </rect>
                }
            }).collect_view()}
            <DateAxis points=axis/>
        </svg>
        <Legend items=vec![(color, format!("{legend}, peak {}", thousands(peak)))]/>
    }
}

/// Distinct caller IPs per day.
#[component]
pub fn UniqueIpsBars(points: Vec<DailyPoint>) -> impl IntoView {
    count_bars(
        points,
        |p| p.unique_ips,
        "#3b6ea8",
        "unique IPs",
        "Unique caller IPs per day",
    )
}

/// Distinct client labels per day.
#[component]
pub fn UniqueClientsBars(points: Vec<DailyPoint>) -> impl IntoView {
    count_bars(
        points,
        |p| p.unique_clients,
        "#5aa9a3",
        "tracked users",
        "Tracked users per day",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_series_still_has_a_scale() {
        assert_eq!(scale_max(&[]), 1);
        assert_eq!(scale_max(&[0, 0, 0]), 1);
        assert_eq!(scale_max(&[3, 9, 1]), 9);
    }

    #[test]
    fn bar_height_is_proportional_and_clamped() {
        let full = plot_h();
        assert_eq!(bar_height(0, 10), 0.0);
        assert_eq!(bar_height(10, 10), full);
        assert_eq!(bar_height(5, 10), full / 2.0);
        // A value above the stated max must not overflow the plot.
        assert_eq!(bar_height(20, 10), full);
        assert_eq!(bar_height(5, 0), 0.0);
    }

    #[test]
    fn slots_cover_the_plot_without_overlap() {
        let (x0, w0) = slot(0, 4);
        let (x1, _) = slot(1, 4);
        assert_eq!(x0, PAD_L);
        assert!(w0 > 0.0);
        assert!(x1 >= x0 + w0, "slots must not overlap");
        let (x_last, w_last) = slot(3, 4);
        assert!(x_last + w_last <= PAD_L + plot_w() + 0.001);
        // A single point fills the width.
        let (_, only) = slot(0, 1);
        assert_eq!(only, plot_w());
    }

    #[test]
    fn short_date_trims_to_month_and_day() {
        assert_eq!(short_date("2026-10-19"), "10-19");
        assert_eq!(short_date("bad"), "bad");
    }

    #[test]
    fn caller_count_series_scale_independently_of_requests() {
        let day = |requests, ips, clients| DailyPoint {
            date: "2026-10-19".to_string(),
            requests,
            input_tokens: 0,
            output_tokens: 0,
            cached_tokens: 0,
            median_latency_ms: 0,
            unique_ips: ips,
            unique_clients: clients,
        };
        let points = [day(9_000, 3, 2), day(500, 40, 12)];
        let ips: Vec<u64> = points.iter().map(|p| p.unique_ips).collect();
        let clients: Vec<u64> = points.iter().map(|p| p.unique_clients).collect();
        // The busiest request day is the quietest caller day, so a chart scaled
        // off requests would draw these bars inside out.
        assert_eq!(scale_max(&ips), 40);
        assert_eq!(scale_max(&clients), 12);
        assert!(bar_height(ips[1], scale_max(&ips)) > bar_height(ips[0], scale_max(&ips)));
    }

    #[test]
    fn a_full_day_sits_at_the_top_of_the_plot() {
        assert_eq!(y_for(100, 100), PAD_T);
        assert_eq!(y_for(0, 100), PAD_T + plot_h());
    }
}
