# Introduction

**OpenCrabs** is a self-hosted, provider-agnostic AI orchestration agent that runs as a single Rust binary. It automates your terminal, browser, channels (Telegram/Discord/Slack/WhatsApp/Trello), and codebase, all while respecting your privacy and keeping you in control.

**11,407 tests** across providers, tools, channels, TUI, self-healing, and browser automation.

## What Makes OpenCrabs Different

### Zero Telemetry, Not Even Opt-In
- **OpenCrabs does not phone home. Ever.** No analytics, no tracking, no usage statistics, no remote logging, no crash reports
- Your conversations, tools, memory, configuration, and API keys never leave your machine
- The only outbound traffic is what you explicitly initiate: LLM API calls, web searches, GitHub commands, browser automation
- Not a privacy-policy checkbox: there is no telemetry code to disable, no opt-out flag, no analytics service to block. There is simply nothing to send

### Provider-Agnostic by Design
- **18 built-in providers + Custom OpenAI Compatible**: OpenCrabs (built in, free), Anthropic Claude, OpenAI, Gemini, Xiaomi MiMo, OpenRouter, Qwen (DashScope), MiniMax, Ollama, z.ai GLM, GitHub Copilot, Codex, Codex CLI, OpenCode, OpenCode CLI, **Moonshot AI (Kimi)**, Command Code CLI, Claude CLI
- **Native CLI integration**: use Claude Code CLI, OpenCode CLI, Codex CLI, and Command Code CLI as providers without API keys
- **Sticky fallback chain**: auto-failover on rate limits or errors, health-aware persistence survives restarts
- **Prompt caching** across Anthropic, OpenRouter, Gemini, Qwen DashScope reduces costs up to 95%
- **Context window override**: cap or expand context for any model via config (the built-in OpenCrabs provider is the exception: its window is fixed at 200k)
- **Xiaomi MiMo**: 30+ models including the MiMo reasoning series, keyed provider
- **Moonshot AI (Kimi)**: API plan (api.moonshot.ai) and Coding plan (api.kimi.com) endpoints, reasoning-effort config, plan-tier context window

### Multi-Agent Orchestration
- **Sessions are fully isolated agents**: each with its own brain, provider, model, working directory, and history
- **Typed sub-agents**: `general`, `explore`, `plan`, `code`, `research` with tailored tool access
- **Team orchestration**: `team_create`, `team_broadcast`, `team_delete` for coordinated workflows
- **Per-call provider/model overrides**: mix models across teams (plan with GLM, code with Deepseek, review with Kimi)
- **A2A protocol**: JSON-RPC 2.0 gateway for agent-to-agent communication

### Channel-Native Communication
- **Telegram, Discord, Slack, WhatsApp, Trello**, full bot integration with DMs, groups, and threads
- **Telegram rich messages**: native tables, headings, lists, math via `rich_messages` config
- **Draft message streaming**: live "typing..." updates as tokens generate in DMs
- **Collapsible blocks**: `<details>/<summary>` sections for long outputs
- **Forum topic session isolation**: each topic in Telegram supergroups gets its own session
- **Telegram reactions**: the bot reads inbound emoji reactions and can reply with a reaction instead of a message when that fits
- **Frame reactions (v0.3.61)**: inbound reactions read by sentiment, agent addresses user by first name
- **Mid-turn reactions (v0.3.61)**: a reaction during a running turn injects into the current loop instead of firing a second turn
- **Follow-up suggestions (v0.3.69)**: tap-to-send optional follow-ups on Telegram, Discord, Slack, WhatsApp; fill-not-submit in TUI
- **Telegram plan card re-stick (v0.3.71)**: re-stick plan card and fold prose into it when buried
- **Full group history capture (v0.3.73)**: persist every group message to history, even from non-allowlisted senders and bots
- **Correct group speaker (v0.3.73)**: stop addressing a group-history sender as the current speaker
- **Session inheritance**: `/new` inherits the working directory from your most recent session
- **`/goal` across all channels**: set autonomous goals from Telegram, Discord, Slack, WhatsApp
- **Voice support**: local Whisper STT + Piper TTS, fully offline
- **Cross-channel crash recovery**: pending requests route back to originating channel on restart
- **`/cowork`**: create shared workspaces from channels and TUI
- **`/rename`, `/profiles`, `/cd`**, manage sessions, profiles, and directories from any channel

### Self-Healing & Self-Improvement
- **Recursive Self-Improvement (RSI)**: agent analyzes performance, identifies patterns, and rewrites brain files
- **Phantom tool call detection**: catches when the model narrates changes without executing tools
- **Self-heal phantom detection (v0.3.72)**: catch zero-tool turns that claim high-stakes side-effects (ship/push/tag/release)
- **Config edits via config_manager only (v0.3.74)**: the agent never raw-edits config.toml/keys.toml; every write is validated before it lands
- **Brain dedup (v0.3.75)**: `/dedup` command plus post-write and weekly cross-file scans keep brain files from drifting into duplication, with Telegram inline-keyboard approval
- **Brain-file hints on tool misses (v0.3.75)**: relevant brain notes surface automatically when a tool call misses or errors
- **Proactive memory surfacing (v0.3.76)**: relevant memory surfaces without being asked when conversation context matches stored knowledge
- **Natural model naming (v0.3.76)**: `/models opus` works; colloquial names resolve to exact model IDs
- **Plan/execute provider routing (v0.3.76)**: route planning and execution onto separate providers/models
- **Thinking-loop timeout (v0.3.78)**: catches infinite reasoning loops, fires phantom enforcement with sticky fallback
- **Epistemic engine (v0.3.78)**: belief tracking with confidence levels, OODA/BDI cognitive mesh, Orient gate before plan execution
- **Ralph verification gate (v0.3.78)**: criteria-aware task verification with iteration cap; tasks must meet stated acceptance criteria
- **TOML safety gates (v0.3.78)**: runtime-configurable bash blocklist with hot-reload, quoted-secret redaction, brain file verification
- **Mission Control analytics (v0.3.78)**: D/W/M filter, 3-across card grid, phantom/model/streaming tabs, dedicated events DB
- **OpenAI TTS onboarding (v0.3.78)**: voice selector and API key field in the onboarding wizard
- **Homebrew install (v0.3.79)**: `brew install opencrabs` from homebrew-core: prebuilt binary, both architectures, no Rust toolchain
- **`/restart` and `/exit` (v0.3.79)**: restart or shut down from any channel or the TUI; owner-only on channels, announce before acting
- **Quota circuit breaker (v0.3.79)**: quota-exhausted providers get a TTL breaker and are skipped in fallback walks; chain exhaustion reported explicitly
- **Plan state across sessions (v0.3.79)**: plans survive session boundaries; isolated plan-task execution defaults on, Ralph verifies in the session's own directory
- **Theme system (v0.5.0)**: verified presets, an interactive picker, and your own themes from `~/.opencrabs/themes/*.toml`; truecolor is detected with an ANSI-256 fallback tier, and a preset whose contrast would make text unreadable is refused
- **Structural code memory (v0.5.0)**: tree-sitter builds a symbol and call-graph index beside the text index, so "who calls X" walks real call edges while conceptual questions keep the text lane (#1324)
- **Drop transfer over SSH (v0.5.0)**: a file dropped into a TUI running on a remote host is pulled back through the connection already open, with an `opencrabs drop-agent` on the client side (#1289)
- **Notify delivery verdicts (v0.5.0)**: `session_notify` returns a machine-readable send result, depth-3 injection receipts make `notify_id` checkable, and delivery redirects to the session that owns the channel now
- **Background compaction (v0.5.0)**: the summariser runs off the turn instead of holding the session while it works
- **Token redaction at the writer (v0.5.0)**: a provider error carrying a URL can no longer leak the bot token into a log, wherever it is logged from (#1322)
- **Receive-only Telegram userbot (v0.5.0, experimental)**: feature-gated MTProto capture with local QR/code/2FA login; it stores allowlisted text for retrieval and never acts as the user
- **Temporal grounding (v0.5.1)**: the agent always knows what time it is: every turn carries a time-stamped marker in the user's timezone, and the system prompt stays byte-stable for provider caching (#1516)
- **Per-job cron sessions (v0.5.1)**: every scheduled job gets its own session, so two jobs running at the same time can never bleed into each other's context or providers (#1511)
- **Skill-glob gating (v0.5.1)**: a skill's `globs:` list can refuse tool calls that touch matching files until the agent has actually read the skill body (#1515)
- **Config writes validated against the struct (v0.5.1)**, the write guard and section suggestions in config_manager now derive from the `Config` type itself, so they never drift from shipped code (#1508)
- **Plan designs refused at write time (v0.5.1)**: a malformed plan design bounces immediately with the missing sections named, and a refused write restores the scaffold instead of wiping the draft (#1507)
- **Provider budget backpressure (v0.5.1)**: a provider that reports an output-token budget triggers one bounded compaction and retry instead of a cascade of 400s (#1518)
- **A2A tokens bound and config owner-only (v0.5.1)**: A2A tokens bind to the gateway that minted them, and config.toml/keys.toml move to owner-exclusive mode with secrets scrubbed from the logs
- **ACP server mode (v0.5.2)**: `opencrabs acp` speaks the Agent Client Protocol over stdio JSON-RPC, so Zed, MonoCode and any ACP client drive the same brain your channels use, with progress streaming and approvals round-tripping to the editor (#1540)
- **Shadow-DOM aware browsing (v0.5.2)**: selector resolution pierces shadow roots, so automation reaches web components that used to be invisible to the tools
- **WhatsApp full surface (v0.5.2)**: native voice notes, inbound video/sticker/location/contact, poll votes, presence and media recovery, reaction acks, edit-in-place streaming, disappearing messages, contact blocking, pin/forward/profile, status updates, newsletter discovery, native-flow buttons, per-chat history
- **Discord live tracing and native tables (v0.5.2)**: long turns stream as a traced live reply, answers auto-thread, markdown tables render natively, and tool turns no longer double-post the final response (#1603, #1608)
- **Telegram forum topics and /clear (v0.5.2)**: forum topic create/rename with session binding, and /clear starts a fresh session without the summariser call (#1585)
- **Command menu payload budget (v0.5.2)**: Telegram's setMyCommands silently rejects request bodies over ~7.8KB while reporting BOT_COMMANDS_TOO_MUCH; descriptions now shorten in tiers so the whole menu fits (#1613)
- **Trigger-gated cron (v0.5.3)**: a job runs a pre-flight check and only executes when your condition is met (non-empty output, exit code, regex), skipped runs cost 0 tokens, and a fired trigger can dispatch a goal into a session (#233)
- **First-class Windows (v0.5.3)**: platform shell handling, BOM/UTF-16 file reads, tilde and cwd fallbacks, with CI building Windows and gating PRs (#627)
- **Epistemic memory (v0.5.3)**: MEMORY.md facts carry confidence and decay (#1643), cold sections archive themselves, and `opencrabs memory prune` previews what went cold before touching it (#1657)
- **Finished theme engine (v0.5.3)**: every TUI widget resolves through the theme with a suite guard against new raw ANSI, and themes can declare a canvas background (#1634)
- **WhatsApp suggestion polls (v0.5.3)**: follow-up option sets past the native button cap render as a poll, and a vote selects the option (#1616)
- **ACP session family (v0.5.4)**: sessions are first-class on the ACP surface: the context meter is restored on load and rides usage updates, `session/load` replays the transcript and restores the per-session model, `set_model` persists across processes, native `session/set_mode` applies the approval policy server-side, `session/compact` pushes, and `session/new` offers a live model catalog
- **Interactive terminal handoff (v0.5.4)**: `!vi`/`!nano` take over the terminal interactively and hand it back with the session intact; Unix terminals only, Windows keeps pipe capture (#1744, #1759)
- **Universal paste & drop (v0.5.4)**: one shared attachment router accepts every file type on paste or drop and classifies code, docs, archives and data so each lands in the right pipeline (#1740, #1743)
- **Serper search engine (v0.5.4)**: `serper_search` adds Google SERP results, and the web_search fan-out dedupes by URL across engines (#1731)
- **Decision cache (v0.5.4)**: `decide_cached` answers repeat decisions from an L1 reuse ring with shadow/live tiers and `[decisions]` config (#1648)
- **Provider-reported costs and one timeout chain (v0.5.4)**, the usage ledger prefers the cost the provider actually reported over table estimates (#1707), and timeout flags resolve `[providers.<name>]` -> `[agent]` -> default across every provider (#1688)
- **Compiled /architecture and /attach, opt-in audit trail (v0.5.4)**, both commands ship as compiled arms, and an opt-in audit trail records what ran with a `/audit` viewer (#933, #1705)
- **Discord native surface (v0.5.5)**: polls via `send_poll`, TTS answers delivered as voice messages, and `commands.toml` projected onto native slash commands so your own commands show up in Discord's picker (#1848, #1849, #1850)
- **Flow groups and write governors (v0.5.5)**: Discord and Slack open the flow group at turn start with a live clock covering thinking and activity text leading the line, and one outbound governor budgets every edit and parks on 429s (#1841, #1843, #1844, #1845, #1807, #1808, #1809, #1910, #1912, #2012)
- **/respond_to and /cowork (v0.5.5)**: per-channel respond_to and open modes in config, plus the two commands to set them from inside the channel itself (#2014, #2015)
- **Telegram rate discipline (v0.5.5)**: instrumented getUpdates polls and handler handoffs for stall triage, a cross-surface sub-second spacing floor with admission telemetry, a per-chat 429 pause behind one Cooldown accessor, and markdown links to local files delivered as documents (#1927, #1916, #1968)
- **ACP v1 session lifecycle (v0.5.5)**: the five v1 session lifecycle methods are implemented and the plan is published as a native v1 plan update (#1815)
- **TUI dialog footer, clickable URLs, onboarding timeline (v0.5.5)**: every dialog shares one command footer with direct a/r approval verbs, URLs and file paths are clickable, and onboarding is rebuilt as a left-side step timeline with direct setup commands (#1775, #1772, #1979, #1980, #1981, #1982)
- **Cron one-shot retirement (v0.5.5)**: one-shot jobs retire on delivery instead of re-firing a year later, every delivered run carries its run id, and Discord cron reports arrive as one forum post (#544, #1703, #1851)
- **Zero-setup first run (v0.5.11)**: a fresh install lands in the chat with the built-in provider already provisioned, `config.toml` and `keys.toml` seeded, and the agent greeting you. The wizard is opt-in through `/onboard` or `--onboard` (#2079)
- **A first turn that cannot abort the process (v0.5.11)**: agent worker threads run on a 16 MiB stack, so the first turn no longer overflows the default one (#2080)
- **Release tags gated on green CI (v0.5.11)**: the release workflow reads the conclusion of the commit's own CI run before publishing. A red run fails the release, a run still in flight is waited for, and a commit with no run fails too (#2078)
- **Evolve restarts the right units (v0.5.11)**: the scheduled restart targets resolved unit names instead of the `opencrabs*.service` discovery glob, which also matched the transient unit running the restart and made systemd stop its own executor (#2081)
- **Telegram guests get the full surface (v0.5.12)**: a tag in a chat the bot was never invited to runs the same tool surface, live status block and rich rendering as a normal turn, with one cancellable turn per guest session and rate limiting (#2087, #2109, #2133, #2135, #2138, #2143)
- **A plan is required past a tool-call threshold (v0.5.12)**: once a turn crosses a threshold of tool calls the agent must stop and write a plan before it continues (#2128)
- **The transcript survives a session switch (v0.5.12)**: the fold map is keyed by a stable role and timestamp anchor instead of a UUID minted per render, so switching sessions and coming back no longer folds every turn you had opened (#2155)
- **Headless installs provision the built-in provider (v0.5.12)**: `opencrabs run`, `daemon`, `agent`, `acp` and `cron` mint the built-in token on a fresh home the way the TUI already did, while `status` and `doctor` stay read-only (#2151)
- **Arabic and Chinese language packs (v0.5.14)**: two more packs join the phantom detector, each with its own script detection, and Han characters route to the Chinese pack while kana or hangul keep Japanese and Korean text off that branch (#2222, #2223)
- **Comparable coverage across the packs (v0.5.14)**: the non-English packs now carry intent phrases at the same order of magnitude as English instead of roughly a third of it, and a test pins one caught announcement and one clean sentence per language (#2225)
- **European Portuguese continuous narration (v0.5.14)**: `estou a carregar` style constructions are caught next to the Brazilian gerund, which a lookahead that silently failed to compile had taken down with it (#2224)
- **Byte-safe edit operations (v0.5.14)**: the line operations in `edit_file` no longer drop a file's final newline or rewrite a CRLF file as LF, and `hashline_edit` plus the fuzzy fallback keep the file's own line ending (#2227, #2212)
- **A lint that stops byte-slice panics (v0.5.14)**: CI fails on a new slice that could split a UTF-8 character, added after a multi-byte whitespace bug in the phantom URL check (#2220, #2221)

- **doctor --fix repair mode (v0.3.83)**: stuck cron rows, stale markers and broken permissions are repaired on the spot instead of only reported
- **Per-path write locks (v0.3.83)**: concurrent writes to the same file serialize; the last writer no longer silently wins
- **Sub-agent worktree isolation (v0.3.83)**: each child agent gets its own worktree and branch, so a fan-out cannot clobber the shared tree
- **Type-aware acceptance criteria (v0.3.83)**: plan criteria are enforced against the toolchain the project actually uses (#1133)
- **Project-aware plan verification (v0.3.83)**: plans verify with the project's own commands, not always cargo, and find the project from the session's folder
- **/stop cancels immediately (v0.3.83)**: interrupts during provider handshake and retry backoffs (#1148)
- **A2A session resume (v0.3.83)**: sessions resume by context id, the gateway is profile-addressed (#1159, #1161)
- **DeepSeek thinking knobs (v0.3.83)**: DeepSeek gets the thinking parameters it actually reads, plus tool-call-only turn handling
- **session_id log correlation (v0.3.82)**: every turn opens a tracing span carrying its session id, and cron jobs and the RSI engine get their own, so one grep reconstructs a whole turn out of a shared daily log
- **Logger reliability (v0.3.82)**: the logger stops dropping events and stops writing to the TUI's terminal; a stalled write can no longer silence every other thread's logging
- **User-owned brain files protected (v0.3.82)**: SOUL.md, USER.md and MEMORY.md are never merged into from upstream, and directives route to AGENTS.md, which is always loaded
- **Config section unification (v0.3.82)**: the legacy `[gateway]` spelling is migrated on disk to `[a2a]`, so one name survives a reload round trip
- **Chunk-hash caching (v0.3.82)**: memory skips re-embedding chunks whose content has not changed
- **Pre-release binaries (v0.3.82)**: five platform targets plus SHA256SUMS, published as a rolling pre-release so contributors can test unreleased code without waiting for a tag. Run on demand, not on every push
- **Rate-limit bail (v0.3.82)**: a long Telegram rate-limit window returns immediately instead of parking the send inline
- **In-tree memory store (v0.3.81)**: the qmd dependency is dropped; SQLite FTS5 + vector search owned in-tree
- **External index paths (v0.3.81)**: memory search reads indexes outside the profile directory, default-deny in shared sessions
- **Mermaid as images on Telegram (v0.3.81)**: diagrams render as images while tables stay native formatting
- **Qwen reasoning across turns (v0.3.81)**: preserve_thinking, family-gated thinking knobs, recommended effort tier by default, hosted Qwen reachable everywhere
- **keys.toml key-loss fix (v0.3.81)**, provider API keys no longer dropped at runtime; poisoned keys self-heal
- **Early loop detection (v0.3.81)**: repeating tool rounds caught before the provider rejects the turn
- **RSI engine gate (v0.3.81)**: autonomous RSI off by default for headless daemons, opt in with `rsi_enabled`
- **memory_search scope (v0.3.80)**: search daily logs (`memory`), brain files (`brain`), or both (`all`); brain files are indexed on write
- **Chunked memory retrieval (v0.3.80)**: documents chunked before embedding so later chunks are searchable; recall ranked with BM25
- **Slack-native rendering (v0.3.80)**: tables and headings arrive in Slack's own shape instead of raw markdown
- **Owner-gated channel commands (v0.3.80)**: custom commands and skills are owner-only on the channel catch-all arm in groups
- **RSI convergence gates (v0.3.80)**: self-improvement pauses on convergence, backs off on zero-improvement streaks, hashes findings by stable identity
- **Loop guards (v0.3.80)**: announcement and near-match repetition guards on all tools; loop-detector kills reach the fallback chain
- **Browser inventory mode (v0.3.80)**: bare `browser_find` enumerates every clickable element on the page
- **Brain-file template updates (v0.3.80)**: AGENTS.md, BOOT.md, and SOUL.md templates changed; seeding never overwrites existing files, so diff against `src/docs/reference/templates/` and merge by hand if you want the new defaults
- **System brain rebuild**: brain files rebuilt from disk when changed, no restart needed
- **Proactive tool discovery**: searches for available tools before claiming inability
- **Multilanguage prompt analyzer (v0.3.67)**: shared PromptAnalyzer with soft-nudge on TUI and Telegram, 6 language packs (EN/ES/FR/ID/PT/RU)
- **Plan mode (v0.3.67-v0.3.68)**: design/checklist tracks, persistent plan card, `/plan <query>` command, agent self-approval when user grants autonomy
- **Click-to-expand (v0.3.69)**: left-click a tool-call or reasoning block to expand just that block, complementing Ctrl+O
- **Skills review_gate (v0.3.70)**: frontmatter declaration for high-stakes skills that require user approval before side effects
- **JIT tool activation**: extended tools activated on-demand, no pre-registration needed
- **Config auto-repair**: auto-repair broken config.toml, never poison last-good config
- **Context budget management**: 65% soft / 90% hard compaction thresholds with LLM fallback
- **Stuck stream detection**: 2048-byte rolling window catches repeating patterns
- **Gaslighting defense**: strips tool-refusal preambles mid-turn
- **Deliver build outcomes (v0.3.61)**: rebuild/evolve results reach whoever asked, across channels and TUI

### Terminal UI
- **Native markdown rendering**: emphasis, lists, links, and task items render directly in the terminal
- **Real-time tok/s throughput meter**: live tokens-per-second during streaming
- **Version display (v0.3.73)**: show the running version on the TUI header and in channel /help + /usage
- **3-state reasoning expand (v0.3.74)**: click or Ctrl+O cycles a reasoning block collapsed → capped → full, so it never floods the view
- **Per-turn headers (v0.3.75)**: every turn gets a one-line summary header, its working-out folds into it, and turns fold by default so the view stays clean
- **Group tool calls (v0.3.61)**: consecutive tool calls collapse into one expandable block, keeping the TUI clean during multi-step operations
- **Fold intermediate text (v0.3.61)**: intermediate processing text folds into the same in-place log as tool calls, so only the final answer stays visible
- **Session search**: search filter + viewport scroll across all sessions
- **Split panes**: tmux-style parallel sessions with layout persistence
- **Clipboard image paste**: paste images from browser or any app directly into TUI
- **Plan pinning**: active plan pinned at end of prompt each turn
- **Agent-driven onboarding**: personalized first-time setup with guided flow
- **`/goal` autonomous loop**: set a goal and the agent loops until an LLM judge says it's done, with pause/resume/status controls
- **Self-goaling (v0.3.61)**: agent can set and drive its own multi-turn goals via `goal_manage` tool without user invoking `/goal`

### Developer Experience
- **50+ built-in tools**: file ops, bash, web search, code execution, browser automation, image gen, voice, PDF rendering
- **Unified parallel web search (v0.3.73)**: web_search, exa_search, and brave_search fan out in parallel under a single web_search tool
- **web_scrape**: native URL-to-markdown scraping with SSRF protection, sitemap crawling, JS-shell detection, and profile-aware export
- **DDG captcha detection (v0.3.73)**: detect DuckDuckGo captcha via HTTP 202 + structural form check
- **Built-in /github skill**: full gh CLI control for issues, PRs, code reviews, repo management
- **Proactive tool discovery**: agent finds tools before saying "I can't"
- **Skills system**: workflow templates with fuzzy-finding, auto-registered as slash commands
- **Dynamic tools**: runtime-defined via TOML (HTTP + shell executors)
- **Projects system**: dedicated sessions with per-project brain overlays, file archiving, and color badges
- **Hashline editing**: hash-anchored file editing with batch support and collision detection
- **Mission Control**: full-screen dashboard with RSI inbox, activity log, and cron schedule
- **RTK auto-download**: bundled 4MB proxy for 53.5% token savings on 100+ commands
- **Confidential file protection**: SSH keys, .env, credentials protected by default
- **AGENTS always-loaded**: hard rules and governance enforced every turn
- **Config-driven debug_logs (v0.3.71)**: toggle debug logs with hot-reload, no restart needed
- **Secret redaction scope (v0.3.71)**: scope redaction to global/group/DM with `/redact` command
- **Secret redaction scope (v0.3.71)**: scope redaction to global/group/DM with `/redact` command
- **Config-driven debug_logs (v0.3.71)**: toggle debug logs with hot-reload, no restart needed
- **Tmux watchdog (v0.3.71)**: pane-only kill, 30s warning, auto-reattach
- **Plan-gate three-state (v0.3.71)**: GateDecision with bash going to approval in post-init Editing
- **MCP-style ToolHints (v0.3.71)**: risk model for tool classification, drives plan gate

### Browser Automation
- **Full CDP support**: navigate, click, type, screenshot, JS eval, find elements
- **Headless or headed** mode with element-specific screenshots
- **Cookie/session persistence** across browser sessions
- **Per-session tab isolation**: no cross-session DOM stomping

### Project Directive Discovery (v0.3.59)

OpenCrabs auto-discovers rule files that other AI coding tools drop in a repo. Point the agent at any repository (via `/cd`, a channel workspace, or launching inside one) and it scans for conventions shipped by Claude Code, Cursor, Windsurf, Cline, Gemini CLI, GitHub Copilot, OpenCode, and the cross-tool `AGENTS.md` standard. No config, no import step.

| Source | Files |
|--------|-------|
| Cross-tool standard | `AGENTS.md` |
| Claude Code | `CLAUDE.md`, `CLAUDE.local.md`, `.claude/CLAUDE.md`, `.claude/rules/**/*.md` |
| Cursor | `.cursorrules`, `.cursor/rules/**/*.mdc` |
| Windsurf | `.windsurfrules` |
| Cline | `.clinerules` (file or `.clinerules/**/*.{md,txt}`) |
| Gemini CLI | `GEMINI.md` |
| GitHub Copilot | `.github/copilot-instructions.md` |
| OpenCode | `.opencode/AGENTS.md` |

The index rebuilds when you `/cd` into a new directory, so directive files are always current.

## Quick Start

```bash
# Install (Linux and macOS)
curl -fsSL https://opencrabs.com/install.sh | bash

# Or via Cargo (requires Rust 1.94+)
cargo install opencrabs --locked
```

The installer resolves the latest release, checks the download against the published `SHA256SUMS`, and leaves a callable `opencrabs` command. On macOS that command is a symlink into `~/Applications/OpenCrabs.app`, so do not go looking for a loose binary next to the extracted archive. Every flag and the manual per-platform routes are on [Installation](getting-started/installation.md).

Auto-update enabled by default. Disable with `[agent] auto_update = false` in `~/.opencrabs/config.toml`.

## Architecture

```
┌─────────────────────────────────────────┐
│           OpenCrabs Binary              │
│  (Single 34-36 MB Rust executable)      │
├─────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────┐  │
│  │   TUI       │  │   CLI Daemon    │  │
│  │  (crossterm)│  │  (systemd/launchd)││
│  └─────────────┘  └─────────────────┘  │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │        Provider Registry         │   │
│  │  15 built-in + Custom OpenAI    │   │
│  │  Sticky fallback chain          │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │        Tool Layer                │   │
│  │  50+ built-in tools             │   │
│  │  Dynamic tools via TOML         │   │
│  │  JIT activation                 │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │        Channel Adapters          │   │
│  │  Telegram / Discord / Slack /    │   │
│  │  WhatsApp / Trello / Voice      │   │
│  └────────────────���────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │        Self-Healing Layer       │   │
│  │  Context budget / Stuck stream  │   │
│  │  Phantom detection / RSI        │   │
│  └─────────────────────────────────┘   │
│                                         │
│  ┌─────────────────────────────────┐   │
│  │        Persistence              │   │
│  │  SQLite + Brain files           │   │
│  │  FTS5 + vector search           │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
```

## Next Steps

- [Installation](getting-started/installation.md): Install and configure
- [Configuration](getting-started/configuration.md): All config options
- [Providers](brain/providers-overview.md): Connect your LLM backends
- [Channels](channels/overview.md): Connect Telegram, Discord, etc.
- [Tools](brain/tools.md): Explore 50+ built-in capabilities
- [Self-Healing](features/self-healing.md): Resilience features
- [Multi-Agent](features/multi-agent.md): Orchestrate sub-agents and teams

## Community & Support

- [Telegram community](https://t.me/opencrabs): join the OpenCrabs group for questions, tips, and release chatter
- [Sponsor on GitHub](https://github.com/sponsors/adolfousier): back the project and keep the crabs fed
- [Buy Me a Coffee](https://buymeacoffee.com/opencrabs): one-time support if OpenCrabs saved you an afternoon

Found a bug or want a feature? Open an issue on [GitHub](https://github.com/opencrabs/opencrabs/issues).
