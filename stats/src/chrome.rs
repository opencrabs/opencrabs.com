//! Site chrome shared with the landing page.
//!
//! The dashboard is its own crate so it can be built and served separately, so
//! it cannot import the landing's components. These are the same markup and the
//! same class names, copied so the fixed header and the footer look and behave
//! identically on both pages. Labels are English only: this page has no locale
//! switcher, because the numbers on it are English-only anyway.
//!
//! Every link that leaves /stats/ is absolute. A relative href would resolve
//! against /stats/ and 404.

use gloo_net::http::Request;
use leptos::prelude::*;
use serde::Deserialize;

const REPO: &str = "https://github.com/opencrabs/opencrabs";

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

/// Star count as a signal, 0 until the request lands.
pub fn star_signal() -> Signal<u32> {
    let stars = LocalResource::new(fetch_star_count);
    Signal::derive(move || stars.get().flatten().unwrap_or(0))
}

#[component]
pub fn Nav(stars: Signal<u32>, callers: Signal<u64>) -> impl IntoView {
    let menu_open = RwSignal::new(false);
    let star_label = move || {
        let count = stars.get();
        if count > 0 {
            format!(" ★ {count}")
        } else {
            String::new()
        }
    };

    view! {
        <nav>
            <div class="container">
                <a href="/" class="nav-logo">
                    <img class="crab-icon" src="/public/opencrabs-logo.png" alt="" />
                    "OpenCrabs"
                </a>
                <ul class="nav-links" class:open=menu_open on:click=move |_| menu_open.set(false)>
                    <li><a href="https://docs.opencrabs.com" target="_blank">"Docs"</a></li>
                    <li><a href="/#features">"Features"</a></li>
                    <li><a href="/#integrations">"Works With Everything"</a></li>
                    <li><a href="/download/" class="nav-download">"Download"</a></li>
                    <li>
                        <a href=REPO class="btn-github" target="_blank">
                            "GitHub"
                            <span class="github-stars">{star_label}</span>
                        </a>
                    </li>
                    <li>
                        // The same caller pill the landing carries, fed by the
                        // snapshot this page already loaded. Hidden until the
                        // count is known so it never claims a measured zero.
                        {move || {
                            let n = callers.get();
                            if n == 0 {
                                ().into_any()
                            } else {
                                view! {
                                    <a href="/stats/" class="nav-callers">
                                        {format!("{} callers", crate::format::thousands(n))}
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

#[component]
pub fn Footer(stars: Signal<u32>) -> impl IntoView {
    let comm_open = RwSignal::new(false);
    let star_cta = move || {
        let count = stars.get();
        if count > 0 {
            format!("★ {count} stars on GitHub, give us one more!")
        } else {
            "★ Star us on GitHub!".to_string()
        }
    };

    view! {
        <footer>
            <div class="container">
                <a href=REPO class="footer-star-cta" target="_blank">
                    {star_cta}
                </a>
                <div class="footer-links">
                    <a href="https://docs.opencrabs.com" target="_blank">"Docs"</a>
                    <a href=REPO>"GitHub"</a>
                    <a href="https://github.com/opencrabs/opencrabs/blob/main/CHANGELOG.md">"Changelog"</a>
                    <a href="/stats/">"Stats"</a>
                    <span class="footer-comm-wrap">
                        <a
                            href="#"
                            class="footer-comm-toggle"
                            aria-haspopup="true"
                            aria-expanded=move || if comm_open.get() { "true" } else { "false" }
                            on:click=move |ev: leptos::ev::MouseEvent| { ev.prevent_default(); comm_open.update(|o| *o = !*o); }
                        >
                            "Community"
                        </a>
                        <span class="footer-comm-popover" class:open=comm_open>
                            <a href="https://t.me/opencrabs" target="_blank" rel="noopener noreferrer">
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" aria-hidden="true"><path d="M11.944 0A12 12 0 0 0 0 12a12 12 0 0 0 12 12 12 12 0 0 0 12-12A12 12 0 0 0 12 0a12 12 0 0 0-.056 0zm4.962 7.224c.1-.002.321.023.465.14a.506.506 0 0 1 .171.325c.016.093.036.306.02.472-.18 1.898-.962 6.502-1.36 8.627-.168.9-.499 1.201-.82 1.23-.696.065-1.225-.46-1.9-.902-1.056-.693-1.653-1.124-2.678-1.8-1.185-.78-.417-1.21.258-1.91.177-.184 3.247-2.977 3.307-3.23.007-.032.014-.15-.056-.212s-.174-.041-.249-.024c-.106.024-1.793 1.14-5.061 3.345-.48.33-.913.49-1.302.48-.428-.008-1.252-.241-1.865-.44-.752-.245-1.349-.374-1.297-.789.027-.216.325-.437.893-.663 3.498-1.524 5.83-2.529 6.998-3.014 3.332-1.386 4.025-1.627 4.476-1.635z"/></svg>
                                "Telegram"
                            </a>
                            <a href="https://discord.gg/dJwCRawb6h" target="_blank" rel="noopener noreferrer">
                                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" aria-hidden="true"><path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z"/></svg>
                                "Discord"
                            </a>
                        </span>
                    </span>
                    <a href="https://github.com/sponsors/adolfousier" target="_blank">"Sponsor"</a>
                    <a href="https://buymeacoffee.com/opencrabs" target="_blank">"Buy Me a Coffee"</a>
                    <a href="https://github.com/opencrabs/opencrabs/blob/main/LICENSE">"MIT License"</a>
                </div>
                <p class="footer-tagline">
                    "Built with 🦀 by "
                    <a href="https://github.com/adolfousier">"Adolfo Usier"</a>
                    " & the OpenCrabs community."
                </p>
            </div>
        </footer>
    }
}
