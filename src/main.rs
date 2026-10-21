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
        (0, "macos") => vec![
            "TAG=$(curl -s https://api.github.com/repos/opencrabs/opencrabs/releases/latest | jq -r .tag_name)".into(),
            "curl -fsSL https://github.com/opencrabs/opencrabs/releases/download/$TAG/opencrabs-$TAG-macos-arm64.tar.gz | tar xz".into(),
            "./opencrabs".into(),
        ],
        (0, _) => vec![
            "sudo apt install libgomp1".into(),
            "TAG=$(curl -s https://api.github.com/repos/opencrabs/opencrabs/releases/latest | jq -r .tag_name)".into(),
            "curl -fsSL https://github.com/opencrabs/opencrabs/releases/download/$TAG/opencrabs-$TAG-linux-amd64.tar.gz | tar xz".into(),
            "./opencrabs".into(),
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

/// 1234 -> "1,234", so a five digit caller count still reads at a glance.
fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
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

    let stars = LocalResource::new(fetch_star_count);
    let stars_signal = Signal::derive(move || stars.get().flatten().unwrap_or(0));

    let callers = LocalResource::new(fetch_caller_count);
    let callers_signal = Signal::derive(move || callers.get().flatten().unwrap_or(0));

    let release = LocalResource::new(fetch_latest_release);
    let tag_signal = Signal::derive(move || {
        release
            .get()
            .flatten()
            .map(|r| r.tag_name.clone())
            .unwrap_or_default()
    });

    view! {
        <Nav stars=stars_signal tag=tag_signal callers=callers_signal />
        <Hero />
        <Testimonials />
        <QuickStart tag=tag_signal />
        <Features />
        <Integrations />
        <Community />
        // <Newsletter /> — coming soon
        <Footer stars=stars_signal />
    }
}

// ── Language switcher ───────────────────────────────────────────────────────

#[component]
fn LangSwitcher() -> impl IntoView {
    let i18n = use_i18n();
    // Display: compact initials (fits nav on any width); title keeps the full name on hover.
    let langs: Vec<(Locale, &'static str, &'static str)> = vec![
        (Locale::en, "en", "English"),
        (Locale::pt_PT, "pt-PT", "Português"),
        (Locale::es, "es", "Español"),
        (Locale::fr, "fr", "Français"),
        (Locale::ru, "ru", "Русский"),
        (Locale::id, "id", "Indonesia"),
    ];
    let initials = |name: &str| -> String {
        // "English" -> "EN", "Português" -> "PT", "Русский" -> "RU", "Indonesia" -> "ID"
        match name {
            "English" => "EN".into(),
            "Português" => "PT".into(),
            "Español" => "ES".into(),
            "Français" => "FR".into(),
            "Русский" => "RU".into(),
            "Indonesia" => "ID".into(),
            other => other.chars().take(2).collect(),
        }
    };

    let stored = window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("oc-lang").ok().flatten());
    let current = stored
        .and_then(|code| code.parse::<Locale>().ok())
        .unwrap_or_else(|| i18n.get_locale());

    view! {
        <select
            class="lang-switcher"
            aria-label="Language"
            on:change=move |ev| {
                let code = event_target_value(&ev);
                if let Ok(loc) = code.parse::<Locale>() {
                    i18n.set_locale(loc);
                    if let Some(storage) = window()
                        .and_then(|w| w.local_storage().ok().flatten())
                    {
                        let _ = storage.set_item("oc-lang", &code);
                    }
                }
            }
        >
            {langs.into_iter().map(|(loc, code, name)| {
                let selected = loc == current;
                let label = initials(name);
                view! { <option value=code title=name selected=selected>{label}</option> }
            }).collect::<Vec<_>>()}
        </select>
    }
}

// ── Navigation ──────────────────────────────────────────────────────────────

#[component]
fn Nav(stars: Signal<u32>, tag: Signal<String>, callers: Signal<u64>) -> impl IntoView {
    let i18n = use_i18n();
    let menu_open = RwSignal::new(false);
    let dl_popover = RwSignal::new(false);
    let star_label = move || {
        let count = stars.get();
        if count > 0 {
            format!(" ★ {}", count)
        } else {
            String::new()
        }
    };

    view! {
        <nav>
            <div class="container">
                <a href="/" class="nav-logo">
                    <img class="crab-icon" src="public/opencrabs-logo.png" alt="" />
                    "OpenCrabs"
                </a>
                <ul class="nav-links" class:open=menu_open on:click=move |_| menu_open.set(false)>
                    <li><a href="https://docs.opencrabs.com" target="_blank">{move || t!(i18n, nav_docs)}</a></li>
                    <li><a href="#features">{move || t!(i18n, nav_features)}</a></li>
                    <li><a href="#integrations">{move || t!(i18n, sec_integrations)}</a></li>
                    <li>
                        {move || {
                            let t = tag.get();
                            let os = detect_os();
                            let url = if t.is_empty() {
                                "https://github.com/opencrabs/opencrabs/releases/latest".to_string()
                            } else {
                                build_download_url_full(&t, os)
                            };
                            if os == "macos" {
                                view! {
                                    <span class="dl-wrap">
                                        <a
                                            href=url.clone()
                                            class="nav-download"
                                            on:click=move |ev: leptos::ev::MouseEvent| { ev.prevent_default(); dl_popover.update(|o| *o = !*o); }
                                        >
                                            {move || t!(i18n, nav_download)}
                                        </a>
                                        <span class="dl-popover" class:open=dl_popover>
                                            <a
                                                class="dl-popover-close"
                                                href="#"
                                                on:click=move |ev: leptos::ev::MouseEvent| { ev.prevent_default(); dl_popover.set(false); }
                                            >
                                                "×"
                                            </a>
                                            <span class="dl-popover-title">"macOS"</span>
                                            <span class="dl-brew-row">
                                                <code>"brew install opencrabs"</code>
                                                <button class="dl-copy" on:click=move |_| copy_inline("brew install opencrabs")>"⧉"</button>
                                            </span>
                                            <a class="dl-direct" href=url>
                                                "⬇ "{move || t!(i18n, or_direct)}
                                            </a>
                                        </span>
                                    </span>
                                }.into_any()
                            } else {
                                view! { <a href="/download/" class="nav-download">{move || t!(i18n, nav_download)}</a> }.into_any()
                            }
                        }}
                    </li>
                    <li>
                        <a href="https://github.com/opencrabs/opencrabs" class="btn-github" target="_blank">
                            {move || t!(i18n, nav_github)}
                            <span class="github-stars">{star_label}</span>
                        </a>
                    </li>
                    <li>
                        // Distinct callers, straight from the public stats
                        // snapshot. Hidden until the count is known, so the nav
                        // never claims zero when the file has not loaded.
                        {move || {
                            let n = callers.get();
                            if n == 0 {
                                ().into_any()
                            } else {
                                view! {
                                    <a href="/stats/" class="nav-callers">
                                        {format!("{} ", thousands(n))}
                                        {move || t!(i18n, nav_callers)}
                                    </a>
                                }
                                .into_any()
                            }
                        }}
                    </li>
                </ul>
                <div class="nav-actions">
                    <div class="nav-social">
                        <a href="https://t.me/opencrabs" target="_blank" rel="noopener noreferrer" class="nav-social-link nav-telegram" aria-label="Telegram">
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" aria-hidden="true"><path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/></svg>
                        </a>
                        <a href="https://discord.gg/dJwCRawb6h" target="_blank" rel="noopener noreferrer" class="nav-social-link nav-discord" aria-label="Discord">
                            <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" aria-hidden="true"><path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"/></svg>
                        </a>
                    </div>
                    <LangSwitcher />
                    <button
                        class="nav-toggle"
                        class:active=menu_open
                        aria-label="Menu"
                        aria-expanded=move || if menu_open.get() { "true" } else { "false" }
                        on:click=move |_| menu_open.update(|o| *o = !*o)
                    >
                        <span class="bar"></span>
                        <span class="bar"></span>
                        <span class="bar"></span>
                    </button>
                </div>
            </div>
        </nav>
    }
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
                    format!("{} — {}", version, date)
                } else {
                    format!("{} — {} — {}", version, date, name)
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
                // Xiaomi collab ended — MiMo is now a normal keyed provider
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
                                            <span class="terminal-cmd">"TAG=$(curl -s https://api.github.com/repos/opencrabs/opencrabs/releases/latest | jq -r .tag_name)"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"curl -fsSL https://github.com/opencrabs/opencrabs/releases/download/$TAG/opencrabs-$TAG-macos-arm64.tar.gz | tar xz"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"./opencrabs"</span>
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
                                            <span class="terminal-cmd">"sudo apt install libgomp1"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"TAG=$(curl -s https://api.github.com/repos/opencrabs/opencrabs/releases/latest | jq -r .tag_name)"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"curl -fsSL https://github.com/opencrabs/opencrabs/releases/download/$TAG/opencrabs-$TAG-linux-amd64.tar.gz | tar xz"</span>
                                        </div>
                                        <div>
                                            <span class="terminal-prompt">"$ "</span>
                                            <span class="terminal-cmd">"./opencrabs"</span>
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
            "🖥️",
            "~34-36 MB Single Binary",
            "Mac, Windows, or Linux. Anthropic, OpenAI, Gemini, GitHub Copilot, or any local model. Zero telemetry, not even opt-in: no analytics, no tracking, no phone-home. The only outbound traffic is what you explicitly initiate. Your data stays yours. First-class Windows (v0.5.3): platform shell handling, BOM/UTF-16 file reads, tilde and cwd fallbacks, and CI builds Windows on every PR. One timeout chain (v0.5.4): timeout flags resolve per provider, then [agent], then the built-in default, honoured by every provider family including anthropic and gemini. Windows lock files (v0.5.5): the LockFileEx saga is closed, and the error paths stopped telling you to run as Administrator or use WSL2. Fresh install (v0.5.11): a first run lands in the chat with the built-in provider already live and the crab greeting you, instead of a provider-less empty screen.",
        ),
        (
            "💬",
            "Any Chat App",
            "Talk to it on Telegram, Discord, Slack, WhatsApp, or Trello. Native rich message rendering with tables, lists, code blocks, math. Draft message streaming shows live \"typing...\" updates as tokens generate. Collapsible <details>/<summary> blocks for long outputs. Forum topic session isolation gives each topic its own context. Discord feature parity: interactive components (select menus, modal forms), media gallery (batch files into one message), grouped tool calls with Expand/Collapse, emoji reactions trigger agent turns. Telegram reactions: reads inbound emoji by sentiment, addresses you by first name, and replies with just a reaction when that fits. Mid-turn reactions inject into the running loop instead of spawning a second turn. Follow-up suggestions (v0.3.69): tap-to-send optional follow-ups on Telegram, Discord, Slack, WhatsApp; fill-not-submit in TUI. Slack Block Kit delivery for rich completion formatting with section blocks, context blocks, and dividers. Slack grouped tool calls collapse into one edited-in-place message. Telegram plan card re-stick (v0.3.71): re-stick plan card and fold prose into it when buried. Full group history capture (v0.3.73): persist every group message to history, even from non-allowlisted senders and bots. Correct group speaker (v0.3.73): stop addressing a group-history sender as the current speaker. /cowork workspaces, /rename sessions, instant fast-cancel on /stop. /cd directory browser with auto project assignment. /new inherits the working directory from your most recent session. /profiles command for managing AI profiles. Owner impersonation detection in group chats. Session search across all channels. Per-group open mode: set open = true on a trusted group to serve all members without individual allowlisting. /cowork opens the group (v0.3.74): /cowork sets the target group open=true and persists it, and its admin deep link has the bot join already promoted. Group onboarding (v0.3.74): the bot greets a new group with an onboarding nudge and registers members per-group via /start. follow_up_question degrades gracefully (v0.3.74): questions fall back to plain text on non-interactive surfaces. Works in DMs and group chats with persistent sessions. Or just use the TUI. /restart and /exit (v0.3.79): /restart relaunches the same binary with the same arguments and resumes unfinished turns on startup, and /exit shuts OpenCrabs down. Owner-only on channels, and both announce before acting. Slack-native rendering (v0.3.80): tables and headings arrive in Slack's own shape instead of raw markdown. Mermaid diagrams render as images in rich messages while tables stay native (v0.3.81). Inline rate-limit waits capped at 30s (v0.3.81). Long rate-limit windows bail immediately instead of parking the send inline (v0.3.82). Rich sends stop building a double-slash URL and now honour a redirected API base, so the rich path is finally testable without a real network call (v0.3.82). The model picker pages and filters instead of arriving as one giant message (v0.3.83). /stop cancels instantly even during the provider handshake and retry backoffs (v0.3.83). WhatsApp full surface (v0.5.2): native voice notes, inbound video, stickers, location, contacts, poll votes and reactions, presence and media recovery, edit-in-place streaming, disappearing messages, contact blocking, pin/forward/profile, status updates, newsletter discovery, native-flow buttons and per-chat history. Discord live tracing and native tables (v0.5.2): long turns stream as a traced live reply, answers auto-thread, markdown tables render natively, and tool turns no longer double-post the final response. Telegram forum topics and /clear (v0.5.2): topic create and rename bind to their own session, and /clear starts a fresh session without the summariser call. Command menu payload budget (v0.5.2): Telegram silently rejects a setMyCommands body over ~7.8KB, so descriptions now shorten in tiers and the whole menu fits. WhatsApp suggestion polls (v0.5.3): follow-up option sets past the native button cap render as a poll, and a vote selects the option. Discord goes native (v0.5.5): native polls via send_poll, TTS answers delivered as voice messages, and commands.toml projected onto native slash commands. Flow groups open at turn start with a live clock over thinking on Discord and Slack, and one outbound write governor budgets every edit and parks on 429s. /respond_to picks who the crab answers per channel, and /cowork opens a channel or thread to its members. Telegram learned rate discipline (v0.5.5): instrumented getUpdates, a cross-surface spacing floor, per-chat 429 pause, and local-file links delivered as documents.",
        ),
        (
            "🧠",
            "Persistent Memory",
            "Remembers you across sessions. Your preferences, your context, your AI. Semantic search over everything. Epistemic memory (v0.5.3): facts carry confidence and decay, cold sections archive themselves, and opencrabs memory prune previews the cleanup before applying it. Decision cache (v0.5.4): decide_cached answers repeat decisions from an L1 reuse ring with shadow/live tiers, so classification-shaped calls are served without re-paying the model. Pre-migration snapshot (v0.5.5): the database copies itself before any migration touches it, so memory upgrades stop being one-way doors.",
        ),
        (
            "⚡",
            "50+ Built-in Tools",
            "File ops, bash, web search, web_scrape (native URL-to-markdown with SSRF guard and sitemap crawling), code execution, image gen, browser automation, local voice STT & TTS, generate_document (native PDF, DOCX, XLSX, PPTX with brand styling, logos, image blocks, live Excel formulas, and custom page sizes/orientations), spreadsheet parsing (XLSX/XLS/CSV), sub-agent orchestration. Built-in /github skill: full gh CLI control for issues, PRs, code reviews, repo management. Concurrent auto-approved tool batches run independent calls in parallel. Proactive tool discovery — agent searches for tools before claiming inability. JIT activation for extended tools on-demand. Auto-download RTK for 10x token savings on 100+ commands. Define custom tools at runtime. Race-free multi-image pickup from any channel. Tool registry shared across all entry points so startup tools are available everywhere. Malformed tools.toml surfaces parse errors instead of silently dropping every tool. last_good .bak snapshot auto-creates on every write with fallback recovery. Multilanguage prompt analyzer (v0.3.67): shared PromptAnalyzer with soft-nudge on TUI and Telegram, 6 language packs (EN/ES/FR/ID/PT/RU). Unified parallel web search (v0.3.73): web_search, exa_search, and brave_search fan out in parallel under a single web_search tool. DDG captcha detection (v0.3.73): detect DuckDuckGo captcha via HTTP 202 + structural form check. Background tasks (v0.3.74): known-long commands (builds, test suites, renders) auto-promote to the background and resume your session the moment they finish. Background-task resume everywhere (v0.3.75): Discord, Slack, WhatsApp, and the CLI now hand the session back the moment a long task finishes, same as TUI and Telegram. Structural code memory (v0.5.0): tree-sitter builds a symbol and call-graph index beside the text index, so \"who calls X\" walks real call edges while conceptual questions keep the text lane. Drop transfer over SSH (v0.5.0): a file dropped into a TUI on a remote host is pulled back through the connection already open, with an opencrabs drop-agent on the client side. Skill-glob gating (v0.5.1): a skill's globs list refuses tool calls that touch matching files until the agent has actually read the skill body. Provider budget backpressure (v0.5.1): a provider that reports an output-token budget triggers one bounded compaction and retry instead of a cascade of 400s. Serper search engine (v0.5.4): serper_search adds Google SERP results and the web_search fan-out dedupes by URL across engines. Generate_image provider chain (v0.5.4): image generation walks a provider chain with vision parity. Compiled /architecture and /attach (v0.5.4), plus an opt-in audit trail with a /audit viewer. claude-cli background tasks (v0.5.5): spawned tasks mirror into the task manager with process-group isolation and group kill, and completions are delivered after the turn ends.",
        ),
        (
            "🪟",
            "Split Panes",
            "Tmux-style horizontal and vertical pane splitting. Run 10 sessions side by side, each with its own provider and context. All processing in parallel. Native markdown rendering: emphasis, lists, links, and task items render directly in the terminal. Group tool calls: consecutive calls collapse into one expandable block. Intermediate processing text folds in-place so only the final answer stays visible. Click-to-expand (v0.3.69): left-click a tool-call or reasoning block to expand just that block, complementing Ctrl+O. Tmux watchdog (v0.3.71): pane-only kill, 30s warning, auto-reattach. Version display (v0.3.73): show the running version on the TUI header and in channel /help + /usage. 3-state reasoning expand (v0.3.74): click or Ctrl+O cycles a reasoning block collapsed → capped → full, so it never floods the view. Per-turn headers (v0.3.75): every turn gets a one-line summary header, its working-out folds into it, and turns fold by default so the view stays clean. Live thinking shows a short excerpt instead of a scrolling wall, and the token counter is labeled as a turn total with the ctx budget. Theme system (v0.5.0): verified presets, an interactive picker, and your own themes from ~/.opencrabs/themes/*.toml, with truecolor detection and an ANSI-256 fallback tier. Cancelling a turn now clears the indicator in split panes too, instead of leaving a pane claiming to think forever (v0.5.0). Per-job cron sessions (v0.5.1): every scheduled job runs in its own session, so concurrent jobs can never bleed into each other's context or overwrite each other's provider. Trigger-gated cron (v0.5.3): a pre-flight check decides whether a job fires at all, skipped runs cost 0 tokens, and a fired trigger can dispatch a goal. Theme enforcement (v0.5.3): every widget resolves through the theme with a suite-level guard against new raw ANSI, and themes can declare a canvas background. Universal paste & drop (v0.5.4): paste or drop any file type and one shared router classifies it into the right pipeline. Interactive terminal handoff (v0.5.4): !vi and !nano take over the terminal and hand it back with the session intact. Shared dialog footer (v0.5.5): every dialog runs on one command footer with direct a/r approval verbs, URLs and file paths are clickable, and onboarding is rebuilt as a left-side step timeline with direct setup commands. Cron one-shot retirement (v0.5.5): one-shot jobs retire on delivery instead of re-firing a year later, every run carries its run id, and Discord cron reports arrive as one forum post.",
        ),
        (
            "🔄",
            "/goal — Autonomous Goal Loop",
            "Set a goal with /goal <text> and the agent loops autonomously: executing, self-evaluating with an LLM judge, and continuing with a correction prompt until the goal is satisfied or the turn budget runs out. Self-goaling: agent can set and drive its own multi-turn goals via goal_manage without you invoking /goal. Supports /goal pause, /goal resume, /goal status, and /goal clear. Works across TUI, Telegram, Discord, Slack, and WhatsApp. Fully hands-off task execution.",
        ),
        (
            "🎯",
            "Mission Control",
            "Full-screen TUI dashboard with four panels: Analytics showing brain file sizes, tool usage bars, and failure rates, Inbox for reviewing RSI-proposed tools, commands, and skills (approve/reject inline with a/r keys), Activity log showing recent self-improvements, and Schedule queue for cron jobs. Rich schedule detail popup shows prompt, delivery target, run history with cost and duration. Cron delivery to Discord and Slack. Two-step delete safeguard with auto-backup. Race-free cron startup (no double-fire on first tick). Scheduler lock prevents duplicate job execution across instances. Shared session per cron job with compaction isolation. Keyboard-driven: Tab/Shift-Tab cycle panels, j/k navigate, Enter for details, Esc to close.",
        ),
        (
            "🤖",
            "Multi-Agent & Teams",
            "Typed sub-agents (General, Explore, Plan, Code, Research) with filtered tool registries. Team orchestration spawns N agents in parallel and broadcasts to all. 20+ CLI subcommands including /mission-control dashboard, /skills picker, /btw parallel agent, /security-audit, /cost-estimate, and /repo-audit. Cross-harness skill system with auto-registered slash commands. Promoted most-used commands to top of /help. All slash commands render through rich AST pipeline with table formatting. Standardized bot menu on underscore form. Daemon mode with health endpoints. Direct model switching everywhere: /models <provider/model> switches directly on every channel with apply-to-scope selector (session or global). Headless model switching via opencrabs session set-model. force_default on reload pushes the default provider/model pair to all sessions. Config drift warnings on startup. Plan mode (v0.3.67-v0.3.68): design/checklist tracks, persistent plan card, /plan <query> command, agent self-approval when user grants autonomy. Skills review_gate (v0.3.70): frontmatter declaration for high-stakes skills that require user approval before side effects. Plan-gate three-state (v0.3.71): GateDecision with bash going to approval in post-init Editing. MCP-style ToolHints (v0.3.71): risk model for tool classification, drives plan gate. Model picker (v0.3.75): Opus 5 surfaced via parallel discovery, newest-first ordering, and claude-cli models discovered live from the CLI instead of a hardcoded list. Plan state across sessions (v0.3.79): plan state threads across session boundaries so plans survive restarts, and spawned child sessions resolve the parent's plan. Isolated plan-task execution (v0.3.79): checklist tasks run in a fresh isolated worker session with only the task brief, isolation defaults ON, Ralph verification runs in the session's own directory, and the plan gate's RequireApproval respects auto_approve. Scoped memory search (v0.3.80): memory_search takes a scope that searches daily logs, brain files, or both, with brain files indexed on write. Browser inventory mode (v0.3.80): bare browser_find enumerates every clickable element on the page. In-tree memory store (v0.3.81): the qmd dependency is dropped, SQLite FTS5 + vector search owned in-tree. External index paths (v0.3.81): memory search reads indexes outside the profile directory. Chunk-hash caching (v0.3.82): memory skips re-embedding chunks whose content has not changed. Plan hardening (v0.3.82): task-outcome beliefs are scoped to their plan id, stale pre-init markers expire after five minutes, and template warnings reach the agent as a retry nudge instead of being swallowed. Sub-agent worktree isolation (v0.3.83): each child agent gets its own worktree and branch so a fan-out cannot clobber the shared tree. Type-aware acceptance criteria (v0.3.83): criteria are enforced against the toolchain the project actually uses, plans verify with the project's own commands instead of always cargo, and the project is found from the session's folder. A2A sessions resume by context id (v0.3.83). Notify delivery verdicts (v0.5.0): session_notify returns a machine-readable send result, depth-3 injection receipts make notify_id checkable, and delivery redirects to the session that owns the channel now. Plan designs refused at write time (v0.5.1): a malformed design bounces immediately with the missing sections named. Config writes validated against the struct (v0.5.1): the write guard derives from the Config type itself. Temporal grounding (v0.5.1): every turn carries a time-stamped marker in the user's timezone while the system prompt stays byte-stable for provider caching. ACP server mode (v0.5.2): opencrabs acp speaks the Agent Client Protocol over stdio JSON-RPC, so Zed, MonoCode and any ACP client drive the same brain your channels use, with progress streaming and approvals round-tripping to the editor. ACP session family (v0.5.4): sessions are first-class in editors, with the context meter restored on load, session/load replaying the transcript and the per-session model, set_model persisting across processes, native session/set_mode, session/compact, and a live model catalog in session/new. ACP v1 lifecycle (v0.5.5): session/list, session/resume, session/set_config_option, session/close and session/delete are native now, the plan publishes as a v1 plan update, and initialize advertises the session capabilities.",
        ),
        (
            "🌐",
            "Browser Automation",
            "Native headless Chrome control via CDP. Navigate, click, type, screenshot, find elements by CSS/XPath/text/aria, extract content, run JS. System prompt rules enforce best practices. Per-session tab isolation. Smart detection of your default Chromium browser. Shadow-DOM aware selectors (v0.5.2): selector resolution pierces shadow roots, so automation reaches web components that used to be invisible to the tools.",
        ),
        (
            "📐",
            "CODE.md — Built-in Coding Standards",
            "Ships with a brain template that enforces modular files, strict size limits, test coverage, and security-first patterns. Your agent writes production code, not prototypes.",
        ),
        (
            "🔒",
            "Security & Trust",
            "Confidential file protection: SSH keys, .env, credentials, and private configs are sacred. Never shared without explicit owner verification. Owner impersonation detection catches non-owners trying to act as the owner in Telegram group chats. bot_owner config field with is_owner() helper for identity checks. Owner-only gate for /evolve and /rebuild prevents unauthorized updates. Per-chat ACL: each Telegram group gets its own allow list, closing the 'DM the bot privately to escape group oversight' bypass. WhatsApp response_policy (auto/owner_only/allowlist/open) controls who can interact. /cowork checks bot admin status before creating invite links. rm-blocklist prevents destructive commands with reversed-flag, quoted-path, and chained-flag defenses. Secret redaction scope (v0.3.71): scope redaction to global/group/DM with /redact command. Self-heal phantom detection (v0.3.72): catch zero-tool turns that claim high-stakes side-effects (ship/push/tag/release). TOML safety gates (v0.3.78): runtime-configurable bash blocklist with hot-reload, quoted-secret redaction across all delivery paths, and TOML-driven brain file verification with post-write anchor checks. Owner-gated channel commands (v0.3.80): user-defined slash commands and skills are owner-only on the channel catch-all arm, so group members cannot trigger owner automation. keys.toml key-loss fix (v0.3.81): provider API keys no longer dropped at runtime, one owner for the stored-key marker, poisoned keys self-heal. User-owned brain files protected (v0.3.82): SOUL.md, USER.md and MEMORY.md are never merged into from upstream, so your personality, identity and learned memory stay yours.",
        ),
        (
            "🛡️",
            "Self-Healing",
            "Auto-recovers corrupted config with auto-repair that never poisons last-good. Tracks per-provider health with auto-failover, 65% context budget management with async LLM compaction. Vision fallback chain: scans all enabled providers (Google, OpenRouter, OpenAI-compatible, Anthropic) before returning an error. Cross-channel crash recovery. Stuck stream detection, reasoning repetition loop detection, thinking/reasoning tag stripping from output so reasoning models never leak internal thoughts. 10-min CLI idle timeout, DB integrity checks. Append-only brain files with upstream template sync. System brain auto-rebuilds when brain files change. Expanded phantom detection, RSI escalation for repeat violations, partial JSON repair, TCP keepalive on all HTTP clients. Browser resilience: network idle wait, CDP health checks, lock release before await. Config edits route through config_manager only (v0.3.74): the agent never raw-edits config.toml/keys.toml, and every write is validated before it lands so a malformed line can't take it down. Self-heal hardening (v0.3.75): bounded phantom self-heal loops, patient mid-stream retry with exponential backoff, flaky-provider 404s retried instead of treated as permanent, multilingual image-hallucination detection, and auto-recovery from repetitive-tool-call poisoned sessions. Thinking-loop timeout (v0.3.78): catches infinite reasoning loops and fires phantom enforcement retry with sticky fallback. Expanded phantom detection (v0.3.76-v0.3.78): file-delivery claims, null-effect tool calls, command verification, evidence checking, shipped-and-tracked claims, colloquial delivery claims, and media-delivery claims all caught. Quota circuit breaker (v0.3.79): providers that exhaust their quota get a TTL circuit breaker and are skipped in fallback walks, chain exhaustion is reported explicitly, and retry notices distinguish hard quota exhaustion from transient throttles and name the provider and model. Loop guards (v0.3.80): cross-turn announcement and near-match repetition guards generalized to all tools, and loop-detector kills reach the fallback chain so a stalled turn retries on another provider instead of dying. Early loop detection (v0.3.81): a repeating tool round is caught before the provider rejects the turn. Logger reliability (v0.3.82): the logger stops dropping events and writing to the TUI's terminal, and a stalled write can no longer silence every other thread's logging. Silent-error sweep (v0.3.82): 112 places that discarded an async failure now report it. doctor --fix (v0.3.83): stuck cron rows, stale markers and broken permissions are repaired on the spot instead of only reported. Per-path write locks (v0.3.83): concurrent writes to the same file serialize so the last writer no longer silently wins. The phantom detector rolls the whole chain and never delivers work that was not done (v0.3.83). Background compaction (v0.5.0): the summariser runs off the turn instead of holding the session while it works. Token redaction at the writer (v0.5.0): a provider error carrying a URL can no longer leak the bot token into a log, wherever it is logged from. Worker stack (v0.5.11): the agent's worker threads get a 16 MiB stack, so the first turn can no longer abort the process on the default 2 MiB one. Release gating (v0.5.11): the release workflow reads the conclusion of the commit's own CI run before it publishes anything, so a red main can never ship a tag again. Evolve restart (v0.5.11): the scheduled restart resolves exact unit names instead of handing systemctl a glob that also matched its own transient unit, and an unreadable unit list reports Unresolved instead of scheduling blind.",
        ),
        (
            "🔧",
            "Self-Improving",
            "Recursive Self-Improvement (RSI) — the agent analyzes its own performance via a persistent feedback ledger, identifies failure patterns, and autonomously updates its own brain files. Proposes new tools, commands, and skills via Mission Control inbox for human review. RSI efficiency gate requires proposals to state TOKEN SAVINGS, ERROR REDUCTION, or CAPABILITY ADDITION. Per-project brain overlay layers project-specific files on top of profile brain. AGENTS.md is always-loaded so hard rules are enforced every turn. Runtime commands and skills index injected so the agent sees live tools. Migrate from OpenClaw, Hermes, or any other tool with built-in CLI migration. Append-only protection, upstream template sync. Skill candidates from tool sequences: RSI detects recurring tool sequences as skill candidates. Slash commands from repeated asks: RSI proposes slash commands from repeated user requests. RSI staleness indicator in Mission Control. Brain dedup (v0.3.75): a /dedup command plus post-write and weekly cross-file scans keep brain files from drifting into duplication, with Telegram inline-keyboard approval. Brain-file hints on tool misses (v0.3.75): when a tool call misses or errors, the relevant brain notes surface automatically. RSI convergence gates (v0.3.80): cycles pause on convergence self-reports, the interval backs off on zero-improvement streaks, and findings are hashed by stable identity. RSI engine gate (v0.3.81): autonomous RSI is off by default for headless daemons, opt in with rsi_enabled. Upstream sync respects ownership (v0.3.82): SOUL.md, USER.md and MEMORY.md are excluded from template sync, and directives route to AGENTS.md rather than the on-demand MEMORY.md where they would not bind on a cold session.",
        ),
        (
            "👤",
            "Multi-Profile",
            "Run multiple isolated instances from one binary. Each profile gets its own config, brain files, sessions, and daemon service. Token-lock isolation prevents two profiles from fighting over the same bot. All paths are profile-aware: RSI state, tmp purge, config edits all route through the active profile home. /profiles command to manage profiles from any channel. Native TUI profiles dialog with browse, create, delete, and migrate flows. Rich profile rendering on Discord, WhatsApp, and Slack. Export/import profiles as portable archives.",
        ),
        (
            "📁",
            "Projects",
            "Organize work into projects with dedicated sessions. Assign sessions with a single keypress. Per-project colours and badges for visual organization. Per-project brain overlay layers project-specific files on top of profile brain. Project file shares: symlink local copies, copy ephemeral ones into project files. Project-scoped file artifacts archived under projects/<name>/files/. Shared images from Telegram archived automatically. Full CRUD UI with SQLite-backed persistence.",
        ),
        (
            "📋",
            "Project Directive Discovery",
            "Auto-discovers rule files from other AI coding tools in any repo. CLAUDE.md, GEMINI.md, .cursorrules, .windsurfrules, .clinerules, .github/copilot-instructions.md, and more. No config, no import step: if the files are there, the agent knows. Index rebuilds on /cd so directives are always current.",
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
            "It's easy to become obsessed w/ crabs, I already feel it.",
            "Roman Gall · Telegram DM",
            "",
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

// ── Newsletter (coming soon — self-hosted on droplet with Resend) ───────────
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

// ── Footer ──────────────────────────────────────────────────────────────────

#[component]
fn Footer(stars: Signal<u32>) -> impl IntoView {
    let i18n = use_i18n();
    let comm_open = RwSignal::new(false);
    let star_cta = move || {
        let count = stars.get();
        if count > 0 {
            t!(i18n, footer_stars, count = count).into_any()
        } else {
            t!(i18n, footer_star_zero).into_any()
        }
    };

    view! {
        <footer>
            <div class="container">
                <a href="https://github.com/opencrabs/opencrabs" class="footer-star-cta" target="_blank">
                    {star_cta}
                </a>
                <div class="footer-links">
                    <a href="https://docs.opencrabs.com" target="_blank">{move || t!(i18n, footer_docs)}</a>
                    <a href="https://github.com/opencrabs/opencrabs">{move || t!(i18n, footer_github)}</a>
                    <a href="https://github.com/opencrabs/opencrabs/blob/main/CHANGELOG.md">{move || t!(i18n, footer_changelog)}</a>
                    <a href="/stats/">{move || t!(i18n, footer_stats)}</a>
                    <span class="footer-comm-wrap">
                        <a
                            href="#"
                            class="footer-comm-toggle"
                            aria-haspopup="true"
                            aria-expanded=move || if comm_open.get() { "true" } else { "false" }
                            on:click=move |ev: leptos::ev::MouseEvent| { ev.prevent_default(); comm_open.update(|o| *o = !*o); }
                        >
                            {move || t!(i18n, footer_community)}
                        </a>
                        <span class="footer-comm-popover" class:open=comm_open>
                            <a href="https://t.me/opencrabs" target="_blank" rel="noopener noreferrer">
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" aria-hidden="true"><path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/></svg>
                                {move || t!(i18n, footer_telegram)}
                            </a>
                            <a href="https://discord.gg/dJwCRawb6h" target="_blank" rel="noopener noreferrer">
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" aria-hidden="true"><path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"/></svg>
                                {move || t!(i18n, footer_discord)}
                            </a>
                        </span>
                    </span>
                    <a href="https://github.com/sponsors/adolfousier" target="_blank">{move || t!(i18n, footer_sponsor)}</a>
                    <a href="https://buymeacoffee.com/opencrabs" target="_blank">{move || t!(i18n, footer_coffee)}</a>
                    <a href="https://github.com/opencrabs/opencrabs/blob/main/LICENSE">{move || t!(i18n, footer_license)}</a>
                </div>
                <p class="footer-tagline">
                    {move || t!(i18n, footer_built_by)}
                    <a href="https://github.com/adolfousier">"Adolfo Usier"</a>
                    {move || t!(i18n, footer_and_community)}
                </p>
            </div>
        </footer>
    }
}

#[cfg(test)]
mod tests {
    const CSS: &str = include_str!("../style.css");

    // Regression pin for issue #35: the mobile hamburger rule has to win on
    // specificity, not on source order. The base .nav-toggle { display: none }
    // rule sits below the media queries in style.css, and media queries add no
    // specificity, so an equally specific mobile rule loses at every width.
    #[test]
    fn mobile_hamburger_rule_beats_later_base_rule() {
        let media_at = CSS
            .find("@media (max-width: 768px)")
            .expect("mobile media query");
        let base_at = CSS.find("\n.nav-toggle {").expect("base .nav-toggle rule");
        assert!(
            base_at > media_at,
            "expected the base .nav-toggle rule below the media block"
        );
        assert!(
            CSS[media_at..base_at].contains(".nav-actions .nav-toggle { display: flex; }"),
            "the 768px block must show the toggle through a 2-class selector"
        );
    }

    #[test]
    fn mobile_nav_panel_opens_anchored_right() {
        for (name, css) in [
            ("landing style.css", CSS),
            (
                "download page inline CSS",
                include_str!("root-files/download/index.html"),
            ),
        ] {
            let media_at = css
                .find("@media (max-width: 768px)")
                .unwrap_or_else(|| panic!("{name}: mobile media query"));
            let block_at = css[media_at..]
                .find(".nav-links {")
                .unwrap_or_else(|| panic!("{name}: mobile .nav-links rule"));
            let block = &css[media_at + block_at..];
            let end = block
                .find('}')
                .unwrap_or_else(|| panic!("{name}: rule end"));
            let block = &block[..end];
            assert!(
                block.contains("left: auto") && block.contains("right: 24px"),
                "{name}: panel must anchor right under the hamburger"
            );
            assert!(
                block.contains("width: max-content") && block.contains("min-width: 240px"),
                "{name}: panel must size to content with a sane floor"
            );
            assert!(
                !block.contains("border-bottom"),
                "{name}: floating panel takes a full border, not a bottom-only edge"
            );
        }
    }

    // The public stats page is reachable from the site chrome, and the caller
    // count rides in the nav. Both are easy to lose in a redesign, so pin the
    // link and the badge on every page that carries the header.
    #[test]
    fn stats_links_and_caller_badge() {
        const DOWNLOAD: &str = include_str!("root-files/download/index.html");
        const CHROME: &str = include_str!("../stats/src/chrome.rs");

        // The landing assertions read this very file, so the needles are built
        // at runtime. Written as literals they would match the assertions
        // themselves and pass even after the markup was deleted.
        let landing = include_str!("main.rs");
        let footer_link = format!("<a href=\"{}\">", "/stats/");
        let nav_badge = format!("class=\"{}\"", "nav-callers");
        assert!(
            landing.contains(&footer_link),
            "landing: footer must link to /stats/"
        );
        assert!(
            landing.contains(&nav_badge),
            "landing: nav must render the caller badge"
        );

        for (name, page) in [("download page", DOWNLOAD), ("stats chrome", CHROME)] {
            assert!(
                page.contains(&footer_link),
                "{name}: footer must link to /stats/"
            );
            assert!(
                page.contains(&nav_badge),
                "{name}: nav must carry the caller badge"
            );
        }

        assert!(
            DOWNLOAD.contains("/stats-data/public.json"),
            "download page: the badge must read the public snapshot"
        );

        // The pill styling has to exist or the badge renders as bare text.
        assert!(
            CSS.contains(".nav-callers {") && CSS.contains(".nav-callers::before"),
            "landing style.css: the caller pill needs its rule and its dot"
        );
    }
}
