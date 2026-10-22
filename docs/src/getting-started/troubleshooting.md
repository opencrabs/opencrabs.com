# Troubleshooting

Common issues and how to fix them.

## Windows Defender Blocking OpenCrabs

Windows Defender may flag `opencrabs.exe` as suspicious because it's an unsigned binary that executes shell commands and makes network requests. This is a false positive.

**Add an exclusion:**

1. Open **Windows Security** → **Virus & threat protection**
2. **Virus & threat protection settings** → **Manage settings**
3. **Exclusions** → **Add or remove exclusions**
4. **Add an exclusion** → **File** → select `opencrabs.exe`

**Or via PowerShell (admin):**
```powershell
Add-MpPreference -ExclusionPath "C:\path\to\opencrabs.exe"
```

If SmartScreen blocks the first run, click **More info** → **Run anyway**.

---

## Binary Won't Start or Crashes

### Startup Errors

Run with debug logging to see what's failing:

```bash
opencrabs -d chat
```

Logs are written to `~/.opencrabs/logs/`.

### Download a Previous Version

If the latest release crashes on your machine, download a previous working version from [GitHub Releases](https://github.com/opencrabs/opencrabs/releases):

```bash
# List all releases
gh release list -R opencrabs/opencrabs

# Download a specific version
gh release download v0.2.66 -R opencrabs/opencrabs -p "opencrabs-*$(uname -m)*$(uname -s | tr A-Z a-z)*"
```

---

## /evolve: Update & Rollback

`/evolve` downloads the latest release from GitHub and hot-swaps the binary. It has built-in safety checks:

1. **Download**: Fetches the platform-specific binary from GitHub Releases
2. **Pre-swap health check**: Runs `opencrabs health-check` on the new binary (10s timeout). If it fails, the new binary is deleted and your current version stays untouched.
3. **Backup**: Creates a backup at `<binary-path>.evolve_backup`
4. **Atomic swap**: Replaces the current binary
5. **Post-swap health check**: Verifies the swapped binary works. If it fails, **auto-rolls back** to the backup.
6. **Restart**: exec()-restarts into the new version
7. **Brain update prompt**: After restart, your crab announces the new version, diffs brain templates against your local files, and offers to update them

### If /evolve Fails

The most common reason is the health check caught an issue, your current version stays safe. If something went wrong after the swap:

```bash
# Restore the backup manually
cp /path/to/opencrabs.evolve_backup /path/to/opencrabs
chmod +x /path/to/opencrabs
```

### Cargo Install Fallback

When `/evolve` uses `cargo install` (building from source), it tries the **stable** toolchain first. If that fails, it automatically falls back to `cargo +nightly`. The progress message shows which toolchain succeeded.

### Check-Only Mode

The agent can check for updates without installing:

```
/evolve check_only=true
```

---

## Bash Tool Safety

The bash tool includes a **hard command blocklist** that prevents catastrophic commands even if accidentally approved:

- `rm -rf /`, `sudo rm -rf .`
- `mkfs`, `dd` to `/dev/`
- Fork bombs
- `/etc` overwrites, `/proc` writes
- Sensitive file exfiltration
- Crypto mining commands

These are blocked at the tool level: no configuration needed.

---

## Older CPUs (Pre-2011 / No AVX)

Some features require AVX/AVX2 instructions. Since v0.2.67, OpenCrabs **detects CPU capabilities at runtime** and automatically hides unavailable options in the onboarding wizard.

### What's Affected

| Feature | CPU Requirement | Fallback |
|---------|----------------|----------|
| Local embeddings (memory search) | AVX (Sandy Bridge 2011+) | FTS-only keyword search (still works) |
| Local STT (rwhisper/candle) | AVX2 (Haswell 2013+) | API mode (Groq Whisper) or disabled |
| Local TTS (Piper) | None: tested on 2007 iMac | Works on any x86/ARM CPU |

### Symptoms

- Local STT option doesn't appear in `/onboard:voice`: your CPU lacks AVX2
- Local TTS (Piper) should always be available: no CPU restrictions, works on machines as old as 2007
- Memory search falls back to text-only FTS silently
- Crash with "illegal instruction" on very old CPUs

### Fix: Build from Source with CPU Targeting

```bash
# For your specific CPU (best performance)
RUSTFLAGS="-C target-cpu=native" cargo build --release

# For Sandy Bridge (AVX but no AVX2)
RUSTFLAGS="-C target-cpu=sandybridge" cargo build --release
```

### macOS with Apple Silicon

Local STT uses **Metal GPU acceleration** on macOS, no CPU flags needed. Works out of the box on M1/M2/M3/M4.

---

## Config Issues

### Config Won't Load

If `config.toml` has a syntax error, OpenCrabs will fail to start. Restore from backup:

```bash
# Check if a backup exists
ls ~/.opencrabs/config.toml.backup

# Restore it
cp ~/.opencrabs/config.toml.backup ~/.opencrabs/config.toml
```

Or reinitialize with defaults:

```bash
opencrabs init --force
```

> **Warning:** `--force` overwrites your config. Back up `keys.toml` first, it contains your API keys.

### Manual Backup

Always keep a backup of your critical files:

```bash
cp ~/.opencrabs/config.toml ~/.opencrabs/config.toml.backup
cp ~/.opencrabs/keys.toml ~/.opencrabs/keys.toml.backup
cp ~/.opencrabs/commands.toml ~/.opencrabs/commands.toml.backup
```

### RSI Has Never Run

Mission Control shows `⚠ RSI has never run: N tool event(s) recorded and unprocessed` even though `rsi_enabled = true` is set.

Most common cause: `self_improvement_provider` is misspelled. Its value is the **bare provider section name**, no `custom:` prefix, no model appended:

```toml
[providers.custom.glm-53-max]              # the name is "glm-53-max"

[agent]
self_improvement_provider = "glm-53-max"   # ✅ not "custom:glm-53-max"
self_improvement_model = "..."             # the model goes in its own key
```

Checklist:

1. The provider value matches the config section name exactly (`[providers.custom.moonshotai]` → `"moonshotai"`).
2. No `custom:` / `custom.` / `custom/` prefix anywhere in a provider reference.
3. The model is in its own key (`self_improvement_model`), never appended to the provider.
4. Restart after fixing: Mission Control confirms cycles on the next run.

> Fixed at source in [opencrabs#1314](https://github.com/opencrabs/opencrabs/issues/1314): current main normalises misspelled values with a warning. Released versions fail silently, which is exactly why the spelling matters.

---

## Channel Issues

### Telegram

**Bot not responding:**
1. Verify token from @BotFather is in `keys.toml`
2. Check your numeric user ID is in `allowed_users`
3. If `respond_to = "mention"`, you must @mention the bot in groups

**Regenerate bot token:**
1. Open @BotFather on Telegram
2. `/mybots` → select bot → API Token → Revoke
3. Copy new token to `keys.toml` under `[channels.telegram]`
4. Restart OpenCrabs

**Re-setup from scratch:** Run `/onboard:channels` in the TUI.

### Telegram Won't Connect / Reconnect

If the Telegram bot stops responding or you need to re-link it, re-run the channels setup and re-confirm the token + your numeric user ID.

**Fix:**

1. Run `/onboard:channels` (TUI: opens the wizard; on a channel: the agent walks you through it).
2. Paste your **bot token** again if it's missing (get it from [@BotFather](https://t.me/BotFather)).
3. Paste your **numeric user ID** and hit Enter to confirm.
4. If the bot sends you a message on Telegram, it worked.

On a channel you can do it in one line: `/onboard:channels telegram <BOT_TOKEN> <YOUR_NUMERIC_ID>`.

**Why you have to provide your numeric ID:** Telegram's Bot API exposes only the bot's identity from a token (via `getMe`), it has no way to reveal who created the bot in BotFather. A bot only learns a human's ID when that human messages it. The onboarding wizard auto-detects your ID via `getUpdates` when you leave the field blank, but that only works if (a) you've already messaged the bot and (b) the bot isn't already running and consuming those updates, which is exactly the case during a reconnect. So on reconnect, message the bot first, or just paste the ID (get it from [@userinfobot](https://t.me/userinfobot)).

### WhatsApp

**QR code / session expired:**

WhatsApp sessions are stored at `~/.opencrabs/whatsapp/session.db`. To reconnect:

```bash
# Delete the session file
rm ~/.opencrabs/whatsapp/session.db

# Re-pair via onboarding
opencrabs chat --onboard
```

Or press `R` on the WhatsApp onboarding screen to reset and get a fresh QR code.

**Messages not received:**
- Verify phone number is in `allowed_phones` using E.164 format: `"+15551234567"`
- Empty `allowed_phones = []` means accept from everyone

### WhatsApp Won't Connect / Wrong Number Replying

Each OpenCrabs instance supports one WhatsApp account (one companion device). If you connected multiple numbers or the wrong number is replying, you need a full reset. The bot always uses the last number you connected.

> **Critical:** Always reset the connection before connecting a new number. OpenCrabs only keeps the last paired number.

**Full reset steps:**

1. **Remove ALL linked devices from WhatsApp.** Open WhatsApp on your phone, go to **Settings > Linked Devices**, and remove every device in the list. Don't bother hunting for the `opencrabs` one. A stale device left over from an earlier pairing is the usual reason the old number keeps replying, so the reliable fix is to clear them all and start fresh.
2. **Reset the connection in OpenCrabs**: in the TUI or from a channel, go to `/onboard:channels` and press **R** to reset the WhatsApp connection. Wait for confirmation that the reset is complete.
3. **Re-pair from scratch**: after the reset is confirmed, go to WhatsApp > **Settings > Linked Devices** > **Link a Device** and scan the new QR code shown by OpenCrabs.

If the bot still shows the old number after resetting, make sure you completed step 1 (removing the device from WhatsApp) before step 2.

**Common issues:**

| Symptom | Cause | Fix |
|---------|-------|-----|
| Old number still replying | A stale linked device is still paired | Remove **ALL** linked devices from WhatsApp > Settings > Linked Devices, then press **R** in `/onboard:channels` and re-pair |
| QR code doesn't appear | Agent is still connected (no restart triggered) | Press **R** in `/onboard:channels` to force a restart, then wait for the new QR |
| Bot doesn't reply to anyone | `response_policy` is too restrictive | Set `response_policy = "allowlist"` and add phone numbers to `allowed_phones` in `config.toml` |
| Bot replies to everyone | `response_policy` is `open` | Set `response_policy = "allowlist"` or `"owner_only"` in `config.toml` |
| Bot doesn't reply to self-chat | `allowed_phones` doesn't include the paired number | The paired number's self-chat is always allowed, regardless of `allowed_phones`. Check that `response_policy` isn't too restrictive |

### Discord

**Bot not receiving messages:**
1. Ensure **Message Content Intent** is enabled in Discord Developer Portal → Bot settings
2. Required intents: `gateway`, `guild_messages`, `direct_messages`, `message_content`
3. Use the **bot token** (starts with `MTk...`), not the application ID

**Regenerate token:** Discord Developer Portal → Bot → Regenerate Token

### Slack

**Both tokens required:**
- **Bot token** (`xoxb-...`): For sending messages
- **App token** (`xapp-...`): For Socket Mode (receiving events)

Without the app token, the bot can send but not receive messages.

**Socket Mode:** Must be enabled in app settings → Features → Socket Mode → ON

### Trello

**Setup:**
1. Get API key: [trello.com/app-key](https://trello.com/app-key)
2. Generate token from the same page
3. Add `board_ids` to config: the bot only monitors listed boards
4. Set `poll_interval_secs > 0` to enable polling (default 0 = disabled)

### General: Re-run Channel Setup

For any channel issues, re-run the onboarding wizard:

```bash
opencrabs chat --onboard
```

Or type `/onboard:channels` in the TUI.

---

## Agent Hallucinating Tool Calls

If the agent starts sending tool call approvals that don't render in the UI (meaning it believes it executed actions that never actually ran) the session context has become corrupted.

**Fix:** Start a new session.

1. Press `/` and type `sessions` (or navigate to the Sessions panel)
2. Press **N** to create a new session
3. Continue your work in the fresh session

This reliably resolves the issue. A fix is coming in a future release.

---

## Claude Code CLI Refuses to Run as Root (VPS, Docker)

**Symptom:** the `claude_cli` provider is enabled and `claude` is authenticated, but every request fails as soon as it spawns. In the log the CLI exits immediately with a message about `--dangerously-skip-permissions` not being allowed for root, or the first spawn hangs waiting on a prompt nobody can answer.

**Cause:** OpenCrabs runs the CLI headless with `--dangerously-skip-permissions` (it owns tool execution itself, so the CLI never asks). Claude Code refuses that flag when the process runs as root unless it believes it is inside a sandbox, and on a fresh install it also wants a one-time interactive acknowledgement of that mode. A VPS or container where OpenCrabs runs as root hits both.

**Fix:** put the acknowledgement and the sandbox marker in the CLI's own settings file, `~/.claude/settings.json` for the user OpenCrabs runs as (`/root/.claude/settings.json` on a root box):

```json
{
  "permissions": { "defaultMode": "bypassPermissions" },
  "skipDangerousModePermissionPrompt": true,
  "env": { "IS_SANDBOX": "1" }
}
```

Merge these keys into the file if it already exists (keep `model`, `theme`, and the rest). The CLI reads `settings.json` on every invocation, so no OpenCrabs restart is needed; the next spawn goes through. Verify as the same user:

```bash
echo "say OK" | claude -p --dangerously-skip-permissions
```

`IS_SANDBOX=1` only tells Claude Code that the root check may be skipped. It does not sandbox anything, so keep the box firewalled the way you would for any root agent. On a desktop where OpenCrabs runs as your own user none of this is required.

## Daemon Stays Down

If `opencrabs service status` says stopped:

1. **Did you open the TUI?** Opening `opencrabs` deliberately shuts the daemon down so the interactive session can own the channels. The daemon stays down until you `opencrabs service start` again.
2. **Old builds** may still have `Restart=on-failure` instead of `Restart=always`. Re-generate the unit with `opencrabs service install` (then `service start`) to pick up the always-restart policy.
3. **Config, keys, commands, and tools hot-reload at runtime.** Editing `config.toml` or `keys.toml` never needs a daemon restart. If a change isn't taking effect, check the logs for a `ConfigWatcher: reloaded` line rather than restarting.

For full daemon/service documentation, see [CLI Commands: Daemon Mode vs TUI](../getting-started/cli-commands#daemon-mode-vs-tui).

---

## Local STT (Speech-to-Text)

Since v0.2.67, local STT uses **rwhisper** (candle, pure Rust) instead of whisper-rs/ggml. On macOS, it uses **Metal GPU acceleration** automatically.

### Models

| Model | Size | Quality |
|-------|------|---------|
| `quantized-tiny` | ~42 MB | Good for short messages |
| `base-en` | ~142 MB | Better accuracy (English) |
| `small-en` | ~466 MB | High accuracy |
| `medium-en` | ~1.5 GB | Best accuracy |

Models download automatically from HuggingFace on first use.

### Common Issues

**Local STT option not showing in wizard:** Your CPU lacks AVX2. Use API mode (Groq Whisper) instead, or build from source with `RUSTFLAGS="-C target-cpu=native"`.

**"No audio samples decoded":** Audio file is corrupt or unsupported format. Supported: OGG/Opus, WAV.

**Transcription hangs:** Times out after 300 seconds. Try a smaller model (`quantized-tiny`).

**Model download fails:** Check network connection. Models are fetched from HuggingFace.

**Audio too short:** Messages under 1 second are automatically padded to prevent tensor errors.

### Disabling

```toml
[voice]
stt_enabled = false
```

---

## Local TTS (Text-to-Speech)

### Requirements

- **Python 3** must be installed and in PATH
- Piper installs automatically in a venv at `~/.opencrabs/models/piper/venv/`

### Voices

| Voice | Description | Size |
|-------|-------------|------|
| `ryan` | US Male (default) | ~200-400 MB |
| `amy` | US Female | ~200-400 MB |
| `lessac` | US Female | ~200-400 MB |
| `kristin` | US Female | ~200-400 MB |
| `joe` | US Male | ~200-400 MB |
| `cori` | UK Female | ~200-400 MB |

### Common Issues

**"python3 -m venv failed":** Install Python 3. On Ubuntu: `sudo apt install python3 python3-venv`. On macOS: `brew install python3`.

**"pip install piper-tts failed":** Network issue or pip corrupted. Fix pip first:
```bash
python3 -m pip install --upgrade pip
```

**Telegram voice messages show no waveform:** This was fixed in v0.2.64, audio is now properly encoded as OGG/Opus (RFC 7845). Update to latest version.

**Voice preview not playing:** Preview uses `afplay` (macOS), `aplay` (Linux), or `powershell` (Windows). Ensure audio output is available.

### Re-setup

Run `/onboard:voice` in the TUI to reconfigure STT/TTS mode and re-download models.

### Disabling

```toml
[voice]
tts_mode = "off"
```

---

## Local Embeddings (Memory Search)

The memory search engine uses a ~300 MB embedding model (llama.cpp) for semantic search. It requires **AVX** on x86 CPUs.

### Fallback

If embeddings can't initialize (no AVX, download failed, disk full), memory search falls back to **FTS-only** (keyword matching). It still works, just less semantic.

### Fix for Older CPUs

Build from source with CPU targeting (see [Older CPUs](#older-cpus-pre-2011--no-avx) above).

### Model Location

Models are stored in `~/.local/share/opencrabs/models/` (platform-specific data directory).

---

## Database Issues

### Location

- **Main database:** `~/.opencrabs/opencrabs.db` (SQLite + WAL)
- **WhatsApp session:** `~/.opencrabs/whatsapp/session.db`

### Database Corruption

SQLite with WAL mode is very resilient, but if corruption occurs:

```bash
# Back up the corrupted file first
cp ~/.opencrabs/opencrabs.db ~/.opencrabs/opencrabs.db.corrupt

# Reinitialize (WARNING: loses all history)
opencrabs db init
```

### Migration Errors

The database automatically migrates on startup (11 migrations). If migrating from an older version with sqlx, the transition is handled automatically, no manual steps needed.

---

## Building from Source

### Quick Setup

```bash
curl -fsSL https://raw.githubusercontent.com/opencrabs/opencrabs/main/scripts/setup.sh | bash
git clone https://github.com/opencrabs/opencrabs.git && cd opencrabs
cargo build --release
```

### Build Without Voice (Smaller Binary)

```bash
cargo build --release --no-default-features --features telegram,whatsapp,discord,slack,trello
```

### Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `local-stt` | On | whisper.cpp for local speech-to-text |
| `local-tts` | On | Piper for local text-to-speech |
| `telegram` | On | Telegram channel |
| `whatsapp` | On | WhatsApp channel |
| `discord` | On | Discord channel |
| `slack` | On | Slack channel |
| `trello` | On | Trello channel |

---

## Debug Mode

Run with `-d` for verbose logging:

```bash
opencrabs -d chat
```

Logs go to `~/.opencrabs/logs/` with 7-day retention.
