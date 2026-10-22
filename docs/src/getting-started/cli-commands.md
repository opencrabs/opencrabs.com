# CLI Commands

OpenCrabs has a full CLI with 20+ subcommands for managing every aspect of the agent.

## Usage

```bash
opencrabs [COMMAND] [OPTIONS]
```

## Commands

| Command | Description |
|---------|-------------|
| `chat` (default) | Launch the TUI chat interface |
| `daemon` | Run in background (channels only, no TUI) |
| `agent` | Interactive multi-turn chat or single-message mode |
| `run` | Execute a single prompt non-interactively; `--quiet` suppresses UI chrome for machine-pure stdout, `--format` picks text/json/markdown (v0.5.4) |
| `cron` | Manage scheduled tasks (add/list/remove/enable/disable/test) |
| `channel` | Channel management (list, doctor) |
| `memory` | Memory management (list, get, stats, prune) |
| `session` | Session management (list, get) |
| `db` | Database management (init, stats, clear) |
| `logs` | Log management (status, view, clean, open) |
| `service` | System service management (install/start/stop/restart/status/uninstall) |
| `status` | Show agent status |
| `doctor` | Run connection health check; `--fix` repairs stuck cron rows, stale markers and permissions (v0.3.83) |
| `onboard` | Run the setup wizard |
| `completions` | Generate shell completions (bash/zsh/fish/powershell) |
| `version` | Show version info |
| `!command` | **Bang operator**: Run any shell command instantly without an LLM round-trip. Output shown as system message. e.g. `!git status`, `!ls -la` |
| `/evolve` | **Auto-update**: Downloads latest release and hot-restarts. Runs automatically on startup when `[agent] auto_update = true` |
| `/restart` | **Restart** (v0.3.79): Relaunches the same binary with the same arguments, so nothing about how it was started has to be known or repeated. Unfinished turns resume automatically on startup. In the TUI it resumes the current session. Owner-only on channels; announces before acting. |
| `/exit` | **Shutdown** (v0.3.79): Shuts OpenCrabs down. Starting it again needs access to the machine it runs on, so this is **the one command with no way back from chat**. Owner-only on channels; announces before acting. |
| `/clear` | **Fresh session** (v0.5.2): Closes the current session and starts a blank one in place, without the summariser call, so it is instant. Owner-only on channels. |
| `/btw` | **Parallel agent**: Spawns an isolated sub-agent for a side task while the main conversation continues. e.g. `/btw research the latest Rust async patterns` |
| `/mission-control` | **Mission Control**: Full-screen dashboard showing RSI inbox (pending proposals), activity log (improvements applied), and cron schedule. Navigate with vim keys, apply/reject proposals with `a`/`r`. |
| `/skills` | **Skills picker**: Browse and launch workflow templates with fuzzy-finding. Every loaded skill auto-registers as a slash command. |
| `/security-audit` | **Security audit**: Comprehensive language-agnostic security & CVE audit. Detects project type, runs the right scanner, reviews recent diff for injection/auth/crypto patterns, scores 0-100. |
| `/cost-estimate` | **Cost estimate**: Codebase cost-to-build estimate, AI-assisted ROI breakdown, and fair-market valuation. Asks for business context before producing the valuation range. |
| `/repo-audit` | **Repo audit**: Language-agnostic repository health checks. 5-phase pipeline: language detection → native tool execution → git metrics → AST analysis → scoring + recommendations. Covers Rust, JS/TS, Python, Go. |
| `/goal` | **Autonomous goal loop**: Set a goal with `/goal <text>` and the agent loops autonomously: executing, self-evaluating with an LLM judge, and continuing until the goal is satisfied or the turn budget (default 20) runs out. Supports `/goal pause`, `/goal resume`, `/goal status`, `/goal clear`. |
| `/models <provider/model>` | **Direct model switch** (v0.3.66): Switch directly to a specific model on every channel. Use `/models anthropic/claude-sonnet-4-20250514` to switch immediately. The apply-to-scope selector lets you choose session-only or global scope. |
| `opencrabs session set-model` | **Headless model switch** (v0.3.66): Switch models from the CLI without the TUI. Useful for scripts and automation. |

## Configuration Flags

| Flag | Default | Description |
|------|---------|-------------|
| `[agent] auto_update` | `true` | Auto-install new releases on startup and hot-restart. Set to `false` to keep the manual prompt dialog. |
| `[agent] force_default` | `false` | (v0.3.66) When true, pushes the default provider/model pair to all sessions on config reload. Useful for enforcing a global model switch. |
| `[agent] config_drift_warnings` | `true` | (v0.3.66) Show startup warnings when a config value silently drifts from the expected default. Helps catch unintended config changes. |

## Keyboard Shortcuts (TUI)

| Shortcut | Action |
|----------|--------|
| `F12` | Toggle mouse capture on/off for native terminal text selection |

## Startup Update Prompt

When a new version is available, a centered dialog appears on the splash screen asking you to accept (Enter) or skip (Esc). Accepting triggers `/evolve` automatically. After update, the binary restarts and the splash shows the new version.

## Channel Commands

`/doctor`, `/help`, `/usage`, `/evolve`, `/restart`, `/exit`, and system commands work directly on Telegram, Discord, Slack, and WhatsApp without going through the LLM. They execute instantly and return results in the channel.

`/restart` and `/exit` announce themselves before acting, because once the process is gone nothing is left to report a failure. Both are owner-only, like every other command that changes the running process. `/exit` is the one command with no way back from chat: recovering needs access to the machine OpenCrabs runs on.

Since v0.3.80, a **spoken stop is honoured too**, telling the bot to stop in natural language works on channels and in the TUI, not just the bare word "stop".

All channel command logic is centralized in `src/channels/commands.rs` (847 lines) -- a shared handler that eliminates duplicated command logic across 5 channel implementations. Each channel delegates to `try_execute_text_command()` for consistent behavior.

`/evolve` on channels now runs directly (downloads + installs the binary) without requiring an LLM round-trip. Previously it was routed through the agent.

## Chat Mode

```bash
# Default: launch TUI
opencrabs

# Same as above
opencrabs chat
```

## Agent Mode

Non-interactive mode for scripting and automation:

```bash
# Interactive multi-turn chat
opencrabs agent

# Single-message mode
opencrabs agent -m "What files changed today?"
```

## Daemon Mode vs TUI

OpenCrabs runs in one of two modes. Pick the one that fits the machine. **For any one profile, run only one at a time.**

| Mode | How you start it | What it is | Use it on |
|------|------------------|------------|-----------|
| **TUI (interactive)** | `opencrabs` | The full terminal UI: chat panes, sessions, settings. Your channels run too and share your session. | A machine you sit at (laptop/desktop) |
| **Daemon (headless)** | `opencrabs daemon`, or install as a service: `opencrabs service install && opencrabs service start` | No UI. Channels only (Telegram, Discord, Slack, WhatsApp) + cron. Survives reboots, SSH disconnects, and crashes. | An always-on box or VPS |

### Why not both at once?

A bot credential (e.g. a Telegram token) can only hold one live `getUpdates` poll. If a daemon and a TUI both own the same profile's token they fight (HTTP 409) and the channel drops.

### The TUI always wins

When you open the TUI while a daemon for the *same profile* is running, the TUI shuts that daemon down first, takes over the channels, and shows a banner saying so. Your channels were already set up, so they just resume, no reconnecting. The daemon stays down until you start it again (`opencrabs service start`, or relaunch `opencrabs daemon`).

On a box where the daemon usually runs, the everyday flow is: open `opencrabs` when you want to sit down with it; close the TUI and `opencrabs service start` when you want it headless again.

### Auto-start on boot

- **Daemon**: use the service installer (`opencrabs service install`); it wires up systemd (Linux) / launchd (macOS) to start on boot and restart on crash. This is the recommended always-on setup.
- **TUI**: to have the terminal UI open automatically on login, use a terminal/desktop autostart, not the service installer:
  - **Linux desktop:** drop a `.desktop` file in `~/.config/autostart/` with `Exec=x-terminal-emulator -e opencrabs`.
  - **macOS:** System Settings > General > Login Items > add a `.command` script that runs `opencrabs`.
  - **VPS over SSH:** run inside `tmux`/`screen` and reattach. On a headless VPS you usually want the daemon, not the TUI.

> **Strongly recommended for everyday users.** If you plan to use OpenCrabs daily, ask it to set itself up as a system service. Just say something like *"set yourself up to start with my computer"* or *"remove the auto-start service"*. The agent handles the launchd (macOS) or systemd (Linux) setup and removal for you automatically.

### Keep the TUI running on a VPS (tmux)

If you want the full interactive TUI always available on a remote server (not the headless daemon), `tmux` is the way. Your session survives SSH disconnects, laptop sleep, and flaky connections.

**1. Install tmux**

```bash
# Debian / Ubuntu
sudo apt install tmux

# RHEL / CentOS / Fedora
sudo yum install tmux
```

**2. Find where opencrabs is installed**

```bash
which opencrabs

# If that returns nothing, search the whole filesystem
find / -name "opencrabs" -type f 2>/dev/null
```

**3. Start a named tmux session**

```bash
tmux new -s opencrabs
```

**4. Launch opencrabs inside it**

```bash
opencrabs
```

**5. Detach (leaves opencrabs running)**

Press `Ctrl+B`, then `D`. You can now close SSH and opencrabs keeps running.

**6. Reattach later**

```bash
tmux attach -t opencrabs
```

That's it. Come back from any machine, reattach, and pick up where you left off. If you only need channels (Telegram, Discord, Slack, WhatsApp) running headless without the TUI, the daemon is a better fit: `opencrabs service install && opencrabs service start`.

### Health Endpoint

Add to `config.toml` to expose a health check:

```toml
[daemon]
health_port = 8080
```

Then `GET http://localhost:8080/health` returns 200 OK with JSON status. Useful for systemd watchdog, uptime monitors, or load balancers.

## Service Management

Install OpenCrabs as a system service (launchd on macOS, systemd on Linux):

```bash
opencrabs service install
opencrabs service start
opencrabs service stop
opencrabs service restart
opencrabs service status
opencrabs service uninstall
```

### Profile-aware services

Each profile gets its own independent service:

```bash
# Install a specific profile as a service
opencrabs -p hermes service install
opencrabs -p hermes service start

# Each profile gets its own service name
# macOS: com.opencrabs.daemon.hermes
# Linux: opencrabs-hermes.service

# Manage independently
opencrabs -p hermes service status
opencrabs -p hermes service stop
opencrabs -p hermes service uninstall
```

Multiple profiles can run as simultaneous daemon services with full isolation.

### OPENCRABS_PROFILE environment variable

Set `OPENCRABS_PROFILE=hermes` to select a profile without the `-p` flag. Useful for systemd services, cron jobs, and daemon mode.

### Troubleshooting: daemon stays down

If `opencrabs service status` says stopped:

1. **Did you open the TUI?** Opening `opencrabs` deliberately shuts the daemon down so the interactive session can own the channels. The daemon stays down until you `opencrabs service start` again.
2. **Old builds** may still have `Restart=on-failure` instead of `Restart=always`. Re-generate the unit with `opencrabs service install` (then `service start`) to pick up the always-restart policy.
3. **Config, keys, commands, and tools hot-reload at runtime.** Editing `config.toml` or `keys.toml` never needs a daemon restart. If a change isn't taking effect, check the logs for a `ConfigWatcher: reloaded` line rather than restarting.

## Cron Management

```bash
# List all cron jobs
opencrabs cron list

# Add a new cron job
opencrabs cron add \
  --name "Daily Report" \
  --cron "0 9 * * *" \
  --tz "America/New_York" \
  --prompt "Check emails and summarize" \
  --provider anthropic \
  --model claude-sonnet-4-20250514 \
  --thinking off \
  --deliver-to telegram:123456

# Remove a cron job (accepts name or ID)
opencrabs cron remove "Daily Report"

# Enable/disable (accepts name or ID)
opencrabs cron enable "Daily Report"
opencrabs cron disable "Daily Report"
```

## TUI Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Send message |
| `Esc` | Cancel / dismiss |
| `Ctrl+N` | New session |
| `Ctrl+L` | Sessions screen |
| `Ctrl+K` | Clear current session |
| `Ctrl+O` | Toggle tool group collapse |
| `\|` | Split pane horizontally |
| `_` | Split pane vertically |
| `Ctrl+X` | Close focused pane |
| `Tab` | Cycle pane focus / Accept autocomplete |
| `Up/Down` | Navigate suggestions / sessions |
| `/` | Start slash command (e.g. `/help`, `/models`) |
| `:` | Start emoji picker |
