# Browser Automation

OpenCrabs includes native headless Chrome control via the Chrome DevTools Protocol (CDP). No Selenium, no Playwright — direct browser control built into the binary.

## Requirements

- A Chromium-based browser installed (Chrome, Brave, Edge, or Chromium)
- Feature flag: `browser` (enabled by default)

OpenCrabs auto-detects your default Chromium browser — no manual path configuration needed.

## Browser Tools

| Tool | Description |
|------|-------------|
| `navigate` | Open a URL in the browser |
| `click` | Click an element by CSS selector |
| `type` | Type text into an input field |
| `screenshot` | Capture a screenshot of the page |
| `eval_js` | Execute JavaScript in the page context |
| `extract_content` | Extract text content from elements |
| `wait_for_element` | Wait for an element to appear |
| `find` | Find elements matching a pattern (CSS, XPath, text, or aria-label). Returns stable selectors for subsequent click/type operations |
| `browser_act` | Batched multi-action in one call: an ordered array of `click` / `fill` / `press` / `select` / `wait` / `upload` (max 10 actions, 120s budget). Selectors resolve up front, so a stale reference rejects the whole batch instead of half-running it; aborts on first failure and reports the completed prefix; one screenshot at the end. `upload` attaches local files to a file input without putting their bytes in the context (#2112) |

## How It Works

The browser is lazy-initialized as a singleton — it only launches when the agent first needs it. It runs in stealth mode with a persistent profile directory, so cookies and sessions survive across tool calls.

On macOS, display auto-detection enables headed mode when a display is available, falling back to headless in CI or daemon environments.

## Example

Ask the agent:

> "Go to our staging site, log in with the test account, navigate to the dashboard, and take a screenshot"

The agent will chain `navigate` → `type` (username) → `type` (password) → `click` (login button) → `navigate` (dashboard) → `screenshot` — all autonomously.

## Inventory Mode (v0.3.80)

`browser_find` now works **without a pattern** (#1022). Call it bare and it enumerates every visible interactive element on the page — links, buttons, inputs, selects, textareas, role=button elements, onclick handlers, tabindex elements, labels, and disclosure widgets — each stamped with a selector and index you can click directly.

This is the structural fix for the screenshot-as-discovery loop: instead of screenshotting repeatedly and guessing selectors, the agent gets the page's clickable inventory in one call. When `browser_screenshot` reports an identical page, the hint now nudges toward inventory mode.

## Shadow-DOM Resolution (v0.5.2)

Selector resolution is shadow-DOM aware: an element inside one or more shadow roots is reached by piercing the roots along the path, so web components and widget libraries that host their UI in shadow trees are automatable like light-DOM content. Applies across the browser tools (click, type, find, screenshot targeting).

## Configuration

No configuration needed. The browser feature is enabled by default. To disable it at build time:

```bash
cargo build --release --no-default-features --features "telegram,discord,slack"
```
