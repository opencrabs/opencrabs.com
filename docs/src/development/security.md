# Security

## Threat Model

OpenCrabs runs locally on your machine with access to your filesystem and shell. Security focuses on:

1. **API key protection**: Keys never leave your machine except to their respective providers
2. **Network exposure**: Minimal attack surface by default
3. **Tool execution**: Sandboxed with user approval

## API Key Storage

Keys are stored in `~/.opencrabs/keys.toml`:

- File permissions: `600` (owner read/write only)
- Keys are loaded into memory with `zeroize`: zeroed on drop
- Keys are never logged or included in conversation history
- Keys are never sent to any provider other than their own

## Network Security

### A2A Gateway

- Binds to `127.0.0.1` (loopback) by default
- CORS disabled unless explicitly configured
- A bearer token (`[a2a] api_key`) is the only authorization boundary in front of the agent's
  tool surface, and since v0.5.1 the gateway refuses to start as an open gate: with no key
  configured, a non-loopback bind is rejected at boot. With a key set, every request must
  present it, compared in constant time. For public exposure still use a reverse proxy + TLS

### Channel Connections

- All channel APIs use TLS (HTTPS/WSS)
- Telegram: long polling over HTTPS
- Discord: WebSocket with TLS
- Slack: Socket Mode (WebSocket)
- WhatsApp: Noise protocol encryption

### Config Files & Logs (v0.5.1)

- Home files (`config.toml`, `keys.toml`, brain files) are written with owner-only
  permissions from every write path: the mode is derived from the `Config` struct itself,
  so a new writer cannot forget it
- The daemon log moved off world-readable `/tmp` into `~/.opencrabs/logs/`
- Secret scrubbing covers all delivery paths, not only chat output

## Tool Approval

Tools that modify your system require approval:

- **File writes**: Shows the file path and diff
- **Shell commands**: Shows the exact command before execution
- **Git operations**: Push, commit, branch operations

Auto-approve mode (`--auto-approve`) bypasses this for automation use cases like cron jobs.

Channel approvals are **owner-only** since v0.5.1: the Yes / Always / YOLO buttons and text
replies on Telegram, Discord, Slack, and WhatsApp resolve only for the bot owner (canonical
resolver), never for an allowlisted non-owner who happened to trigger the tool. The default
`approval_policy` is `auto-always` by design (you are not meant to babysit prompts) because
the channel allowlists (below) are the actual access boundary. To confirm every tool call
instead, set `approval_policy = "ask"` in `[agent]`.

## Data Storage

- All data stored locally in `~/.opencrabs/opencrabs.db` (SQLite)
- No telemetry or analytics
- No data sent to OpenCrabs servers (there are none)
- Conversation history stays on your machine

## Reporting Vulnerabilities

If you discover a security vulnerability, please report it responsibly:

- Email: adolfo@meetneura.ai
- Do not open a public issue for security vulnerabilities
- We will acknowledge receipt within 48 hours

## Safety Gates (v0.3.78)

OpenCrabs v0.3.78 introduced a layered safety-gate system:

### TOML Bash Blocklist
Shell commands are checked against a TOML-defined blocklist before execution. Dangerous patterns (recursive deletes on root, raw disk writes, fork bombs) are rejected with a clear error. The blocklist is user-extensible and hot-reloads on file change. See [Configuration](../getting-started/configuration.md#toml-bash-blocklist-v0378) for details.

### Secret Redaction
Quoted secrets in agent output are now **redacted before delivery**. This covers:
- Bearer tokens
- API key patterns
- Colon-token shapes (key: value in YAML-like output)
- Query-param keys
- URL passwords

Redaction fires on all delivery paths (TUI, Telegram, Discord, Slack, WhatsApp), so a secret accidentally echoed by the agent never reaches the chat.

### Brain File Verification
Post-write verification (via brain_verify.toml) ensures brain files maintain their structural integrity after RSI writes. Required anchors (H1 title, Owns header) are checked, and writes that break structure are flagged.

## Harness-Level Denials (v0.5.1)

Prompt instructions are a request; the harness is a wall. The v0.5.1 hardening batch moved
the load-bearing safety rules out of brain-file prose and into code paths that run even when
the model is jailbroken or an RSI pass weakens a prompt file:

- **Confidential-path deny on `read_file`**: resolved paths matching the agent's own
  `keys.toml`, anything under a `.ssh/` directory, SSH private-key names, `.env` files,
  `.pem` / `.key` / `.pfx` material, `/etc/shadow`, or files that name themselves a
  credential are refused **before a byte is read**. Public keys and ordinary files are
  unaffected, and `..`/`~` spelling does not change the verdict.
- **One SSRF guard for every fetch tool**: `web_scrape` and `http_request` share
  `brain::tools::ssrf`: cloud metadata endpoints (by name as well as IP), RFC1918/LAN,
  IPv6 link-local, the unspecified addresses, CGNAT `100.64.0.0/10`, and IPv4-mapped IPv6
  forms are blocked; domains are DNS-resolved and **every** answer checked before the
  request, and each redirect hop is re-validated, so a public-looking name or a 302 cannot
  smuggle an internal target. Loopback stays deliberately allowed: a local-first agent
  legitimately reaches the operator's own machine.
- **The bash blocklist floor everywhere**: `execute_code` and dynamic shell tools now pass
  through the same blocklist as `bash` (hardcoded plus the user TOML list), checked before
  writing or running, for every language, since `os.system`/`subprocess` can carry the same
  command. Dynamic tools also cannot be named like core tools: `add_tool` refuses to create
  an unguarded stand-in for `bash`, `read_file`, `evolve`, and friends.
- **Deny-by-default channel allowlists**: an unconfigured Discord or Slack channel now
  refuses everyone (matching Telegram): empty `allowed_users`/`allowed_roles` is no
  longer "accept all". On WhatsApp an empty `allowed_phones` no longer elevates strangers
  to owner. Details per channel in [Channels overview](../channels/overview.md).
- **Verified self-update**: `/evolve` checks the release `SHA256SUMS` against the download
  before swapping the binary, and the download host is pinned, so a redirected release
  asset cannot hot-swap an unverified binary.

These complement the v0.3.78 safety gates above; together they mean a stranger who reaches
a channel, or a prompt injection that reaches a tool, meets code that refuses, not prose
that asks politely.
