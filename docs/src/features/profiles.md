# Multi-Profile Support

Run multiple isolated OpenCrabs instances from a single installation. Each profile gets its own config, memory, sessions, brain files, skills, cron jobs, and gateway service.

Introduced in **v0.2.94**.

## Why Profiles?

Common use cases:
- **Work vs personal**: separate API keys, brain files, Telegram bots
- **Multiple clients**: different persona and config per customer
- **Model experimentation**: compare different provider setups without clobbering your main config
- **Staging vs production**: test brain file changes on a staging profile before rolling to your main agent

## Creating a Profile

```bash
# Create a new profile
opencrabs profile create hermes

# List all profiles
opencrabs profile list

# Show details for a profile
opencrabs profile show hermes

# Delete a profile
opencrabs profile delete hermes
```

## Switching Profiles

There are two ways to use a non-default profile:

```bash
# CLI flag (per-session)
opencrabs -p hermes

# Environment variable (persistent)
export OPENCRABS_PROFILE=hermes
opencrabs
```

The default profile (`~/.opencrabs/`) works exactly as before: zero breaking changes.

## Directory Structure

Each profile gets its own directory under `~/.opencrabs/profiles/<name>/`:

```
~/.opencrabs/
├── config.toml          # default profile config
├── memory/              # default profile memory
├── sessions.db          # default profile sessions
└── profiles/
    ├── hermes/
    │   ├── config.toml
    │   ├── memory/
    │   ├── sessions.db
    │   ├── logs/
    │   └── layout/
    └── assistant/
        ├── config.toml
        └── ...
```

## Profile Migration

Copy config and brain files from one profile to another:

```bash
# Copy from default to hermes
opencrabs profile migrate --from default --to hermes

# Overwrite existing files in target
opencrabs profile migrate --from default --to hermes --force
```

Migration copies all `.md` and `.toml` files plus the `memory/` directory. It **excludes** the database, sessions, logs, and layout state, so the target profile starts fresh with the source's personality and configuration, not its history.

## Export and Import

Share profiles as portable archives:

```bash
# Export a profile as .tar.gz
opencrabs profile export hermes
# → creates hermes.tar.gz in current directory

# Import on another machine
opencrabs profile import ./hermes.tar.gz
```

## Token-Lock Isolation

Two profiles cannot bind the same bot token simultaneously. Before connecting a Telegram, Discord, Slack, or Trello channel, OpenCrabs checks for existing token locks using PID-based lock files:

```
~/.opencrabs/locks/telegram_<token_hash>.lock
```

If another profile (still running) holds the lock, startup fails with a clear message. Stale locks (process dead) are automatically cleaned up.

This prevents split-brain scenarios where two agents fight over the same bot.

## Profile-Aware Daemons

Install a separate OS service per profile:

```bash
# Install daemon for the hermes profile
opencrabs -p hermes service install

# Start it
opencrabs -p hermes service start

# macOS: creates com.opencrabs.daemon.hermes.plist
# Linux: creates opencrabs-hermes.service
```

Multiple profile daemons can run simultaneously as separate OS services, each with its own ports, bot connections, and config.

## Per-Session Provider Isolation

Changing the provider in one session does **not** affect other sessions or profiles. Each session remembers its own provider independently. See [Sessions](./sessions.md) for the full isolation story.

## Profile-Aware Paths (v0.3.35)

All internal paths now resolve through `opencrabs_home()` instead of hardcoded `~/.opencrabs/`. This means subagent status directories, `tools.toml` fallback resolution, and `write_opencrabs_file` confirmation messages all respect the active profile. No more cross-profile contamination when running multiple instances.

## /profiles Command and TUI Dialog (v0.3.55)

The `/profiles` command works across every channel: TUI, Telegram, Discord, WhatsApp, and Slack.

**In the TUI**, `/profiles` opens a native dialog with full keyboard navigation:

- **Browse** all profiles with active profile highlighted
- **Create** a new profile inline with name validation
- **Delete** a profile with confirmation
- **Migrate** brain files and config from one profile to another
- **Switch** instantly to another profile

On **Discord, WhatsApp, and Slack**, `/profiles` renders a rich profile browser showing all profiles, the active one, and quick switch/delete/create actions.

```bash
# From any channel
/profiles

# In the TUI, use j/k to navigate, Enter to select, d to delete, c to create, m to migrate
```
