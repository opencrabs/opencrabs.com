# Configuration

OpenCrabs uses two config files stored at `~/.opencrabs/`:

| File | Purpose |
|------|---------|
| `config.toml` | Provider settings, features, channel connections |
| `keys.toml` | API keys and secrets (never committed to git) |

## Workspace Layout

```
~/.opencrabs/
├── config.toml          # Main configuration
├── keys.toml            # API keys (gitignored)
├── commands.toml        # Custom slash commands
├── opencrabs.db         # SQLite database
├── SOUL.md              # Agent personality
├── USER.md              # Your profile
├── MEMORY.md            # Long-term memory
├── AGENTS.md            # Agent behavior docs
├── TOOLS.md             # Tool reference
├── SECURITY.md          # Security policies
├── HEARTBEAT.md         # Periodic check tasks
├── memory/              # Daily memory notes
│   └── YYYY-MM-DD.md
├── images/              # Generated images
├── logs/                # Application logs
└── skills/              # Custom skills/plugins
```

## Provider Configuration

See [Provider Setup](../brain/providers.md) for detailed provider configuration.

**Quick example**: add Anthropic:

```toml
# config.toml
[providers.anthropic]
enabled = true
default_model = "claude-sonnet-4-20250514"
```

```toml
# keys.toml
[providers.anthropic]
api_key = "sk-ant-..."
```

## Provider Priority

When multiple providers are enabled, the first one found in this order is used for new sessions:

MiniMax > OpenRouter > Anthropic > OpenAI > Gemini > Custom

Each session remembers which provider and model it was using. Switch providers per-session via `/models`.

## Config Safety (v0.3.74)

A malformed line in `config.toml` or `keys.toml` corrupts the whole file and takes the agent down (no provider keys, no bot token, no way back in). v0.3.74 makes that class of failure nearly impossible:

- **Config edits via config_manager only**: the agent edits `config.toml` / `keys.toml` only through the `config_manager` tool, never with raw file edits. The write path validates the result and refuses anything that would break parsing (#715).
- **Guard config-breaking writes**: `edit_file` / `write_file` writes that would break `config.toml` / `keys.toml` are denied outright (#713).
- **Re-validate before write**: config is re-validated immediately before `fs::write` in the `config_manager` write paths (#714).
- **Validate keys before snapshot**: `keys.toml` is validated before it is saved as the last-good snapshot, so a bad snapshot can't poison recovery (#712).

**v0.3.79 hardened this further:**

- **Every config write is atomic** (#911): a crash or a concurrent reader mid-write can never observe a half-written `config.toml` or `keys.toml`.
- **Config loads no longer race on shared state**, concurrent reloads can't interleave and leave a torn config in memory.
- **Reload failures are specific**: a failed reload says *what* failed, and a write race is named as the transient condition it is instead of a scary generic error.
- **Never select a provider you didn't configure**, fallback and model routing can no longer land on a provider section that doesn't exist in your config.

## Agent Behavior

```toml
[agent]
working_directory = "/path/to/default/dir"
thinking = "on"                     # "on", "off", or "budget_XXk"
approval_policy = "auto-always"     # "ask", "auto-session", "auto-always"
max_concurrent = 4                  # max parallel tool calls
context_limit = 200000              # context window cap (tokens)
max_tokens = 65536                  # max output tokens per API call
auto_update = true                  # auto-install releases on startup
silent_compaction = false           # suppress post-compaction personality narration
compaction_notice = false            # show the compaction notice pair; off by default (v0.5.4)
lazy_tools = true                   # JIT tool-schema loading (ships core + tool_search only)
redact_sensitive_data = true        # redact API keys, tokens, passwords, IPs from output
debug_logs = false                  # enable debug file logging (hot-reloads, no restart)
default_provider = "minimax"        # fallback provider when no provider is active (v0.3.62)
default_model = "MiniMax-M2.7"      # fallback model when no model is active (v0.3.62)
```

| Field | Default | Description |
|-------|---------|-------------|
| `working_directory` | home dir | Default working directory for the agent |
| `thinking` | `"on"` | Extended thinking mode: `"on"`, `"off"`, or `"budget_XXk"` |
| `approval_policy` | `"auto-always"` | `"ask"` = confirm every tool call, `"auto-session"` = auto-approve for session, `"auto-always"` = never ask |
| `max_concurrent` | `4` | Max tool calls running in parallel |
| `context_limit` | `200000` | Context window limit in tokens. When exceeded, oldest messages are dropped |
| `max_tokens` | `65536` | Max output tokens per single API call |
| `auto_update` | `true` | Automatically install new releases on startup (binary mode only) |
| `silent_compaction` | `false` | When true, suppresses the agent's playful post-compaction narration. Useful for corporate/formal deployments |
| `compaction_notice` | `false` | Show the compaction notice pair in the TUI. Off by default since v0.5.4 (#1686) |
| `lazy_tools` | `true` | Ships only core tool schemas (~4k tokens) plus `tool_search` per request. The agent discovers and activates extended tools on demand via `tool_search`. Set `false` to load all ~95 schemas every request |
| `redact_sensitive_data` | `true` | Redacts API keys, tokens, passwords, and IPs from tool outputs and display. Set `false` during sysadmin/devops work where seeing IPs/tokens/passwords is necessary |
| `debug_logs` | `false` | Enable debug file logging to `~/.opencrabs/logs/`. Hot-reloads on change (no restart). The `--debug` CLI flag always wins when set (OR logic) |
| `default_provider` | `None` (uses active provider) | Fallback provider when no provider is active in the current session. Also used for cron jobs without an explicit provider (v0.3.62) |
| `default_model` | `None` (uses active model) | Fallback model when no model is active in the current session. Also used for cron jobs without an explicit model (v0.3.62) |

Timeout flags resolve per provider first: `[providers.<name>]` wins over `[agent]`, which wins over the built-in default, and every provider family honours the chain, anthropic and gemini included (v0.5.4, #1688).

### Sub-agent and RSI Overrides

Route spawned sub-agents and RSI (self-improvement) cycles to separate providers so they never compete with your main chat for quota:

```toml
[agent]
subagent_provider = "minimax"           # provider for spawned sub-agents
subagent_model = "MiniMax-M2.7"         # model for spawned sub-agents
self_improvement_provider = "minimax"   # provider for RSI self-improvement cycles
self_improvement_model = "MiniMax-M2.7" # model for RSI cycles
```

| Field | Default | Description |
|-------|---------|-------------|
| `subagent_provider` | `None` (uses session provider) | Provider for spawned sub-agents. Keeps sub-agents off your main provider |
| `subagent_model` | `None` (uses session model) | Model for spawned sub-agents |
| `self_improvement_provider` | `None` (uses session provider) | Provider for RSI self-improvement cycles. RSI runs on its own provider chain |
| `self_improvement_model` | `None` (uses session model) | Model for RSI cycles. Prefer cheap, fast models since results are deterministic |

## Channel Configuration

### Telegram

```toml
[channels.telegram]
enabled = true
token = "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
allowed_users = ["123456789"]       # numeric Telegram user IDs
allowed_channels = ["-100123456"]   # restrict to specific group/channel IDs (empty = all)
respond_to = "mention"              # "all", "dm_only", "mention" (default)
session_idle_hours = 24.0           # idle timeout for non-owner sessions
rich_messages = true                # native Telegram rich messages (Bot API 10.1, default since v0.3.64)
silence_group_start = true          # silently ignore /start from non-allowed users in groups
bot_owner = ["123456789"]           # owner IDs (gated commands, /cd hidden dirs, /profiles)
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Telegram bot channel |
| `token` | `None` | Telegram Bot API token from @BotFather |
| `allowed_users` | `[]` (accept all) | Numeric Telegram user IDs. Accepts int or string arrays. Empty = open mode |
| `allowed_channels` | `[]` (all channels) | Restrict bot to specific channel/group IDs. DMs always pass |
| `respond_to` | `"mention"` | When to respond in groups: `"all"` = every message, `"dm_only"` = ignore groups, `"mention"` = only when @mentioned or replied-to |
| `session_idle_hours` | `None` (no timeout) | Idle timeout in hours for non-owner sessions. Owner sessions never expire |
| `rich_messages` | `false` | Send structured replies as native Telegram rich messages (tables, headings, lists, math). Only works on current mobile/desktop Telegram clients. Telegram Web and older clients show a "not supported" placeholder. Enable only when your audience is on modern clients |
| `silence_group_start` | `true` | Silently ignore /start from non-allowed users in group chats. Users who need their ID can DM the bot |
| `bot_owner` | `[]` (first allowed_user) | Bot owner user IDs. Owners can access gated commands (/profiles, hidden files in /cd), manage profiles. When unset, defaults to first entry in `allowed_users` |

### Discord

```toml
[channels.discord]
enabled = true
token = "your-discord-bot-token"
allowed_users = ["123456789012345678"]
allowed_channels = ["123456789012345678"]
respond_to = "mention"
session_idle_hours = 24.0
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Discord bot channel |
| `token` | `None` | Discord bot token from the Developer Portal |
| `allowed_users` | `[]` (accept all) | Discord user IDs. Accepts int or string arrays |
| `allowed_channels` | `[]` (all channels) | Restrict bot to specific channel IDs |
| `respond_to` | `"mention"` | When to respond: `"all"`, `"dm_only"`, `"mention"` |
| `session_idle_hours` | `None` | Idle timeout for non-owner sessions |

### Slack

```toml
[channels.slack]
enabled = true
token = "xoxb-your-bot-token"
app_token = "xapp-your-app-token"      # Socket Mode token
allowed_users = ["U12345678"]
allowed_channels = ["C12345678"]
respond_to = "mention"
session_idle_hours = 24.0
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Slack bot channel |
| `token` | `None` | Bot token (`xoxb-...`) |
| `app_token` | `None` | App-level token for Socket Mode (`xapp-...`) |
| `allowed_users` | `[]` (accept all) | Slack user IDs (`U12345678`) |
| `allowed_channels` | `[]` (all channels) | Restrict bot to specific channel IDs |
| `respond_to` | `"mention"` | When to respond: `"all"`, `"dm_only"`, `"mention"` |
| `session_idle_hours` | `None` | Idle timeout for non-owner sessions |

### WhatsApp

```toml
[channels.whatsapp]
enabled = true
allowed_phones = ["+15551234567"]      # E.164 format
session_idle_hours = 24.0
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the WhatsApp channel |
| `allowed_phones` | `[]` (accept all) | E.164 phone numbers. Empty = accept everyone (not recommended for business numbers) |
| `session_idle_hours` | `None` | Idle timeout for non-owner sessions |

### Trello

```toml
[channels.trello]
enabled = true
token = "your-trello-api-token"
app_token = "your-trello-api-key"
allowed_users = ["memberId1"]
board_ids = ["boardId1", "boardId2"]
poll_interval_secs = 60
session_idle_hours = 24.0
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Trello channel |
| `token` | `None` | Trello API token |
| `app_token` | `None` | Trello API key (stored as `app_token` for keys.toml symmetry) |
| `allowed_users` | `[]` (accept all) | Trello member IDs |
| `board_ids` | `[]` (all boards) | Board IDs to monitor for @mentions. Also accepts `allowed_channels` as an alias |
| `poll_interval_secs` | `None` (tool-only) | Polling interval in seconds. Absent or 0 = no polling (tool-only mode) |
| `session_idle_hours` | `None` | Idle timeout for non-owner sessions |

### Other Channels (Preview)

Signal, Google Chat, and iMessage are available as preview placeholders:

```toml
[channels.signal]
enabled = true
allowed_phones = ["+15551234567"]

[channels.google_chat]
enabled = true
token = "your-google-chat-token"
allowed_users = ["user@example.com"]

[channels.imessage]
enabled = true
allowed_phones = ["+15551234567"]
```

## Cron Defaults

Route cron jobs to cheaper providers so they never compete with your interactive session:

```toml
[cron]
default_provider = "minimax"
default_model = "MiniMax-M2.7"
```

| Field | Default | Description |
|-------|---------|-------------|
| `default_provider` | `None` (uses active provider) | Default provider for cron jobs without an explicit provider |
| `default_model` | `None` (uses active model) | Default model for cron jobs without an explicit model |

## Memory and Embeddings

```toml
[memory]
vector_enabled = true

[memory.embedding]
url = "https://api.openai.com/v1"
model = "text-embedding-3-small"
# api_key loaded from keys.toml: [providers.memory_embedding] api_key = "sk-..."
# dimensions = 1536  # auto-detected from first API response if unset
```

| Field | Default | Description |
|-------|---------|-------------|
| `vector_enabled` | `true` (desktop), `false` (VPS) | Enable vector embeddings for semantic memory search. When disabled, only FTS5 keyword search is used. Auto-disabled on systems with < 2GB RAM or detected cloud instances |
| `embedding.url` | `None` | OpenAI-compatible API base URL. The `/embeddings` path is appended automatically |
| `embedding.model` | `None` | Embedding model name (e.g. `text-embedding-3-small`, `nomic-embed-text`) |
| `embedding.api_key` | `None` | API key for the embedding endpoint. Also loaded from `keys.toml` under `[providers.memory_embedding]` |
| `embedding.dimensions` | `None` (auto-detected) | Embedding vector dimensions. Auto-detected from the first API response if unset. Local GGUF model always produces 768-dim vectors |
| `extra_paths` | `[]` | External paths indexed into the `external` collection. Bare path strings or `{ path, pattern }` tables; relative paths resolve against the OpenCrabs home |
| `exclude` | VCS/build/secret globs | Glob excludes for external indexing, global across all entries |
| `external_allowed_in_shared` | `false` | Allow `scope="external"` results in shared/group sessions. Default-deny: external content stays owner-session-only unless opted in |
| `sweep_interval_secs` | `300` | Seconds between external-path freshness sweeps; modified files are also caught lazily at search time |

When `[memory.embedding]` is not set, embeddings are generated locally via the embeddinggemma-300M GGUF model (~300MB download, ~2.9GB RAM). Setting `[memory.embedding]` with an API endpoint eliminates the local model overhead.

To index your own docs (or an entire scraped docs site) into memory search, see [Memory System → External Index Paths](../brain/memory.md#external-index-paths-v0381) for the full knowledge-base workflow.

## Brain Files

```toml
[brain]
strip_empty_sections = true
default_cap = 500

[brain.caps]
SOUL.md = 300
AGENTS.md = 800
```

| Field | Default | Description |
|-------|---------|-------------|
| `strip_empty_sections` | `true` | Strip empty header stubs (`## Header` with no body) from brain-file reads. Writes are never affected, only the loaded view is filtered |
| `default_cap` | `500` | Per-file line cap for `sync_templates`. When a merged file exceeds its cap, the sync bails instead of writing |
| `caps` | `{}` (empty) | Per-file line caps overrides. Keys are filenames (case-sensitive: `TOOLS.md` and `tools.md` are distinct) |

## Browser

```toml
[browser]
cdp_endpoint = "http://localhost:9222"
```

| Field | Default | Description |
|-------|---------|-------------|
| `cdp_endpoint` | `None` (spawn new browser) | CDP endpoint for an existing Chromium instance. When set, connects via Chrome DevTools Protocol instead of spawning a new browser. Useful for sharing a single browser across multiple profiles to save memory (~250-300MB per instance) |

To start a standalone Chromium with CDP enabled:
```bash
chromium --remote-debugging-port=9222 --headless --no-sandbox
```

## A2A (Agent-to-Agent) Gateway

```toml
[a2a]
enabled = false
bind = "127.0.0.1"
port = 18790
allowed_origins = ["https://your-app.com"]
# api_key = "your-secret-key"  # Bearer token for incoming requests
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the A2A JSON-RPC 2.0 gateway |
| `bind` | `"127.0.0.1"` | Bind address |
| `port` | `18790` | Gateway port |
| `allowed_origins` | `[]` | CORS origins. Must be set explicitly, no cross-origin requests allowed by default |
| `api_key` | `None` | Bearer token for authenticating incoming A2A requests. If unset, no authentication required |

## Daemon Mode

```toml
[daemon]
health_port = 8080
```

| Field | Default | Description |
|-------|---------|-------------|
| `health_port` | `None` (no health server) | HTTP port for `GET /health` endpoint. Useful for systemd watchdog, uptime monitors, and external health probes |

OpenCrabs runs in two modes: **TUI** (interactive terminal UI with chat) and **Daemon** (headless background service for channels + cron). For any one profile, run only one at a time. The TUI always wins: opening it while a daemon runs shuts the daemon down and takes over the channels.

For full service lifecycle management (TUI vs Daemon comparison, `opencrabs service install/start/stop`, profile-aware services, `OPENCRABS_PROFILE` env var, troubleshooting), see the [CLI Commands](../getting-started/cli-commands#daemon-mode-vs-tui) reference.

## Image Generation and Vision

```toml
[image.generation]
enabled = true
model = "gemini-3.1-flash-image-preview"

[image.vision]
enabled = true
model = "gemini-3.1-flash-image-preview"
provider = "openrouter"           # bypasses enabled gate for vision-only providers (v0.3.63)
```

| Section | Field | Default | Description |
|---------|-------|---------|-------------|
| `image.generation` | `enabled` | `false` | Enable image generation via the `generate_image` tool |
| `image.generation` | `model` | `"gemini-3.1-flash-image-preview"` | Model for image generation |
| `image.vision` | `enabled` | `false` | Enable vision analysis via the `analyze_image` tool. **Since v0.3.64:** setting `vision_model` alone is sufficient to enable vision. `enabled` is no longer required when `vision_model` is set |
| `image.vision` | `model` | `"gemini-3.1-flash-image-preview"` | Model for image/vision analysis |
| `image.vision` | `provider` | `None` (auto-detect) | Dedicated provider for vision. Bypasses the enabled gate so you can use a vision-only provider without enabling it for general chat (v0.3.63) |

Vision analysis automatically scans all enabled providers (Google, OpenRouter, OpenAI-compatible, Anthropic) before returning an error. No configuration needed.

## Voice Provider Fallback

STT and TTS providers support automatic failover via `fallback_chain`. When the primary returns a 5xx, fails a liveness probe (Voicebox), or is otherwise unreachable, the dispatcher walks the chain in order and tries each entry that has the credentials/config it needs.

```toml
[providers.stt]
fallback_chain = ["groq", "openai_compatible", "local"]

[providers.tts]
fallback_chain = ["openai_compatible", "openai", "local"]
```

| Chain | Valid labels |
|-------|-------------|
| **STT** | `voicebox`, `openai_compatible`, `groq`, `local` (aliases: `whisper`, `local_whisper`) |
| **TTS** | `voicebox`, `openai_compatible`, `openai`, `local` (aliases: `piper`, `local_piper`). `groq` is STT-only, the TTS chain rejects it |

Empty or omitted chain means "use the default priority order with the primary removed."

## TOML Bash Blocklist (v0.3.78)

A new **TOML-driven bash blocklist** provides runtime-configurable safety gates for shell commands. The blocklist lives in a TOML file and defines patterns that the agent's `bash` tool will refuse to execute:

```toml
# safety.toml (example)
[bash_blocklist]
patterns = ["rm -rf /", "dd if=", "mkfs", ":(){ :|:& };:"]
```

- Patterns are matched against the command string before execution
- The blocklist **reloads on file change** (no restart needed)
- Configurable per-profile, so different environments can have different safety gates
- Blocked commands return a clear error explaining which pattern matched

This replaces hardcoded safety checks with a user-extensible system. You can add project-specific dangerous commands without code changes.

## Safety TOML Reload (v0.3.78)

All safety-related TOML files (bash blocklist, brain verification rules) now **hot-reload on change**. Edit the file and the new rules take effect immediately on the next tool call. No restart, no `/reload`, no config manager dance.
