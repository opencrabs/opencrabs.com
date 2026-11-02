//! Public usage dashboard for the OpenCrabs inference provider.
//!
//! The page renders one static JSON snapshot that a timer regenerates every
//! five minutes. It is deliberately dumb: fetch, parse, draw. Everything it
//! shows is already aggregated, and it has no way to ask for anything else.

use leptos::prelude::*;
use serde::Deserialize;

mod charts;
mod format;

use charts::{CacheLine, HourBars, RequestsBars, TokenBars, UniqueClientsBars, UniqueIpsBars};
use format::{compact, pct, pct2, thousands, uptime};

/// Snapshot schema this build understands. A newer file is refused rather than
/// half-rendered, because a renamed field would otherwise show as a zero.
///
/// Schema 3 added the uptime and truncation shares. They arrive here as
/// `Option<f64>` with `serde(default)`, so a schema 2 file still renders and
/// simply hides the uptime card until the proxy's timer catches up. That is
/// what makes the page safe to deploy BEFORE the proxy.
const SCHEMA: u32 = 3;

/// Oldest schema still readable. Schema 1 predates the distinct-caller counts,
/// so a schema 1 file renders without them instead of failing outright. That
/// keeps the page up while the proxy's timer catches up after a deploy.
const MIN_SCHEMA: u32 = 1;

/// A snapshot older than this is reported as stale. The timer runs every five
/// minutes, so this tolerates several missed runs before crying wolf.
const STALE_AFTER_MINUTES: i64 = 30;

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Snapshot {
    pub schema: u32,
    pub generated_at: String,
    pub history_since: Option<String>,
    pub totals: Totals,
    pub windows: Windows,
    pub daily: Vec<DailyPoint>,
    pub hourly: Vec<HourlyPoint>,
    pub capabilities: Vec<CapabilityCount>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Totals {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub cache_hit_pct: f64,
    pub reasoning_tokens: u64,
    pub median_latency_ms: u64,
    /// Distinct caller IPs, a count only. Absent in schema 1.
    #[serde(default)]
    pub unique_ips: u64,
    /// Distinct client labels the proxy tracks, a count only. One label can be
    /// used from many addresses, so this reads lower than the IP count.
    #[serde(default)]
    pub unique_clients: u64,
    /// Share of requests answered without a server-side failure, 0 to 100.
    /// Absent in schema 2.
    #[serde(default)]
    pub uptime_pct: Option<f64>,
    /// Share of requests whose response body was cut short, 0 to 100.
    #[serde(default)]
    pub truncated_pct: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Window {
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub cache_hit_pct: f64,
    pub median_latency_ms: u64,
    #[serde(default)]
    pub unique_ips: u64,
    #[serde(default)]
    pub unique_clients: u64,
    /// Absent in schema 2.
    #[serde(default)]
    pub uptime_pct: Option<f64>,
    #[serde(default)]
    pub truncated_pct: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Windows {
    pub h24: Window,
    pub d7: Window,
    pub d30: Window,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct DailyPoint {
    pub date: String,
    pub requests: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub median_latency_ms: u64,
    #[serde(default)]
    pub unique_ips: u64,
    #[serde(default)]
    pub unique_clients: u64,
    /// Absent in schema 2.
    #[serde(default)]
    pub uptime_pct: Option<f64>,
    #[serde(default)]
    pub truncated_pct: Option<f64>,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct HourlyPoint {
    pub hour: u32,
    pub requests: u64,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct CapabilityCount {
    pub kind: String,
    pub requests: u64,
}

/// Which slice of `daily` the charts show.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Range {
    Day,
    Week,
    Month,
    All,
}

impl Range {
    /// How many trailing daily points this range covers.
    fn days(self, total: usize) -> usize {
        match self {
            Range::Day => 1,
            Range::Week => 7,
            Range::Month => 30,
            Range::All => total,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Range::Day => "24h",
            Range::Week => "7d",
            Range::Month => "30d",
            Range::All => "all",
        }
    }

    /// The five headline numbers for this range. `All` reads the lifetime
    /// totals, the rest read their own rolling window, so the cards and the
    /// charts always describe the same slice of time.
    fn metrics(self, snap: &Snapshot) -> Metrics {
        match self {
            Range::All => Metrics {
                requests: snap.totals.requests,
                input_tokens: snap.totals.input_tokens,
                output_tokens: snap.totals.output_tokens,
                cached_tokens: snap.totals.cached_tokens,
                cache_hit_pct: snap.totals.cache_hit_pct,
                median_latency_ms: snap.totals.median_latency_ms,
                reasoning_tokens: Some(snap.totals.reasoning_tokens),
                unique_ips: snap.totals.unique_ips,
                unique_clients: snap.totals.unique_clients,
                uptime_pct: snap.totals.uptime_pct,
                truncated_pct: snap.totals.truncated_pct,
            },
            other => {
                let w = match other {
                    Range::Day => &snap.windows.h24,
                    Range::Week => &snap.windows.d7,
                    _ => &snap.windows.d30,
                };
                Metrics {
                    requests: w.requests,
                    input_tokens: w.input_tokens,
                    output_tokens: w.output_tokens,
                    cached_tokens: w.cached_tokens,
                    cache_hit_pct: w.cache_hit_pct,
                    median_latency_ms: w.median_latency_ms,
                    // The windows carry no reasoning split; only totals do.
                    reasoning_tokens: None,
                    unique_ips: w.unique_ips,
                    unique_clients: w.unique_clients,
                    uptime_pct: w.uptime_pct,
                    truncated_pct: w.truncated_pct,
                }
            }
        }
    }

    /// How a card reads the range it is showing.
    fn span(self) -> &'static str {
        match self {
            Range::Day => "in the last 24 hours",
            Range::Week => "in the last 7 days",
            Range::Month => "in the last 30 days",
            Range::All => "lifetime",
        }
    }
}

/// Whether the service is answering callers, judged over the last 24 hours.
///
/// One input only: the share of requests that came back without a server-side
/// failure. Probe results, rotation state and lane health are deliberately not
/// inputs. Two lanes parked on price while rotation still answers every caller
/// is not a caller being failed, and a dashboard that cried degraded on that
/// would be reporting the proxy's internals instead of its service.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ServiceState {
    Operational,
    Degraded,
    Outage,
}

impl ServiceState {
    /// Below this share of served requests the service is degraded, in percent.
    const DEGRADED_BELOW: f64 = 99.5;

    /// Below this share it is an outage rather than degradation, in percent.
    const OUTAGE_BELOW: f64 = 95.0;

    /// Classify a 24 hour uptime share.
    fn of(uptime_pct: f64) -> Self {
        if uptime_pct < Self::OUTAGE_BELOW {
            Self::Outage
        } else if uptime_pct < Self::DEGRADED_BELOW {
            Self::Degraded
        } else {
            Self::Operational
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Operational => "operational",
            Self::Degraded => "degraded",
            Self::Outage => "outage",
        }
    }

    fn headline(self) -> &'static str {
        match self {
            Self::Operational => "Every request is being served",
            Self::Degraded => "Some requests are failing",
            Self::Outage => "Requests are failing at scale",
        }
    }

    /// Border and tint for the notice, kept next to the state so a new state
    /// cannot be added without a colour.
    fn tone(self) -> (&'static str, &'static str) {
        match self {
            Self::Operational => ("var(--border)", "transparent"),
            Self::Degraded => ("#5a4a1a", "rgba(217,180,74,0.07)"),
            Self::Outage => ("#6d3030", "rgba(220,80,80,0.08)"),
        }
    }
}

/// The headline numbers for one range, so the cards can follow the picker.
#[derive(Clone, Debug, Default)]
struct Metrics {
    requests: u64,
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
    cache_hit_pct: f64,
    median_latency_ms: u64,
    reasoning_tokens: Option<u64>,
    /// Distinct caller IPs for the same slice. A count, never the addresses.
    unique_ips: u64,
    /// Distinct client labels for the same slice. A count, never the labels.
    unique_clients: u64,
    /// Share of requests served without a server-side failure, when the
    /// snapshot carries it.
    uptime_pct: Option<f64>,
    /// Share of requests whose body was cut short, when the snapshot has it.
    truncated_pct: Option<f64>,
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let snapshot = LocalResource::new(|| async {
        let resp = gloo_net::http::Request::get("/stats-data/public.json")
            .send()
            .await
            .map_err(|e| format!("could not reach the stats file: {e}"))?;
        if !resp.ok() {
            return Err(format!("the stats file answered {}", resp.status()));
        }
        let snap: Snapshot = resp
            .json()
            .await
            .map_err(|e| format!("the stats file is not valid JSON: {e}"))?;
        if snap.schema > SCHEMA || snap.schema < MIN_SCHEMA {
            return Err(format!(
                "this page understands schema {MIN_SCHEMA} to {SCHEMA}, the file is schema {}",
                snap.schema
            ));
        }
        Ok(snap)
    });

    let range = RwSignal::new(Range::Week);

    // The header and the footer are rendered by the shared `chrome` crate and
    // dropped in as HTML, so this page cannot drift from the landing page.
    //
    // The caller count comes from the snapshot this page has already loaded
    // rather than a second fetch of the same file. A schema 1 file predates the
    // caller counts, so it reads as zero and the pill stays hidden. The star
    // count is left empty and `chrome.js` fills it in from GitHub.
    let counts = Signal::derive(move || chrome::Counts {
        stars: 0,
        callers: snapshot
            .get()
            .and_then(|r| r.ok())
            .filter(|s| s.schema >= 2)
            .map(|s| s.totals.unique_ips)
            .unwrap_or(0),
    });
    let nav = move || {
        chrome::nav_html(&chrome::NavLabels::english(), &counts.get(), None)
    };
    let footer = move || chrome::footer_html(&chrome::FooterLabels::english(), &counts.get());

    view! {
        <div class="chrome-nav-host" inner_html=nav></div>
        <div class="stats-wrap">
            <div class="stats-head">
                <h1><span class="stats-logo">OpenCrabs</span> " provider stats"</h1>
            </div>
            <p class="stats-sub">
                "Live usage of the OpenCrabs inference endpoint. Requests, tokens, cache, latency, "
                "uptime and how many distinct callers showed up, refreshed every five minutes. Uptime "
                "counts a request as served unless it came back with a server-side error. No routing, "
                "upstream or per-caller detail is published, only counts."
            </p>
            <Suspense fallback=|| view! { <Loading/> }>
                {move || Suspend::new(async move {
                    match snapshot.await {
                        Ok(snap) => view! { <Dashboard snap=snap range=range/> }.into_any(),
                        Err(e) => view! { <Failure message=e/> }.into_any(),
                    }
                })}
            </Suspense>
        </div>
        <div inner_html=footer></div>
    }
}

#[component]
fn Loading() -> impl IntoView {
    view! {
        <div class="stats-state">
            <strong>"Loading the snapshot"</strong>
            "Fetching the latest usage numbers."
        </div>
    }
}

#[component]
fn Failure(message: String) -> impl IntoView {
    view! {
        <div class="stats-state">
            <strong>"The snapshot could not be read"</strong>
            {message}
            <p class="stats-foot">"The page retries on the next reload."</p>
        </div>
    }
}

#[component]
fn Dashboard(snap: Snapshot, range: RwSignal<Range>) -> impl IntoView {
    let age_minutes = format::age_minutes(&snap.generated_at);
    let stale = age_minutes > STALE_AFTER_MINUTES;
    let generated = format::stamp(&snap.generated_at);
    let since = snap
        .history_since
        .clone()
        .unwrap_or_else(|| "no data yet".to_string());
    let snap_cards = snap.clone();
    let since_cards = since.clone();
    let caps = snap.capabilities.clone();
    let daily = snap.daily.clone();
    let hourly = snap.hourly.clone();
    // A schema 1 file has no caller counts, so serde fills them with 0.
    // Printing that zero would publish a number nobody measured, so the
    // caller cards and charts only appear once the exporter counts callers.
    let has_callers = snap.schema >= 2;
    // Service state and the uptime card read request outcomes only. A schema 2
    // snapshot carries neither field, so both stay hidden instead of printing a
    // number nobody measured.
    let service = snap.windows.h24.uptime_pct;
    let h24_requests = snap.windows.h24.requests;
    let h24_truncated = snap.windows.h24.truncated_pct;

    view! {
        <div class="stats-meta">
            <span class=if stale { "stats-pill stale" } else { "stats-pill live" }>
                {if stale {
                    format!("stale, updated {age_minutes} min ago")
                } else {
                    format!("updated {age_minutes} min ago")
                }}
            </span>
            <span class="stats-pill">{format!("generated {generated} UTC")}</span>
            <span class="stats-pill">{format!("history since {since}")}</span>
            {service.map(|u| {
                let state = ServiceState::of(u);
                let class = match state {
                    ServiceState::Operational => "stats-pill live",
                    _ => "stats-pill stale",
                };
                view! { <span class=class>{format!("{} {}", state.label(), pct2(u))}</span> }
            })}
        </div>

        {service.map(|u| {
            let state = ServiceState::of(u);
            (state != ServiceState::Operational).then(|| {
                let (border, tint) = state.tone();
                let truncated_note = match h24_truncated {
                    Some(t) if t > 0.0 => format!(
                        " Separately, {} of those requests had their response body cut short, \
                         which a status code cannot show.",
                        pct2(t)
                    ),
                    _ => String::new(),
                };
                view! {
                    <div class="stats-state"
                        style=format!("margin-bottom:26px;text-align:left;padding:24px;border-color:{border};background:{tint}")>
                        <strong>{format!("{}: {}", state.label(), state.headline())}</strong>
                        {format!(
                            "{} of the {} requests in the last 24 hours were answered without a \
                             server-side failure. A 4xx does not count against this: that is the \
                             caller's mistake, not the service's.",
                            pct2(u),
                            thousands(h24_requests),
                        )}
                        {truncated_note}
                    </div>
                }
            })
        })}

        {if stale {
            Some(view! {
                <div class="stats-state" style="margin-bottom:26px">
                    <strong>"This snapshot is stale"</strong>
                    "The timer that regenerates it has not run recently, so these numbers are older than "
                    {STALE_AFTER_MINUTES} " minutes."
                </div>
            })
        } else {
            None
        }}

        {move || {
            let r = range.get();
            let m = r.metrics(&snap_cards);
            let span = if r == Range::All {
                format!("since {since_cards}")
            } else {
                r.span().to_string()
            };
            let reasoning = match m.reasoning_tokens {
                Some(t) => format!("{} reasoning", compact(t)),
                None => span.clone(),
            };
            view! {
                <div class="stats-cards">
                    <Card k="Requests" v=thousands(m.requests) s=span.clone()/>
                    <Card k="Input tokens" v=compact(m.input_tokens)
                        s=format!("{} cached", compact(m.cached_tokens))/>
                    <Card k="Output tokens" v=compact(m.output_tokens) s=reasoning/>
                    <Card k="Cache hit" v=pct(m.cache_hit_pct) s=span.clone()/>
                    <Card k="Median latency" v=uptime(m.median_latency_ms) s=span.clone()/>
                    {m.uptime_pct.map(|u| {
                        // Truncation rides along with uptime because the two
                        // answer the same question from different sides: the
                        // status said yes, the body said no.
                        let s = match m.truncated_pct {
                            Some(t) => format!("{} truncated, {span}", pct2(t)),
                            None => span.clone(),
                        };
                        view! { <Card k="Uptime" v=pct2(u) s=s/> }
                    })}
                    {has_callers.then(|| view! {
                        <Card k="Unique IPs" v=thousands(m.unique_ips) s=span.clone()/>
                        <Card k="Tracked users" v=thousands(m.unique_clients) s=span/>
                    })}
                </div>
            }
        }}

        <div class="stats-cards">
            {caps.into_iter().map(|c| {
                let label = match c.kind.as_str() {
                    "chat" => "Chat requests",
                    "image" => "Image requests",
                    "audio" => "Voice requests",
                    _ => "Other requests",
                };
                view! { <Card k=label v=thousands(c.requests) s="lifetime".to_string()/> }
            }).collect_view()}
        </div>

        <RangePicker range=range/>

        {move || {
            let r = range.get();
            let days = r.days(daily.len());
            let start = daily.len().saturating_sub(days);
            let slice = daily[start..].to_vec();
            let m = r.metrics(&snap);
            let mut totals_line = format!(
                "{} requests, {} in / {} out, {} cache hit, median {}",
                thousands(m.requests),
                compact(m.input_tokens),
                compact(m.output_tokens),
                pct(m.cache_hit_pct),
                uptime(m.median_latency_ms),
            );
            if has_callers {
                totals_line.push_str(&format!(
                    ", {} unique IPs, {} tracked users",
                    thousands(m.unique_ips),
                    thousands(m.unique_clients),
                ));
            }
            view! {
                <div class="stats-meta" style="margin-bottom:18px">
                    <span class="stats-pill">{totals_line}</span>
                </div>
                <div class="stats-charts">
                    <section class="stats-chart">
                        <h2>"Requests per day"</h2>
                        <RequestsBars points=slice.clone()/>
                    </section>
                    <section class="stats-chart">
                        <h2>"Tokens per day"</h2>
                        <TokenBars points=slice.clone()/>
                    </section>
                    {has_callers.then(|| view! {
                        <section class="stats-chart">
                            <h2>"Unique IPs per day"</h2>
                            <UniqueIpsBars points=slice.clone()/>
                        </section>
                        <section class="stats-chart">
                            <h2>"Tracked users per day"</h2>
                            <UniqueClientsBars points=slice.clone()/>
                        </section>
                    })}
                    <section class="stats-chart">
                        <h2>"Cache hit rate per day"</h2>
                        <CacheLine points=slice/>
                    </section>
                    <section class="stats-chart">
                        <h2>"Requests by hour of day (UTC, lifetime)"</h2>
                        <HourBars points=hourly.clone()/>
                    </section>
                </div>
            }
        }}

        <p class="stats-note">
            "Every figure is an aggregate of the last 90 days of proxy records. Uptime is the share of "
            "requests answered without a server-side error, so a 4xx from a misconfigured caller does "
            "not read as downtime; a request whose response body was cut short still counts as served "
            "by status, which is why it is reported separately. Unique IPs and unique clients are "
            "counts only: no address, key or label is ever published. A rotating mobile address can "
            "inflate the IP count and a shared office address can deflate it, so read it as reach "
            "rather than a headcount. Times are UTC."
        </p>
    }
}

#[component]
fn Card(k: &'static str, v: String, s: String) -> impl IntoView {
    view! {
        <div class="stats-card">
            <div class="k">{k}</div>
            <div class="v">{v}</div>
            <div class="s">{s}</div>
        </div>
    }
}

#[component]
fn RangePicker(range: RwSignal<Range>) -> impl IntoView {
    let options = [Range::Day, Range::Week, Range::Month, Range::All];
    view! {
        <div class="stats-ranges">
            {options.into_iter().map(|r| view! {
                <button
                    class=move || if range.get() == r { "on" } else { "" }
                    on:click=move |_| range.set(r)
                >{r.label()}</button>
            }).collect_view()}
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_state_reads_the_served_share() {
        // A day with no failures at all.
        assert_eq!(ServiceState::of(100.0), ServiceState::Operational);
        // The boundary belongs to the better state: exactly on the threshold is
        // not a failure yet.
        assert_eq!(
            ServiceState::of(ServiceState::DEGRADED_BELOW),
            ServiceState::Operational
        );
        assert_eq!(
            ServiceState::of(ServiceState::DEGRADED_BELOW - 0.01),
            ServiceState::Degraded
        );
        assert_eq!(
            ServiceState::of(ServiceState::OUTAGE_BELOW),
            ServiceState::Degraded
        );
        assert_eq!(
            ServiceState::of(ServiceState::OUTAGE_BELOW - 0.01),
            ServiceState::Outage
        );
        // The live 24h figure at the time of writing: serving fine.
        assert_eq!(ServiceState::of(99.85), ServiceState::Operational);
        // The live 7d figure, which had a genuine outage inside it.
        assert_eq!(ServiceState::of(94.98), ServiceState::Outage);
    }

    #[test]
    fn every_state_carries_a_label_and_a_colour() {
        for state in [
            ServiceState::Operational,
            ServiceState::Degraded,
            ServiceState::Outage,
        ] {
            assert!(!state.label().is_empty());
            assert!(!state.headline().is_empty());
            let (border, tint) = state.tone();
            assert!(!border.is_empty());
            assert!(!tint.is_empty());
        }
        // A distinct label per state, so two states cannot render the same pill.
        assert_ne!(
            ServiceState::Operational.label(),
            ServiceState::Degraded.label()
        );
        assert_ne!(ServiceState::Degraded.label(), ServiceState::Outage.label());
    }

    #[test]
    fn percentages_keep_two_decimals() {
        // One decimal would round this to 99.9% and hide a real 0.15% loss.
        assert_eq!(pct2(99.85), "99.85%");
        assert_eq!(pct2(100.0), "100.00%");
    }
}
