// leptos_i18n 0.6.2: load_locales!/provide_i18n_context are deprecated in favor of
// build.rs codegen that is not published as a usable crate; these are the only
// file-based paths. Allowed until upstream ships the codegen crate.
#![allow(deprecated)]
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use serde::Deserialize;
use web_sys::{HtmlElement, window};

leptos_i18n::load_locales!();

use crate::i18n::{Locale, use_i18n};
use leptos_i18n::{t, t_string};

fn copy_terminal_code() {
    // Index-based detection: tab labels are localized, so never match on text.
    let tab_idx: u8 = document()
        .query_selector_all(".terminal-tabs button")
        .ok()
        .and_then(|nodes| {
            (0..nodes.length()).find(|&i| {
                nodes
                    .item(i)
                    .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
                    .map(|el| el.class_list().contains("active"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(0) as u8;

    let os = detect_os();
    let _tag = document()
        .query_selector(".hero-badge")
        .ok()
        .flatten()
        .and_then(|el| el.dyn_into::<HtmlElement>().ok())
        .map(|el| el.inner_text())
        .unwrap_or_default();

    let commands: Vec<String> = match (tab_idx, os) {
        (0, "windows") => vec![
            "$t=((iwr -useb https://api.github.com/repos/opencrabs/opencrabs/releases/latest).Content|ConvertFrom-Json).tag_name; iwr \"https://github.com/opencrabs/opencrabs/releases/download/$t/opencrabs-$t-windows-amd64.zip\" -OutFile opencrabs.zip".into(),
            "Expand-Archive opencrabs.zip -DestinationPath .; .\\opencrabs.exe".into(),
        ],
        // The one-liner is the whole install on both platforms: it fetches the
        // release for the machine it runs on, unpacks it, and links the binary
        // onto PATH. The old three-command sequence needed jq, called the
        // anonymous GitHub API (rate limited by IP reputation), and on macOS
        // ended in a bare binary path while the tarball unpacks to
        // OpenCrabs.app/Contents/MacOS/opencrabs, so the last command failed.
        (0, _) => vec![
            "curl -fsSL https://opencrabs.com/install.sh | bash".into(),
            "opencrabs".into(),
        ],
        (1, _) => vec![
            "cargo install opencrabs".into(),
            "opencrabs".into(),
        ],
        (2, _) => vec![
            "curl -fsSL https://raw.githubusercontent.com/opencrabs/opencrabs/main/src/scripts/setup.sh | bash".into(),
            "git clone https://github.com/opencrabs/opencrabs.git && cd opencrabs".into(),
            "cargo build --release && ./target/release/opencrabs".into(),
        ],
        (3, _) => vec![
            "brew install opencrabs".into(),
            "opencrabs".into(),
        ],
        _ => vec![],
    };

    let text = commands.join("\n");
    if let Some(w) = window() {
        let _ = w.navigator().clipboard().write_text(&text);
    }
}

fn copy_inline(text: &'static str) {
    if let Some(w) = window() {
        let _ = w.navigator().clipboard().write_text(text);
    }
}

fn detect_os() -> &'static str {
    let window = leptos::prelude::window();
    let nav: web_sys::Navigator = window.navigator();
    let ua = nav.user_agent().unwrap_or_default().to_lowercase();
    let platform = nav.platform().unwrap_or_default().to_lowercase();

    if platform.contains("win") || ua.contains("windows") {
        "windows"
    } else if platform.contains("mac") || ua.contains("macintosh") {
        "macos"
    } else {
        "linux"
    }
}

fn download_asset(os: &str) -> &'static str {
    match os {
        "macos" => "opencrabs-macos-arm64.tar.gz",
        "windows" => "opencrabs-windows-amd64.zip",
        _ => "opencrabs-linux-amd64.tar.gz",
    }
}

fn build_download_url_full(tag: &str, os: &str) -> String {
    let asset = download_asset(os);
    let name = asset.replace("opencrabs-", &format!("opencrabs-{}-", tag));
    format!(
        "https://github.com/opencrabs/opencrabs/releases/download/{}/{}",
        tag, name
    )
}

#[derive(Deserialize, Clone, Debug)]
struct GitHubRepo {
    stargazers_count: u32,
}

async fn fetch_star_count() -> Option<u32> {
    Request::get("https://api.github.com/repos/opencrabs/opencrabs")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?
        .json::<GitHubRepo>()
        .await
        .ok()
        .map(|r| r.stargazers_count)
}

/// The slice of the public stats snapshot the badge needs. The snapshot is
/// written by a timer on the host and served as static JSON, so this is a
/// read of an already aggregated file, never a call to the proxy.
#[derive(Deserialize, Clone, Debug, Default)]
struct StatsTotals {
    #[serde(default)]
    unique_ips: u64,
}

#[derive(Deserialize, Clone, Debug)]
struct StatsSnapshot {
    schema: u32,
    #[serde(default)]
    totals: StatsTotals,
}

/// Distinct callers of the inference endpoint, from the public snapshot.
///
/// None when the file is missing, unreadable, or older than schema 2, which
/// predates the caller counts. The badge hides on None instead of printing a
/// zero nobody measured.
async fn fetch_caller_count() -> Option<u64> {
    let snap = Request::get("/stats-data/public.json")
        .send()
        .await
        .ok()?
        .json::<StatsSnapshot>()
        .await
        .ok()?;
    if snap.schema < 2 || snap.totals.unique_ips == 0 {
        return None;
    }
    Some(snap.totals.unique_ips)
}

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    crate::i18n::provide_i18n_context();
    // Restore persisted language (set after provide; runs once at mount).
    if let Some(i18n) = leptos::prelude::use_context::<leptos_i18n::I18nContext<Locale>>()
        && let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten())
        && let Ok(Some(saved)) = storage.get_item("oc-lang")
        && let Ok(loc) = saved.parse::<Locale>()
    {
        i18n.set_locale(loc);
    }

    if let Some(i18n) = leptos::prelude::use_context::<leptos_i18n::I18nContext<Locale>>() {
        listen_for_language_changes(i18n);
    }

    let stars = LocalResource::new(fetch_star_count);
    let callers = LocalResource::new(fetch_caller_count);
    let counts = Signal::derive(move || chrome::Counts {
        stars: stars.get().flatten().unwrap_or(0),
        callers: callers.get().flatten().unwrap_or(0),
    });

    let release = LocalResource::new(fetch_latest_release);
    let tag_signal = Signal::derive(move || {
        release
            .get()
            .flatten()
            .map(|r| r.tag_name.clone())
            .unwrap_or_default()
    });

    view! {
        <ChromeNav counts=counts />
        <Hero />
        <Testimonials />
        <QuickStart tag=tag_signal />
        <Features />
        <Integrations />
        <Community />
        // <Newsletter /> - coming soon
        <ChromeFooter counts=counts />
    }
}

// ── Shared chrome ───────────────────────────────────────────────────────────
//
// The header and the footer are rendered by the `chrome` crate, the same one
// the download page and the stats dashboard use, so all of them are the same
// markup by construction. This app only supplies the translations and the two
// live numbers; the shape, the stylesheet and the behaviour are shared.

const LANG_OPTIONS: &[(&str, &str, &str)] = &[
    ("en", "EN", "English"),
    ("pt-PT", "PT", "Português"),
    ("es", "ES", "Español"),
    ("fr", "FR", "Français"),
    ("ru", "RU", "Русский"),
    ("id", "ID", "Indonesia"),
];

fn locale_code(locale: Locale) -> &'static str {
    match locale {
        Locale::en => "en",
        Locale::pt_PT => "pt-PT",
        Locale::es => "es",
        Locale::fr => "fr",
        Locale::ru => "ru",
        Locale::id => "id",
    }
}

/// The language picker lives in the shared script, which cannot reach into the
/// wasm app. It announces the choice on the window instead, and this is where
/// the app hears it.
fn listen_for_language_changes(i18n: leptos_i18n::I18nContext<Locale>) {
    let Some(win) = window() else { return };
    let handler = leptos::wasm_bindgen::closure::Closure::<dyn FnMut(web_sys::Event)>::new(
        move |event: web_sys::Event| {
            let code = event
                .dyn_ref::<web_sys::CustomEvent>()
                .and_then(|custom| custom.detail().as_string());
            if let Some(locale) = code.and_then(|code| code.parse::<Locale>().ok()) {
                i18n.set_locale(locale);
            }
        },
    );
    let _ = win.add_event_listener_with_callback("oc-lang", handler.as_ref().unchecked_ref());
    handler.forget();
}

#[component]
fn ChromeNav(counts: Signal<chrome::Counts>) -> impl IntoView {
    let i18n = use_i18n();
    let markup = move || {
        let labels = chrome::NavLabels {
            docs: t_string!(i18n, nav_docs).to_string(),
            features: t_string!(i18n, nav_features).to_string(),
            integrations: t_string!(i18n, sec_integrations).to_string(),
            download: t_string!(i18n, nav_download).to_string(),
            github: t_string!(i18n, nav_github).to_string(),
            callers: t_string!(i18n, nav_callers).to_string(),
            or_direct: t_string!(i18n, or_direct).to_string(),
            features_href: "#features".to_string(),
            integrations_href: "#integrations".to_string(),
        };
        let picker = chrome::lang_switcher(LANG_OPTIONS, locale_code(i18n.get_locale()));
        chrome::nav_html(&labels, &counts.get(), Some(&picker))
    };
    view! { <div class="chrome-nav-host" inner_html=markup></div> }
}

#[component]
fn ChromeFooter(counts: Signal<chrome::Counts>) -> impl IntoView {
    let i18n = use_i18n();
    let markup = move || {
        let current = counts.get();
        // The sentence carries a literal `{n}` marker instead of an i18n
        // interpolation, so the same string is both what the page shows now
        // and the template the shared script fills on a page that does not
        // know the count yet.
        let star_template = t_string!(i18n, footer_stars).to_string();
        let star_cta = if current.stars > 0 {
            star_template.replace("{n}", &chrome::thousands(u64::from(current.stars)))
        } else {
            t_string!(i18n, footer_star_zero).to_string()
        };
        let labels = chrome::FooterLabels {
            docs: t_string!(i18n, footer_docs).to_string(),
            stories: t_string!(i18n, footer_stories).to_string(),
            github: t_string!(i18n, footer_github).to_string(),
            changelog: t_string!(i18n, footer_changelog).to_string(),
            stats: t_string!(i18n, footer_stats).to_string(),
            community: t_string!(i18n, footer_community).to_string(),
            telegram: t_string!(i18n, footer_telegram).to_string(),
            discord: t_string!(i18n, footer_discord).to_string(),
            sponsor: t_string!(i18n, footer_sponsor).to_string(),
            coffee: t_string!(i18n, footer_coffee).to_string(),
            license: t_string!(i18n, footer_license).to_string(),
            star_cta,
            star_cta_template: star_template,
            callers: t_string!(i18n, nav_callers).to_string(),
            built_by: t_string!(i18n, footer_built_by).to_string(),
            and_community: t_string!(i18n, footer_and_community).to_string(),
        };
        chrome::footer_html(&labels, &current)
    };
    view! { <div class="chrome-footer-host" inner_html=markup></div> }
}

// ── Hero ────────────────────────────────────────────────────────────────────

#[derive(Deserialize, Clone, Debug)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    published_at: Option<String>,
}

async fn fetch_latest_release() -> Option<GitHubRelease> {
    Request::get("https://api.github.com/repos/opencrabs/opencrabs/releases/latest")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?
        .json::<GitHubRelease>()
        .await
        .ok()
}

fn format_release_date(published_at: &str) -> String {
    // "2026-03-07T01:44:48Z" → "Mar 7, 2026"
    let parts: Vec<&str> = published_at
        .split('T')
        .next()
        .unwrap_or("")
        .split('-')
        .collect();
    if parts.len() == 3 {
        let month = match parts[1] {
            "01" => "Jan",
            "02" => "Feb",
            "03" => "Mar",
            "04" => "Apr",
            "05" => "May",
            "06" => "Jun",
            "07" => "Jul",
            "08" => "Aug",
            "09" => "Sep",
            "10" => "Oct",
            "11" => "Nov",
            "12" => "Dec",
            _ => parts[1],
        };
        let day = parts[2].trim_start_matches('0');
        format!("{} {}, {}", month, day, parts[0])
    } else {
        published_at.to_string()
    }
}

#[component]
fn Hero() -> impl IntoView {
    let i18n = use_i18n();
    let release = LocalResource::new(fetch_latest_release);

    let badge_text = move || {
        match release.get() {
            Some(Some(r)) => {
                let version = r.tag_name.clone();
                let name = r.name.as_deref().unwrap_or("").to_string();
                let date = r
                    .published_at
                    .as_deref()
                    .map(format_release_date)
                    .unwrap_or_default();

                // Use release name if available, otherwise just version + date
                if name.is_empty() || name == version {
                    format!("{} - {}", version, date)
                } else {
                    format!("{} - {} - {}", version, date, name)
                }
            }
            _ => t_string!(i18n, loading_release).to_string(),
        }
    };

    view! {
        <section class="hero">
            <div class="container">
                <img class="hero-crab" src="public/opencrabs-logo.png" alt="OpenCrabs" />
                <h1>"OpenCrabs"</h1>
                <p class="hero-tagline">{move || t!(i18n, hero_tagline)}</p>
                <p class="hero-description">
                    {move || t!(i18n, hero_description)}
                </p>
                <a href="https://github.com/opencrabs/opencrabs/releases/latest" class="hero-badge">
                    <span class="badge-new">{move || t!(i18n, badge_latest)}</span>
                    {badge_text}
                    <span class="arrow">" →"</span>
                </a>
                <p class="release-note">
                    {move || t!(i18n, latest_release_note)}
                </p>
                // Xiaomi collab ended - MiMo is now a normal keyed provider
            </div>
        </section>
    }
}

// ── Quick Start ─────────────────────────────────────────────────────────────

#[component]
fn QuickStart(tag: Signal<String>) -> impl IntoView {
    let i18n = use_i18n();
    let (active_tab, set_active_tab) = signal(0u8);

    view! {
        <section id="quickstart">
            <div class="container">
                <h2 class="section-title">
                    <span class="chevron">"›"</span>
                    {move || t!(i18n, qs_title)}
                </h2>
                <div class="terminal">
                    <div class="terminal-header">
                        <div class="terminal-dots">
                            <span class="red"></span>
                            <span class="yellow"></span>
                            <span class="green"></span>
                        </div>
                        <div class="terminal-tabs">
                            <button
                                class:active=move || active_tab.get() == 0
                                on:click=move |_| set_active_tab.set(0)
                            >{move || t!(i18n, tab_binary)}</button>
                            <button
                                class:active=move || active_tab.get() == 1
                                on:click=move |_| set_active_tab.set(1)
                            >{move || t!(i18n, tab_cargo)}</button>
                            <button
                                class:active=move || active_tab.get() == 2
                                on:click=move |_| set_active_tab.set(2)
                            >{move || t!(i18n, tab_source)}</button>
                            <button
                                class:active=move || active_tab.get() == 3
                                on:click=move |_| set_active_tab.set(3)
                            >{move || t!(i18n, tab_homebrew)}</button>
                        </div>
                        <span class="terminal-platform">{move || t!(i18n, qs_platform)}</span>
                        <button class="terminal-copy-btn" on:click=move |_| copy_terminal_code()>
                            "📋"
                        </button>
                    </div>
                    <div class="terminal-body" style:display=move || if active_tab.get() == 0 { "block" } else { "none" }>
                        {move || {
                            let os = detect_os();
                            let t = tag.get();
                            let label = match os {
                                "macos" => t_string!(i18n, dl_macos).to_string(),
                                "windows" => t_string!(i18n, dl_windows).to_string(),
                                _ => t_string!(i18n, dl_linux).to_string(),
                            };
                            let url = if t.is_empty() {
                                "https://github.com/opencrabs/opencrabs/releases/latest".to_string()
                            } else {
                                build_download_url_full(&t, os)
                            };
                            view! {
                                {if os == "macos" {
                                    view! {
                                        <div class="brew-hero">
                                            <div class="brew-cmd-row">
                                                <span class="terminal-prompt">"$ "</span>
                                                <span class="terminal-cmd"><strong>"brew install opencrabs"</strong></span>
                                                <button class="copy-inline" on:click=move |_| copy_inline("brew install opencrabs")>"📋"</button>
                                            </div>
                                            <div class="terminal-comment">{move || t!(i18n, brew_rec_cmt)}</div>
                                        </div>
                                        <div class="download-row" style="margin-top:12px">
                                            <a href=url class="download-all">"⬇ "{move || t!(i18n, dl_macos)}</a>
                                            <a href="https://github.com/opencrabs/opencrabs/releases/latest" class="download-all">{move || t!(i18n, all_platforms)}</a>
                                        </div>
                                        <div class="download-divider">{move || t!(i18n, or_via_terminal)}</div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"curl -fsSL https://opencrabs.com/install.sh | bash"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"opencrabs"</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="download-row">
                                            <a href=url class="download-btn">{label}</a>
                                            <a href="https://github.com/opencrabs/opencrabs/releases/latest" class="download-all">{move || t!(i18n, all_platforms)}</a>
                                        </div>
                                        <div class="download-divider">{move || t!(i18n, or_via_terminal)}</div>
                                        {if os == "windows" {
                                    view! {
                                        <div>
                                            <span class="terminal-comment">{move || t!(i18n, cmt_powershell)}</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"PS> "</span>
                                            <span class="terminal-cmd">"Invoke-WebRequest -Uri https://github.com/opencrabs/opencrabs/releases/latest/download/opencrabs-windows-amd64.zip -OutFile opencrabs.zip"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"PS> "</span>
                                            <span class="terminal-cmd">"Expand-Archive opencrabs.zip -DestinationPath . && .\\opencrabs.exe"</span>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"curl -fsSL https://opencrabs.com/install.sh | bash"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"opencrabs"</span>
                                        </div>
                                    }.into_any()
                                }}
                                    }.into_any()
                                }}
                            }
                        }}
                    </div>
                    <div class="terminal-body" style:display=move || if active_tab.get() == 1 { "block" } else { "none" }>
                        <div>
                            <span class="terminal-comment">{move || t!(i18n, cmt_cargo)}</span>
                        </div>
                        <div>
                            <span class="terminal-prompt">"$ "</span>
                            <span class="terminal-cmd">"cargo install opencrabs"</span>
                        </div>
                        <div>
                            <span class="terminal-prompt">"$ "</span>
                            <span class="terminal-cmd">"opencrabs"</span>
                        </div>
                    </div>
                    <div class="terminal-body" style:display=move || if active_tab.get() == 2 { "block" } else { "none" }>
                        <div>
                            <span class="terminal-comment">{move || t!(i18n, cmt_deps)}</span>
                        </div>
                        <div>
                            <span class="terminal-prompt">"$ "</span>
                            <span class="terminal-cmd">"curl -fsSL https://raw.githubusercontent.com/opencrabs/opencrabs/main/src/scripts/setup.sh | bash"</span>
                        </div>
                        <div>
                            <span class="terminal-comment">{move || t!(i18n, cmt_clone)}</span>
                        </div>
                        <div>
                            <span class="terminal-prompt">"$ "</span>
                            <span class="terminal-cmd">"git clone https://github.com/opencrabs/opencrabs.git && cd opencrabs"</span>
                        </div>
                        <div>
                            <span class="terminal-prompt">"$ "</span>
                            <span class="terminal-cmd">"cargo build --release && ./target/release/opencrabs"</span>
                        </div>
                    </div>
                    <div class="terminal-body" style:display=move || if active_tab.get() == 3 { "block" } else { "none" }>
                        <div>
                            <span class="terminal-comment">{move || t!(i18n, cmt_homebrew)}</span>
                        </div>
                        <div>
                            <span class="terminal-prompt">"$ "</span>
                            <span class="terminal-cmd">"brew install opencrabs"</span>
                        </div>
                        <div>
                            <span class="terminal-prompt">"$ "</span>
                            <span class="terminal-cmd">"opencrabs"</span>
                        </div>
                    </div>
                </div>
                <p class="terminal-note">
                    {move || t!(i18n, note_a)}
                    <strong>{move || t!(i18n, note_evolve)}</strong>
                    {move || t!(i18n, note_b)}
                </p>
            </div>
        </section>
    }
}

// ── Features ────────────────────────────────────────────────────────────────

#[component]
fn Features() -> impl IntoView {
    let i18n = use_i18n();
    let features = vec![
        (
            "🆓",
            "Free AI Out of the Box",
            "Ships with a free built-in provider. No account, no API key, no card, and it is already running on your first turn.",
        ),
        (
            "🖥️",
            "One Binary, Every OS",
            "About 35 MB for macOS, Linux and Windows. Zero telemetry, not even opt-in: the only traffic is what you ask for.",
        ),
        (
            "💬",
            "Any Chat App",
            "Telegram, Discord, Slack, WhatsApp and Trello, with native tables, code blocks, polls and live streaming.",
        ),
        (
            "🧠",
            "Remembers You",
            "Your preferences, context and past decisions persist across sessions and stay searchable long after.",
        ),
        (
            "⚡",
            "50+ Built-in Tools",
            "Files, bash, web, browser automation, image generation, documents and voice. Nothing to install first.",
        ),
        (
            "🪟",
            "Split Panes",
            "Run ten sessions side by side, each with its own provider and context, all working at once.",
        ),
        (
            "🎯",
            "Mission Control",
            "One dashboard for analytics, the improvement inbox, recent activity and the job schedule.",
        ),
        (
            "🤖",
            "Multi-Agent Teams",
            "Typed sub-agents and parallel teams, each child in its own worktree so a fan-out cannot clobber your tree.",
        ),
        (
            "🔄",
            "Autonomous Goals",
            "Set a goal with /goal and it executes, judges its own work and corrects until done or out of budget.",
        ),
        (
            "🌐",
            "Browser Automation",
            "Native headless Chrome. Click, type, screenshot and read any page, including shadow-DOM components.",
        ),
        (
            "🔒",
            "Security Gates",
            "SSH keys, .env files and credentials stay private. Owner-only gates on updates, commands and channel actions.",
        ),
        (
            "🛡️",
            "Self-Healing",
            "Repairs a corrupt config, fails over between providers, compacts before the context wall, never poisons last-good.",
        ),
        (
            "🔧",
            "Self-Improving",
            "Reads its own failure ledger and updates its brain files, proposing new tools and skills for your review.",
        ),
        (
            "👤",
            "Multi-Profile",
            "Several isolated instances from one binary, each with its own config, brain, sessions and daemon.",
        ),
        (
            "📁",
            "Projects",
            "Group work into projects with their own sessions, brain overlay and archived file artifacts.",
        ),
        (
            "📋",
            "Reads Your Repo Rules",
            "Picks up CLAUDE.md, GEMINI.md, .cursorrules and AGENTS.md with no import step, and ships its own CODE.md.",
        ),
    ];

    view! {
        <section id="features">
            <div class="container">
                <h2 class="section-title">
                    <span class="chevron">"›"</span>
                    {move || t!(i18n, sec_features)}
                </h2>
                <div class="features-grid">
                    {features.into_iter().map(|(icon, title, desc)| view! {
                        <div class="feature-card">
                            <span class="feature-icon">{icon}</span>
                            <h3>{title}</h3>
                            <p>{desc}</p>
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </section>
    }
}

// ── Integrations ────────────────────────────────────────────────────────────

#[component]
fn Integrations() -> impl IntoView {
    let i18n = use_i18n();
    let channels = vec![
        "📱 Telegram",
        "💬 Discord",
        "🔔 Slack",
        "📞 WhatsApp",
        "📋 Trello",
    ];
    let providers = vec![
        "🤖 Anthropic",
        "🧠 OpenAI",
        "💎 Gemini",
        "🌐 OpenRouter",
        "🐙 GitHub Copilot",
        "🇨🇳 Xiaomi MiMo",
        "🇨🇳 z.ai GLM",
        "🏠 Ollama",
        "📡 LM Studio",
        "🔮 MiniMax",
        "💻 Claude CLI",
        "⌨️ OpenCode CLI",
        "🤖 Codex CLI",
    ];
    let tools = vec![
        "🐙 GitHub CLI",
        "📧 Gmail",
        "📅 Calendar",
        "🐦 Twitter/X",
        "📸 Instagram",
        "💼 LinkedIn",
        "🔍 Brave Search",
        "🎙️ Local STT/TTS",
        "🐳 Docker",
        "🌐 Browser (CDP)",
    ];

    view! {
        <section id="integrations">
            <div class="container">
                <h2 class="section-title">
                    <span class="chevron">"›"</span>
                    {move || t!(i18n, sec_integrations)}
                </h2>
                <div class="integrations-row">
                    {channels.into_iter().map(|name| view! {
                        <span class="integration-badge">{name}</span>
                    }).collect::<Vec<_>>()}
                </div>
                <div class="integrations-row">
                    {providers.into_iter().map(|name| view! {
                        <span class="integration-badge">{name}</span>
                    }).collect::<Vec<_>>()}
                </div>
                <div class="integrations-row">
                    {tools.into_iter().map(|name| view! {
                        <span class="integration-badge">{name}</span>
                    }).collect::<Vec<_>>()}
                </div>
                <div class="integrations-links">
                    <a href="https://docs.opencrabs.com/channels/overview.html" target="_blank">{move || t!(i18n, view_integrations)}</a>
                </div>
            </div>
        </section>
    }
}

// ── Testimonials ────────────────────────────────────────────────────────────

#[component]
fn Testimonials() -> impl IntoView {
    let i18n = use_i18n();
    let quotes: Vec<(&str, &str, &str)> = vec![
        (
            "I've been an openclaw user since launch (still am), but opencrabs definitely feels polished, lightweight, and stable. It treats Telegram as a first-class channel: TG support means I am using my agent on every device I have.",
            "@arfonzo",
            "https://x.com/arfonzo/status/2094934506796916828",
        ),
        (
            "Outperforms Hermes because it's in Rust and very tight use case wise. Reminds me of what I hoped OpenClaw would be.",
            "@5fakb48",
            "https://x.com/5fakb48/status/2069564004104561125",
        ),
        (
            "I tested OpenCrabs with OpenRouter and a random Nvidia free model, and right off the bat I like it more than the alternatives. Setup was a lot easier.",
            "@mariodian",
            "https://x.com/mariodian/status/2031545206559170847",
        ),
        (
            "Local STT + TTS in ~135MB is impressive. Feels like the voice layer for local agents is getting a lot more practical.",
            "@AlexBuildsCo",
            "https://x.com/AlexBuildsCo/status/2031037889094562160",
        ),
        (
            "After I was using OpenClaw and Hermes, OpenCrabs is the first agent I click with.",
            "@anschmieg",
            "https://github.com/opencrabs/opencrabs/pull/61#issuecomment-4201983933",
        ),
        (
            "I decided to check out Opencrabs. I installed it on Ubuntu Linux and it just works. Lot better than openclaw and if continued development it may even get enough traction among developers.",
            "@0xMaheshK",
            "https://x.com/0xMaheshK/status/2062763007529320646",
        ),
        (
            "My favorite agent harness right now is @opencrabs. It comes with a free license to DeepSeek 4 Flash so I've been playing with that model lately.",
            "@abraxas_null",
            "https://x.com/abraxas_null/status/2115990452679938558",
        ),
        (
            "The agent personality is the best thing about OpenCrabs. Hard to explain.",
            "Declutter · @declutterKE",
            "https://x.com/declutterKE/status/2079883087979159823",
        ),
    ];

    view! {
        <section id="testimonials">
            <div class="container">
                <div class="testimonials-header">
                    <h2 class="section-title" style="margin-bottom: 0">
                        <span class="chevron">"›"</span>
                        {move || t!(i18n, sec_testimonials)}
                    </h2>
                </div>
                <div class="testimonials-grid">
                    {quotes.into_iter().map(|(quote, author, link)| view! {
                        <div class="testimonial-card">
                            <p>"\""{ quote }"\""</p>
                            {if link.is_empty() {
                                view! { <span class="testimonial-author">{author}</span> }.into_any()
                            } else {
                                view! { <a class="testimonial-author" href={link} target="_blank" rel="noopener">{author}</a> }.into_any()
                            }}
                        </div>
                    }).collect::<Vec<_>>()}
                </div>
                <div class="stories-cta-wrap">
                    <a class="stories-cta" href="https://docs.opencrabs.com/user-stories.html" target="_blank" rel="noopener">
                        {move || t!(i18n, stories_cta)}
                        <span class="arrow">" →"</span>
                    </a>
                </div>
            </div>
        </section>
    }
}

// ── Community ───────────────────────────────────────────────────────────────

#[component]
fn Community() -> impl IntoView {
    let i18n = use_i18n();
    view! {
        <section id="community">
            <div class="container">
                <h2 class="section-title">
                    <span class="chevron">"›"</span>
                    {move || t!(i18n, sec_community)}
                </h2>
                <div class="community-grid">
                    <a href="https://www.youtube.com/@OpenCrabs" class="community-card" target="_blank">
                        <span class="icon">
                            <svg viewBox="0 0 24 24" width="30" height="30" fill="#FF0000" aria-hidden="true"><path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z"/></svg>
                        </span>
                        <h3>{move || t!(i18n, comm_youtube_t)}</h3>
                        <p>{move || t!(i18n, comm_youtube_d)}</p>
                    </a>
                    <a href="https://github.com/opencrabs/opencrabs/issues" class="community-card" target="_blank">
                        <span class="icon">"🐛"</span>
                        <h3>{move || t!(i18n, comm_issues_t)}</h3>
                        <p>{move || t!(i18n, comm_issues_d)}</p>
                    </a>
                    <a href="https://github.com/opencrabs/opencrabs/blob/main/CHANGELOG.md" class="community-card" target="_blank">
                        <span class="icon">"📋"</span>
                        <h3>{move || t!(i18n, comm_changelog_t)}</h3>
                        <p>{move || t!(i18n, comm_changelog_d)}</p>
                    </a>
                    <a href="https://docs.opencrabs.com" class="community-card" target="_blank">
                        <span class="icon">"📖"</span>
                        <h3>{move || t!(i18n, comm_docs_t)}</h3>
                        <p>{move || t!(i18n, comm_docs_d)}</p>
                    </a>
                    <a href="https://t.me/opencrabs" class="community-card" target="_blank">
                        <span class="icon">
                            <svg viewBox="0 0 24 24" width="30" height="30" fill="#26A5E4" aria-hidden="true"><path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/></svg>
                        </span>
                        <h3>{move || t!(i18n, comm_telegram_t)}</h3>
                        <p>{move || t!(i18n, comm_telegram_d)}</p>
                    </a>
                    <a href="https://discord.gg/dJwCRawb6h" class="community-card" target="_blank">
                        <span class="icon">
                            <svg viewBox="0 0 24 24" width="30" height="30" fill="#5865F2" aria-hidden="true"><path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"/></svg>
                        </span>
                        <h3>{move || t!(i18n, comm_discord_t)}</h3>
                        <p>{move || t!(i18n, comm_discord_d)}</p>
                    </a>
                </div>
            </div>
        </section>
    }
}

// ── Newsletter (coming soon - self-hosted on droplet with Resend) ───────────
//
// #[component]
// fn Newsletter() -> impl IntoView {
//     view! {
//         <section>
//             <div class="container">
//                 <div class="newsletter">
//                     <h2>
//                         <span class="chevron">"› "</span>
//                         "Stay in the Loop"
//                     </h2>
//                     <p>"Get updates on new features, integrations, and crab wisdom. No spam, unsubscribe anytime."</p>
//                     <div class="newsletter-form">
//                         <input type="email" placeholder="your@email.com" />
//                         <button>"Subscribe →"</button>
//                     </div>
//                 </div>
//             </div>
//         </section>
//     }
// }

#[cfg(test)]
mod tests {
    use chrome::{Counts, FOOTER_START, FooterLabels, NAV_START, NavLabels, inject_page};

    /// The one stylesheet that styles the header and the footer. Every page
    /// loads it, so every rule about the chrome is pinned here and nowhere
    /// else.
    const CHROME_CSS: &str = include_str!("../chrome/chrome.css");

    /// The behaviour: the menu, the brew popover, the language picker and both
    /// live counters. Also the only script the chrome needs.
    const CHROME_JS: &str = include_str!("../chrome/chrome.js");

    /// The markup itself, for the two wasm pages that call the builders.
    const CHROME_LIB: &str = include_str!("../chrome/src/lib.rs");

    /// Every page on the site. The two static ones are stamped by
    /// `chrome-static`; the two Leptos ones call the builders directly. All
    /// four have to end up with the same header and the same footer.
    const PAGES: [(&str, &str); 4] = [
        ("landing", include_str!("../index.html")),
        ("stats", include_str!("../stats/index.html")),
        ("download", include_str!("root-files/download/index.html")),
        ("404", include_str!("root-files/404.html")),
    ];

    /// The declarations of the first rule whose selector starts a line with
    /// `selector`, up to its closing brace.
    fn rule<'a>(css: &'a str, selector: &str) -> &'a str {
        let at = css
            .find(selector)
            .unwrap_or_else(|| panic!("no {selector} rule"));
        let block = &css[at..];
        &block[..block.find('}').expect("rule end")]
    }

    /// Every `@media (max-width: Npx)` block, as (width, block text).
    fn media_blocks(css: &str) -> Vec<(u32, String)> {
        let mut out = Vec::new();
        let mut rest = css;
        while let Some(at) = rest.find("@media (max-width: ") {
            let after = &rest[at + "@media (max-width: ".len()..];
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            let width: u32 = digits.parse().expect("media width");
            let open = rest[at..].find('{').expect("media block opens");
            let mut depth = 0usize;
            let mut end = rest.len();
            for (idx, ch) in rest[at + open..].char_indices() {
                match ch {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            end = at + open + idx;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            out.push((width, rest[at..end].to_string()));
            rest = &rest[end..];
        }
        out
    }

    // The hamburger is the nav, at every width. It used to appear only under
    // 1200px, so the same site read as two different products depending on the
    // window, and every page that forgot the breakpoint kept six links on one
    // row until they wrapped mid-phrase and the bar doubled in height. There is
    // now no width at which the links are laid out as a row: they are a panel
    // that opens under the bar, or they are hidden.
    #[test]
    fn hamburger_is_the_only_nav_shape() {
        let links = rule(CHROME_CSS, ".nav-links {");
        assert!(
            links.contains("display: none"),
            "the link panel must be closed until the toggle opens it"
        );
        assert!(
            links.contains("position: absolute"),
            "the panel floats under the bar instead of taking a row in it"
        );

        let toggle = rule(CHROME_CSS, ".nav-toggle {");
        assert!(
            toggle.contains("display: flex"),
            "the toggle is the only way into the menu, so it is always shown"
        );

        // No breakpoint may bring the row back. A block that shows the toggle
        // is the signature of the old collapse, and a block that shows the
        // links as a row is the same mistake spelled differently.
        for (width, block) in media_blocks(CHROME_CSS) {
            assert!(
                !block.contains(".nav-toggle"),
                "{width}px: the toggle is not conditional, it is always on"
            );
            assert!(
                !block.contains(".nav-links {"),
                "{width}px: the link panel must not change shape at any width"
            );
        }
    }

    // The panel hangs off the right edge of the page, under the hamburger it
    // belongs to, and sizes to its own content so a long label does not push it
    // off screen. It takes a full border: a bottom-only edge reads as part of
    // the bar rather than as a floating card.
    #[test]
    fn nav_panel_opens_anchored_right() {
        let links = rule(CHROME_CSS, ".nav-links {");
        for needle in [
            "right: 24px",
            "width: max-content",
            "min-width: 240px",
            "max-width: calc(100vw - 48px)",
            "flex-direction: column",
        ] {
            assert!(links.contains(needle), "the panel is missing {needle}");
        }
        assert!(
            !links.contains("border-bottom"),
            "a floating panel takes a full border, not a bottom-only edge"
        );
    }

    // The bar carries the logo and the action cluster and nothing else, so it
    // lines up with the page's own content column instead of needing a wider
    // one. The wider 1200px container existed only to fit six links in a row.
    #[test]
    fn nav_fits_the_content_column() {
        let container = rule(CHROME_CSS, "nav .container {");
        assert!(
            container.contains("max-width: 1100px"),
            "the bar has two items in it now, so it takes the page column"
        );
    }

    // Every page loads the same two files. A page that links its own copy of
    // the chrome is exactly the drift this module exists to stop, so the link
    // and the script are pinned on all four.
    #[test]
    fn every_page_loads_the_shared_chrome() {
        let sheet = format!("href=\"{}\"", "/chrome.css");
        let script = format!("src=\"{}\"", "/chrome.js");
        for (name, page) in PAGES {
            assert!(page.contains(&sheet), "{name}: does not load /chrome.css");
            assert!(page.contains(&script), "{name}: does not load /chrome.js");
        }
    }

    // The static pages carry generated markup, so they can go stale. This
    // re-runs the generator in memory: if a checked-in page differs by a byte,
    // someone edited the header and forgot `chrome-static --write`.
    #[test]
    fn static_pages_carry_the_generated_chrome() {
        let nav = chrome::nav_html(&NavLabels::english(), &Counts::default(), None);
        let footer = chrome::footer_html(&FooterLabels::english(), &Counts::default());

        for (name, page) in [("download", PAGES[2].1), ("404", PAGES[3].1)] {
            assert!(
                page.contains(NAV_START) && page.contains(FOOTER_START),
                "{name}: carries no chrome markers, so nothing can regenerate it"
            );
            let rendered =
                inject_page(page, &nav, &footer).unwrap_or_else(|err| panic!("{name}: {err}"));
            assert_eq!(
                rendered, page,
                "{name}: the checked-in header or footer is stale, run \
                 `cargo run -p chrome --bin chrome-static -- --write src/root-files`"
            );
        }
    }

    // One stylesheet owns the chrome. The page sheets used to repeat it: the
    // download page had its own copy of every nav and footer rule, and the
    // landing carried the whole thing twice. Anything chrome-shaped found in a
    // page sheet is that copy growing back.
    #[test]
    fn page_sheets_do_not_duplicate_the_chrome() {
        let page_sheets = [
            ("landing style.css", include_str!("../style.css")),
            ("stats.css", include_str!("../stats/stats.css")),
            (
                "download inline CSS",
                include_str!("root-files/download/index.html"),
            ),
        ];
        let chrome_selectors = [
            "\nnav {",
            "\nnav .container {",
            "\n.nav-links",
            "\n.nav-toggle",
            "\n.nav-logo",
            "\n.nav-actions",
            "\n.footer-",
            "@media (max-width: 1200px)",
        ];
        for (name, css) in page_sheets {
            for needle in chrome_selectors {
                assert!(
                    !css.contains(needle),
                    "{name}: still styles {needle}, which belongs to chrome.css"
                );
            }
        }
    }

    // The fixed header is 77px tall (16px padding either side of the 44px logo
    // plus its border). A page whose body starts closer than that has its first
    // row tucked under the bar, which is what the stats page did at 620px and
    // down: the mobile override set 72px of top padding under a 77px header.
    // Every rule that sets the top padding counts, not just the base one.
    #[test]
    fn pages_clear_the_fixed_header() {
        const HEADER_H: u32 = 77;
        let stats_css = include_str!("../stats/stats.css");
        let download = include_str!("root-files/download/index.html");
        let not_found = include_str!("root-files/404.html");

        for (name, css, selector) in [
            ("stats", stats_css, ".stats-wrap {"),
            ("download", download, "body {"),
            ("404", not_found, "body {"),
        ] {
            let mut tops = Vec::new();
            for (idx, _) in css.match_indices(selector) {
                let block = &css[idx..];
                let block = &block[..block.find('}').expect("rule end")];
                let pad_at = match block.find("padding") {
                    Some(v) => v,
                    None => continue,
                };
                let value = &block[pad_at..];
                let digits: String = value
                    .chars()
                    .skip_while(|c| !c.is_ascii_digit())
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                let top: u32 = digits
                    .parse()
                    .unwrap_or_else(|_| panic!("{name}: unreadable padding {value:?}"));
                tops.push(top);
            }
            assert!(
                !tops.is_empty(),
                "{name}: no {selector} rule sets a padding"
            );
            let smallest = *tops.iter().min().expect("at least one padding");
            assert!(
                smallest >= HEADER_H,
                "{name}: a {selector} rule starts {smallest}px down, under the {HEADER_H}px header"
            );
        }
    }

    // Regression pin for the 320px header: the base .nav-social and
    // .lang-switcher rules sit ABOVE the 380px block, and media queries add no
    // specificity, so a one-class hide rule there loses on source order and the
    // row stayed 5px too wide. The narrow-width block must reach through a
    // 2-class selector.
    #[test]
    fn narrow_header_hides_social_by_specificity() {
        let block = media_blocks(CHROME_CSS)
            .into_iter()
            .find(|(width, _)| *width == 380)
            .map(|(_, block)| block)
            .expect("380px media query");
        assert!(
            block.contains(".nav-actions .nav-social { display: none; }"),
            "the 380px block must hide the social icons through a 2-class selector"
        );
        assert!(
            block.contains(".nav-actions .lang-switcher"),
            "the 380px block must shrink the picker through a 2-class selector"
        );
    }

    // The public stats page is reachable from the site chrome, and the caller
    // count rides in the nav and in the footer. Both are easy to lose in a
    // redesign, so pin the link and the badge in the module every page uses.
    #[test]
    fn stats_links_and_caller_badge() {
        // The assertions below read files that contain the same strings, so the
        // needles are built at runtime. Written as literals they would match
        // themselves and pass even after the markup was deleted.
        let stats_link = format!("href=\"{}\"", "/stats/");
        let nav_badge = format!("class=\"{}\"", "nav-callers");
        let footer_badge = format!("class=\"{}\"", "footer-callers");

        assert!(
            CHROME_LIB.contains(&stats_link),
            "the footer must link to /stats/"
        );
        assert!(
            CHROME_LIB.contains(&nav_badge),
            "the nav must render the caller badge"
        );
        assert!(
            CHROME_LIB.contains(&footer_badge),
            "the footer repeats the badge beside the star button"
        );

        // The pill styling has to exist or the badge renders as bare text.
        for selector in [
            ".nav-callers {",
            ".nav-callers::before",
            ".footer-callers {",
        ] {
            assert!(
                CHROME_CSS.contains(selector),
                "chrome.css is missing {selector}"
            );
        }

        // Both numbers are filled in by the shared script, from the public
        // snapshot and from GitHub. No page fetches them itself any more.
        assert!(
            CHROME_JS.contains("/stats-data/public.json"),
            "chrome.js must read the public snapshot for the caller pill"
        );
        // The script builds the star URL from its own REPO constant, so pin
        // both halves rather than the joined string it never contains.
        assert!(
            CHROME_JS.contains("api.github.com/repos/")
                && CHROME_JS.contains("opencrabs/opencrabs"),
            "chrome.js must read the star count from GitHub"
        );
    }
}
