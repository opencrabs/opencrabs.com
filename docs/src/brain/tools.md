# Built-in Tools

OpenCrabs ships with 50+ tools available to the agent out of the box, plus support for user-defined dynamic tools.

## File Operations

| Tool | Parameters | Description |
|------|-----------|-------------|
| `ls` | `path` | List directory contents |
| `glob` | `pattern`, `path` | Find files by glob pattern |
| `grep` | `pattern`, `path`, `include` | Search file contents with regex |
| `read_file` | `path`, `line_start`, `line_end` | Read file contents |
| `edit_file` | `path`, `old_string`, `new_string` | Edit files with search/replace |
| `write_file` | `path`, `content` | Write new files |

## Code Execution

| Tool | Parameters | Description |
|------|-----------|-------------|
| `bash` | `command`, `timeout` | Execute shell commands |
| `execute_code` | `language`, `code` | Run code in sandboxed environment |

## Web & Network

| Tool | Parameters | Description |
|------|-----------|-------------|
| `web_search` | `query` | Search the web (multi-engine fan-out; results deduped by URL across engines, v0.5.4) |
| `serper_search` | `query` | Google SERP results via Serper; set the key in `keys.toml` (serper.dev) (v0.5.4) |
| `web_scrape` | `url` | Native URL-to-markdown scraping with SSRF protection, sitemap crawling, JS-shell detection (v0.3.60) |
| `http_request` | `method`, `url`, `headers`, `body` | Make HTTP requests |

## Document Parsing

| Tool | Parameters | Description |
|------|-----------|-------------|
| `parse_document` | `path` | Extract text from PDF, DOCX, XLSX, XLS, XLSM, XLSB, ODS, DOC, CSV, HTML, TXT, MD, JSON, XML (v0.3.61 added spreadsheet support; v0.3.80 added xlsm/xlsb/ods via calamine, legacy .doc via rwml, and VBA macro source extraction from macro-enabled workbooks) |

## Document Generation

| Tool | Parameters | Description |
|------|-----------|-------------|
| `generate_document` | `format`, `content`, `style` | Generate PDF, DOCX, XLSX, PPTX natively. Brand colors, logos, image blocks, live Excel formulas. Files delivered as channel attachments (v0.3.62) |

## Session & Memory

| Tool | Parameters | Description |
|------|-----------|-------------|
| `session_search` | `query`, `limit` | Semantic search across sessions |
| `session_context` | `action` | Read/write session context |
| `task_manager` | `action`, various | Manage plans and tasks |
| `decide_cached` | decision input | L1 decision cache (#1648): exact-repeat decisions are served from a reuse ring keyed on canonicalized input, with a shadow tier that logs what it would answer before it goes live (v0.5.4) |

## Image & Video

| Tool | Parameters | Description |
|------|-----------|-------------|
| `generate_image` | `prompt`, `filename` | Generate images via Gemini |
| `analyze_image` | `image`, `question` | Analyze images via Gemini vision |
| `analyze_video` | `video`, `question` | Analyze videos via Gemini multimodal vision. Supports mp4/m4v/mov/webm/mkv/avi/3gp/flv. Inline bytes for ≤18 MB, resumable Files API upload for larger files (v0.3.17) |

## Channel Integrations

| Tool | Parameters | Description |
|------|-----------|-------------|
| `telegram_send` | `action`, various | Telegram operations (19 actions) |
| `discord_connect` | `action`, various | Discord operations (17 actions) |
| `slack_send` | `action`, various | Slack operations (17 actions) |
| `trello_connect` | `action`, various | Trello operations (22 actions) |

## Sub-Agent Orchestration

Agents can spawn independent child agents for parallel task execution:

| Tool | Parameters | Description |
|------|-----------|-------------|
| `spawn_agent` | `label`, `agent_type`, `prompt` | Spawn a typed child agent in an isolated session |
| `wait_agent` | `agent_id`, `timeout_secs` | Wait for a child agent to complete and return output |
| `send_input` | `agent_id`, `text` | Send follow-up input to a running agent (multi-turn) |
| `close_agent` | `agent_id` | Terminate a running agent and clean up resources |
| `resume_agent` | `agent_id`, `prompt` | Resume a completed/failed agent with new prompt (preserves context) |
| `team_create` | `team_name`, `agents[]` | Spawn N typed agents as a named team (parallel) |
| `team_broadcast` | `team_name`, `message` | Fan-out message to all running agents in a team |
| `team_delete` | `team_name` | Cancel and clean up all agents in a team |

### Agent Types

When spawning, `agent_type` selects a specialized role with a curated tool registry:

| Type | Role | Tool Access |
|------|------|-------------|
| `general` | Full-capability (default) | All parent tools minus recursive/dangerous |
| `explore` | Fast read-only codebase navigation | `read_file`, `glob`, `grep`, `ls` |
| `plan` | Architecture planning | `read_file`, `glob`, `grep`, `ls`, `bash` |
| `code` | Implementation with full write access | All parent tools minus recursive/dangerous |
| `research` | Web search + documentation lookup | `read_file`, `glob`, `grep`, `ls`, `web_search`, `http_request` |

**ALWAYS_EXCLUDED tools** (no agent type has these): `spawn_agent`, `resume_agent`, `wait_agent`, `send_input`, `close_agent`, `rebuild`, `evolve` -- no recursive spawning, no self-modification from subagents.

## Browser Automation

Native headless Chrome control via Chrome DevTools Protocol (CDP):

| Tool | Parameters | Description |
|------|-----------|-------------|
| `navigate` | `url` | Open a URL in the browser |
| `click` | `selector` | Click an element by CSS selector |
| `type` | `selector`, `text` | Type text into an input field |
| `screenshot` | `selector` | Capture a screenshot |
| `eval_js` | `code` | Execute JavaScript in the page context |
| `extract_content` | `selector` | Extract text content from elements |
| `wait_for_element` | `selector`, `timeout` | Wait for an element to appear |
| `find` | `pattern`, `mode` | Find elements by CSS, XPath, text, or aria-label. Returns stable selectors |
| `browser_close` | - | Close browser tab and free CDP session. Prevents stale page reuse across browser actions (v0.3.18) |

Auto-detects your default Chromium browser. Feature-gated under `browser` (enabled by default).

## Dynamic Tools

Define custom tools at runtime via `~/.opencrabs/tools.toml`. See [Dynamic Tools](../features/dynamic-tools.md) for details.

| Tool | Parameters | Description |
|------|-----------|-------------|
| `tool_manage` | `action`, various | Create, remove, or reload dynamic tools |

## System

| Tool | Parameters | Description |
|------|-----------|-------------|
| `slash_command` | `command`, `args` | Execute slash commands (/cd, /compact, etc.) |
| `config_manager` | `action`, various | Read/write config, manage commands |
| `evolve` | `check_only` | Download latest release |
| `rebuild` | - | Build from source and restart |
| `plan` | `action`, various | Create and manage execution plans |

## Error Handling

v0.2.92 improved error surfacing across all tool connections. Channel connect tools (`slack_connect`, `whatsapp_connect`, `trello_connect`) now surface actual connection errors instead of silently swallowing them. Tool call status correctly transitions from "running" to success/failure instead of showing a perpetual spinner.

## System CLI Tools

OpenCrabs runs in a TUI with full terminal access. The agent can execute **any CLI tool** installed on the host via the `bash` tool -- no plugins, no wrappers. If it's on your system, the agent can use it. Common ones:

| Tool | Purpose | Check |
|------|---------|-------|
| `gh` | GitHub CLI: issues, PRs, repos, releases, actions | `gh --version` |
| `gog` | Google CLI: Gmail, Calendar (OAuth) | `gog --version` |
| `docker` | Container management | `docker --version` |
| `ssh` | Remote server access | `ssh -V` |
| `node` | Run JavaScript/TypeScript tools | `node --version` |
| `python3` | Run Python scripts and tools | `python3 --version` |
| `ffmpeg` | Audio/video processing | `ffmpeg -version` |
| `curl` | HTTP requests (fallback when `http_request` insufficient) | `curl --version` |

### GitHub CLI (gh)

Authenticated GitHub CLI for full repo management:

```bash
gh issue list / view / create / close / comment
gh pr list / view / create / merge / checks
gh release list / create
gh run list / view / watch
```

### Google CLI (gog)

OAuth-authenticated Google Workspace CLI. Supports Gmail and Calendar:

```bash
gog calendar events --max 10
gog gmail search "is:unread" --max 20
gog gmail send --to user@email.com --subject "Subject" --body "Body"
```

Requires `GOG_KEYRING_PASSWORD` and `GOG_ACCOUNT` env vars.

## Companion Tools

### SocialCrabs: Social Media Automation

[SocialCrabs](https://github.com/adolfousier/socialcrabs) is a social media automation tool with human-like behavior simulation (Playwright). Supports **Twitter/X**, **Instagram**, and **LinkedIn**.

The agent calls SocialCrabs CLI commands via `bash`:

```bash
node dist/cli.js x tweet "Hello world"
node dist/cli.js x mentions -n 5
node dist/cli.js ig like <post-url>
node dist/cli.js linkedin connect <profile-url>
```

Read operations are safe. Write operations (tweet, like, follow, comment) require explicit user approval.

### WhisperCrabs: Floating Voice-to-Text

[WhisperCrabs](https://github.com/adolfousier/whispercrabs) is a floating voice-to-text widget controllable via D-Bus. Click to record, click to stop, text goes to clipboard. The agent can start/stop recording, switch providers, and view transcription history via D-Bus commands.
