# Architecture

## High-Level Overview

```
┌─────────────────────────────────────────────────┐
│          TUI (ratatui) + Split Panes             │
├────────┬────────┬──────────┬────────────────────┤
│Telegram│Discord │  Slack   │     WhatsApp       │
├────────┴────────┴──────────┴────────────────────┤
│                 Brain (Agent Core)               │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │ Providers│ │  Tools   │ │  Memory (3-tier) │ │
│  │ Registry │ │ +Dynamic │ │                  │ │
│  └──────────┘ └──────────┘ └──────────────────┘ │
├─────────────────────────────────────────────────┤
│   Services / DB (SQLite) │ Browser (CDP)         │
├─────────────────────────────────────────────────┤
│   A2A Gateway │ Cron Scheduler │ Sub-Agents      │
├─────────────────────────────────────────────────┤
│   Shared Channel Commands (commands.rs, 847 lines) │
├─────────────────────────────────────────────────┤
│   Self-Healing (config recovery, provider health, │
│   ARG_MAX compaction, error surfacing)             │
├─────────────────────────────────────────────────┤
│   Daemon Mode (health endpoint, auto-reconnect)  │
└─────────────────────────────────────────────────┘
```

## Source Layout

```
src/
├── main.rs              # Entry point, CLI parsing
├── lib.rs               # Library root
├── cli/                 # CLI argument parsing (clap)
├── config/              # Configuration types, loading, health tracking
│   └── health.rs        # Provider health persistence (120 lines)
├── db/                  # SQLite database layer
│   ├── models.rs        # Data models (Session, Message, etc.)
│   └── repository/      # Query functions per entity
├── migrations/          # SQL migration files
├── services/            # Business logic layer
│   └── session.rs       # Session management service
├── brain/               # Agent core
│   ├── agent/           # Agent service, context, tool loop
│   │   └── service/     # Builder, context, helpers, tool_loop
│   ├── provider/        # LLM provider implementations
│   ├── tools/           # 50+ tool implementations
│   └── memory/          # 3-tier memory system
├── tui/                 # Terminal UI (ratatui + crossterm)
│   ├── app/             # App state, input, messaging
│   └── render/          # UI rendering modules
├── channels/            # Messaging platform integrations
│   ├── commands.rs      # Shared text command handler (847 lines)
│   ├── telegram/        # Teloxide-based bot
│   ├── discord/         # Serenity-based bot
│   ├── slack/           # Slack Socket Mode
│   └── whatsapp/        # WhatsApp Web pairing
├── a2a/                 # Agent-to-Agent gateway (axum)
├── cron/                # Cron job scheduler
├── memory/              # Vector search + FTS5
├── docs/                # Embedded doc templates
├── tests/               # Integration tests
└── benches/             # Criterion benchmarks
```

## Key Crates

| Crate | Purpose |
|-------|---------|
| `ratatui` + `crossterm` | Terminal UI rendering and input |
| `rusqlite` + `deadpool-sqlite` | SQLite database with connection pooling |
| `reqwest` | HTTP client for LLM APIs |
| `axum` + `tower-http` | A2A HTTP gateway |
| `crabrace` | Provider registry and routing |
| `teloxide` | Telegram Bot API |
| `serenity` | Discord gateway |
| `slack-morphism` | Slack API |
| In-tree memory store (`src/memory/`) | Memory search (SQLite FTS5 + vector embeddings) |
| `rwhisper` (candle) | Local STT: pure Rust, Metal GPU on macOS |
| `piper` (Python venv) | Local TTS with OGG/Opus encoding |
| `syntect` | Syntax highlighting in TUI |
| `tiktoken-rs` | Token counting |

## Data Flow

1. **Input** arrives from TUI, channel, A2A, or cron trigger
2. **Channel commands** (`/doctor`, `/help`, `/usage`, `/evolve`) execute directly via the shared handler without LLM routing
3. **Brain** builds context (system prompt + brain files + memory + conversation)
4. **Provider** streams the LLM response via the selected provider; health is tracked per-provider
5. **Tool Loop** executes any tool calls, feeds results back to the LLM. CLI provider segments (text/tool interleaving) are tracked for correct ordering
6. **Response** is delivered back to the originating channel
7. **DB** persists messages, token usage, session state, and CLI tool segments
8. **Self-healing** monitors for config corruption, context budget overflow (65% threshold), ARG_MAX limits, stuck streams (2048-byte repeat detection), idle timeouts (60s), provider failures (per-provider health tracking with auto-failover), and DB integrity. Crash recovery replays pending requests on restart. All errors surfaced -- nothing swallowed silently

## Database

SQLite with WAL mode. Tables:

- `sessions`: Session metadata, provider, model, working directory
- `messages`: Conversation history per session
- `usage_ledger`: Permanent token/cost tracking
- `memory_*`: FTS5 and vector tables for semantic memory

Migrations run automatically on startup from `src/migrations/`.

## Concurrency Model

- **Tokio** async runtime with multi-threaded scheduler
- Each channel runs as an independent tokio task
- Sessions are isolated: each has its own conversation state
- Tool execution uses `tokio::task::block_in_place` for sync operations
- A2A gateway runs as a separate axum server task

## Logging and Forensics (v0.3.82)

One daily rolling log file per profile, written synchronously on the calling thread.

### Session correlation (#1078)

Every agent turn opens a `tracing` span carrying its `session_id`, so each log line emitted inside that turn is stamped with it. Cron jobs open a `job` span (name + id) and the RSI engine opens its own. Spawned tasks inherit the span, so work that moves onto another task still reports the turn it belongs to.

This makes a single turn greppable out of a file where every session, cron job and RSI cycle is interleaved:

```bash
grep '<session-id>' ~/.opencrabs/logs/opencrabs.YYYY-MM-DD
```

Before this, filtering was guesswork: thread ids get reused and outlive a turn, and timestamps only narrow the window.

### Writer reliability (#1115, #1077)

The file writer is deliberately synchronous rather than using a background worker, because the worker swallows IO errors and drops events under load. Two failure modes were fixed in v0.3.82:

- **Events were dropped, and log output could reach the TUI's terminal** (#1115), corrupting the display.
- **A stalled write silenced everything** (#1077). The writer held a mutex for the duration of a write, so if one write blocked (a hung filesystem, disk pressure), every other thread trying to log blocked behind it. One parked write could take the whole process's logging down while the agent kept running. The writer now uses `try_lock`: under contention the event is dropped rather than blocking the caller, so one stalled write can never silence another thread.
