# Web Scraping

OpenCrabs includes a native URL-to-markdown scraping tool (`web_scrape`) that converts any web page into clean markdown. Zero AI cost, zero API tokens for the extraction itself. The agent uses `tool_search` to activate it on demand, keeping it out of the always-loaded core tool set.

## How It Works

`web_scrape` follows a five-step pipeline:

1. **Validate**: SSRF guard rejects private IPs, localhost, loopback, link-local addresses, cloud metadata endpoints (169.254.169.254), and non-http(s) schemes
2. **Fetch**: `reqwest` with browser User-Agent and timeout. `is_js_shell` heuristic detects JS-heavy pages (React/Vue/Angular/Svelte shells, `<div id="app">`, no `<article>`). Escalates to browser manager when available
3. **Extract**: CSS selector cascade (article, main, .content, etc.) isolates primary content from HTML, falling back to body with junk selectors (header, nav, sidebar, ads) removed
4. **Clean**: Language-agnostic HTML cleaner strips scripts, styles, inline handlers, HTML comments. Decodes entities, collapses blank lines
5. **Convert**: `htmd` converts cleaned HTML to markdown. `absolutize_urls` resolves relative src/href against page base URL. Images preserved as `![alt](url)` tags for selective agent vision

## Single URL Mode

Scrape a single URL and return the markdown content directly:

```
web_scrape https://example.com/page
```

The agent receives the markdown in its context, ready for analysis, summarisation, or extraction.

## Sitemap Mode

Discover and crawl an entire site via its sitemap:

```
web_scrape https://example.com --sitemap
```

This discovers `/sitemap.xml`, `/sitemap_index.xml`, and common variations. Recursively crawls sitemap indexes (iterative worklist, 1000-URL cap, 3 levels deep). Returns the URL list for the agent to pick from.

Each page's markdown is exported to a directory. The output path resolves to the project files directory if the session is assigned to a project, or the profile-scoped OpenCrabs home otherwise. Files never land outside managed workspace.

**Pair it with memory:** point `[memory] extra_paths` at the export directory and every scraped page becomes searchable via `memory_search scope="external"`, a zero-cost knowledge base your agent quotes verbatim. Full walkthrough in [Memory System → Build a knowledge base](../brain/memory.md#build-a-knowledge-base-from-any-docs-site). Note the hard cap: 100 pages per sitemap scrape.

## SSRF Protection

The SSRF guard uses `url::Host` enum classification to properly handle:

| Blocked | Reason |
|---------|--------|
| Private IPs (10.x, 172.16-31.x, 192.168.x) | Internal network |
| Localhost (127.0.0.1, ::1) | Local services |
| Link-local (169.254.x.x) | Cloud metadata, DHCP |
| Non-http(s) schemes | file://, ftp://, gopher:// |

This runs before any network request, so malicious URLs never reach the fetcher.

## JS-Shell Detection

When a page returns mostly empty HTML with JS framework markers (React root, Vue app, Angular bootstrap, Svelte kit), `web_scrape` detects it as a JS shell and escalates to the browser manager if available. This avoids returning empty markdown for single-page applications that require JavaScript rendering.

## Image Handling

Images are preserved as `![alt](url)` markdown tags rather than being stripped. This lets the agent decide which images are worth visioning (via `analyze_image`) and which can be ignored, rather than losing all visual context.
