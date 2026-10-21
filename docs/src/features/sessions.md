# Sessions

## Sessions as Isolated Agents

A session in OpenCrabs is not a tab, not a window, not a chat thread. **Each session is a fully independent agent** with its own brain: conversation history, provider, model, working directory, tool state, approval policy, and context window. When you create a new session, you are spinning up a separate agent that knows nothing about any other session and shares nothing with them.

This is the core mental model: **one session = one agent = one context**. You can run dozens of sessions in parallel and they will never interfere with each other.

## Zero Context Contamination

Sessions are completely isolated at every layer:

- **Separate message queues** — each session has its own queue. Messages are routed strictly to their originating session. No cross-session bleeding, even when 10 sessions are processing simultaneously.
- **Separate provider and model** — switching to Gemini in session A does not affect session B running Claude. Each session remembers its own provider independently.
- **Separate working directory** — `/cd` in one session does not change the working directory of any other session.
- **Separate conversation history** — full SQLite-backed history per session. No shared memory, no prompt pollution, no context bleed.
- **Separate token tracking** — cumulative usage, cost, and context window are tracked per session.

This isolation is guaranteed by **Rust's thread safety and async runtime**. Each session runs as an independent `tokio` task with its own state, and the type system prevents accidental sharing at compile time. You can run split panes, channel sessions, background agents, and sub-agents all at once with zero risk of one session's output leaking into another.

## Switching Sessions Keeps the Transcript

Switching sessions with `/sessions` and coming back does not fold your transcript away. The fold state for every turn is keyed by a stable role and timestamp anchor, so a turn you had opened stays open across a reload, and a turn you had folded stays folded (#2155). A turn that is still running also stays unfolded while it runs, so the work in flight stays visible instead of hiding behind a collapsed header.

The display window keeps a bounded number of recent messages in the pane rather than trimming by token budget, so one huge tool result can no longer evict the rest of the conversation. The hidden-count marker still tells you how many older rows are out of view, and Ctrl+O pages back through them.

## Workflow Patterns: One Session Per Context

The power of isolated sessions becomes clear when you treat each one as a dedicated agent for a specific domain:

### Pattern 1: DevOps for a Server

Create a session for a specific server or infrastructure concern. Send a first message like `"Devops for server XYZ — monitor nginx builds, manage cronjobs, handle deployments, run log cleanups"`. The session locks into that context: working directory set to the server's codebase, provider set to a fast model for quick ops tasks, history filled with that server's deployment patterns. Come back to it days later and the agent remembers every previous deploy, every cron change, every nginx config tweak.

### Pattern 2: Mobile App Development with a Co-Founder

Create a session named `mybrand-mobile` and connect it to a Telegram group with your co-founder. The agent is locked into the Dart/Flutter codebase, the product design context, and the mobile-specific toolchain. Your co-founder can ask questions, request design changes, or review PRs directly in Telegram while you work on backend tasks in a separate session. The two contexts never mix.

### Pattern 3: Production Logs with a Team on Slack

Create a session named `mybrand-prod-logs-debug` and connect it to a Slack channel. Your team can ask questions about production, staging, or dev logs without you having to context-switch. The agent stays locked into log analysis mode with the right SSH aliases, the right log paths, and the right debugging tools. Meanwhile, your main TUI session is free for development work.

The key insight: **you never have to explain the full context again**. Once a session is locked into a domain, every follow-up message inherits that context automatically.

## Session Context Integrity (v0.3.78)

Two fixes harden `session_context` (the tool agents use to store key-value state):

- **Atomic writes** — context writes now use atomic file operations, eliminating trailing-character corruption that could occur when a write was interrupted mid-flush. Previously, a crashed write could leave a JSON file with a truncated trailing character, making the entire context unreadable.
- **Concurrency pinning** — concurrent `session_context` calls from parallel tool executions no longer race on the same file. A concurrency pin serializes writes, and temp files are cleaned up after each operation.

## Creating Sessions

- **TUI:** Press `Ctrl+N` or type `/new`
- **Channels:** Type `/new` in any channel

## The First Message Matters

When you create a new session, the first message you send becomes the seed for the entire session's context. OpenCrabs uses it to:

1. **Auto-generate a session title** — a background LLM call extracts a 3-8 word descriptive title from your first message. This runs asynchronously and never enters the conversation context.
2. **Anchor the agent's context** — the initial message establishes what this session is about, what codebase it should focus on, what tools it should prioritize.

Good first messages are specific and contextual:
- `"Devops for server XYZ — nginx, cronjobs, deployments, log cleanups"`
- `"Flutter mobile app for mybrand — Dart codebase at ~/srv/mobile/mybrand"`
- `"Debug production logs for mybrand staging and dev environments"`

The auto-title will generate something like `Devops Server XYZ`, `Mybrand Mobile Flutter`, or `Mybrand Prod Logs Debug`. You can always rename it later.

## Switching Sessions

- **TUI:** Press `Ctrl+L` to open the sessions screen, navigate with arrow keys, press `Enter` to select
- **Channels:** Type `/sessions` to see recent sessions with inline buttons

## Renaming Sessions

Auto-generated titles are a starting point, not a final name. You can rename any session:

- **TUI:** Press `Ctrl+L`, navigate to the session, press `r` to rename
- **Agent-initiated:** the `rename_session` tool lets the agent rename the current session with a descriptive title when the conversation evolves beyond its original scope

Empty or whitespace-only titles are rejected (v0.3.30, #128).

## Session Screen

The sessions screen shows:
- Session name
- Created date
- Provider/model badge
- Working directory
- Token usage
- Context window usage (current session)
- Status indicators (processing spinner, pending approval, unread)

## Sub-Agent Sessions Stay Out of Your Way (v0.3.79)

Every spawned sub-agent creates a session titled `subagent: {label}`, and a busy turn can create several. They are an implementation detail of one tool call, so since v0.3.79:

- Sub-agent sessions are **hidden from session lists** the moment they exist — your session screen only ever shows your sessions.
- They are **automatically purged** once past `agent.subagent_session_ttl_days` (default **7**; set `0` to keep them forever).
- Your own sessions are never touched — the cleanup sweep matches the `subagent:` title prefix only.

## Per-Session State

Each session remembers:
- **Provider and model** — Switch to Claude in one, Gemini in another
- **Working directory** — `/cd` persists per session
- **Conversation history** — Full message history in SQLite
- **Token count and cost** — Cumulative usage tracking

**v0.3.74 hardened per-session isolation:**

- **Per-session working directory** — the working directory is isolated per session so it can't leak across concurrent sessions (#703).
- **Restore session provider** — a session's saved provider is restored before its turn, so it never accidentally runs on the global default (#704).
- **Don't persist involuntary remap** — an involuntary provider/model remap is not persisted over a session's saved pair (#705).

## Session Management

| Action | TUI | Channels |
|--------|-----|----------|
| New | `Ctrl+N` / `/new` | `/new` |
| Switch | `Ctrl+L` + Enter | `/sessions` |
| Rename | `R` on sessions screen | — |
| Delete | `D` on sessions screen | — |

## Background Processing

Sessions can process in the background while you work in another session. The sessions screen shows:
- Spinner for actively processing sessions
- `!` for sessions waiting for tool approval
- Dot for sessions with unread messages

## Background Tasks (v0.3.74)

Long-running shell commands no longer block your session. When the agent runs a known-long command (`cargo test`, `cargo build`, `npx remotion render`, `gh run watch`, and similar), OpenCrabs **auto-promotes it to a background task** (#722): the command runs detached, the session is freed immediately, and the agent can keep working (or you can switch away) while it finishes.

When the task completes, the result is injected back into the originating session and **resumes it automatically**. On the TUI the session picks up at the next tool-call boundary; on Telegram the resume is delivered to the same chat (#722). No polling loops, no re-running the command to "check" — the completion report arrives on its own.

The agent is told via its preamble that long tasks run in the background and resume the session when done, so it wraps up or moves on instead of sitting in a wait loop (#722).

**Two related session fixes landed in the same release:**

- **No perpetual resume** — a completed session no longer resumes on every restart. Resume turns are not re-tracked as pending requests, so a finished background task stays finished across restarts (#729).
- **No lingering cancelled query** — a query cancelled before the agent replied, plus its empty assistant placeholder, are dropped as a pair so they don't linger in context and duplicate on resend (#730).

## Background-Task Resume Everywhere (v0.3.75)

v0.3.74 introduced background-task resume on the TUI and Telegram. v0.3.75 extends it to **every channel**: Discord, Slack, WhatsApp, and the interactive CLI agent now resume your session the moment a long task finishes, powered by a shared `bg_resume` helper so the behaviour is identical across surfaces.

A few refinements make background tasks easier to live with:

- **Indicator on the input border** — the background-task indicator moved onto the input border, and a detached command shows what it's currently doing.
- **No scaffolding in chat** — a background task's system scaffolding no longer reaches the chat.
- **Restart-killed tasks reported** — background tasks a restart killed are reported instead of leaving the session waiting forever.

## Split Panes

Run multiple sessions side by side with tmux-style pane splitting. Each pane is a fully isolated agent — see [Split Panes](./split-panes.md) for details.

## State Management

v0.2.92 improved session state tracking:

- **Session reload after cancellation** — After Esc+Esc cancel, session context reloads from DB to pick up any changes made during the cancelled operation
- **Cached state cleanup** — Deleting a session now clears stale pane cache entries, preventing phantom state on restart
- **CLI tool segment persistence** — Tool results from CLI providers (Claude CLI, OpenCode CLI) are now saved to DB alongside regular messages, preserving correct text/tool interleaving across restarts
- **Case-insensitive tool input** — Tool input descriptions use case-insensitive key lookup, fixing failures when providers return different casing

## Channel Sessions

All channels (Telegram, Discord, Slack, WhatsApp, Trello) persist sessions in SQLite by channel/group title. Sessions survive process restarts — no more lost context after daemon restart. Each channel group gets its own isolated session, while owner DMs share the TUI session. Cross-channel stable session suffixes (`[chat:<id>]`) ensure reliable session resolution across Discord, Slack, and WhatsApp (v0.3.29).

## Session-Routed Recovery Reports (v0.3.81)

Crash-recovery reports **route by session** (#1037): a resumed turn reports back to the session that owned it, not to whichever session happened to boot the recovery. Multi-session setups no longer cross-deliver recovery noise.
