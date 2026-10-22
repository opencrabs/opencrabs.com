# WhatsApp

Connect OpenCrabs to WhatsApp via QR code pairing.

## Setup

### Configure via the Onboarding Wizard

Run `/onboard:channels` (or `/onboard` and navigate to the Channels step):

1. Use `↑`/`↓` to focus **WhatsApp**
2. Press `Space` to toggle it on
3. Press `Enter` to open the WhatsApp setup screen
4. The setup screen has two fields:
   - **Connection**: shows the QR code or connection status
   - **Phone Allowlist**: comma-separated phone numbers in E.164 format (e.g. `+15551234567`). Leave empty to accept all messages.
5. When not yet paired, a **QR code** appears in the terminal:
   - Open WhatsApp on your phone → Settings → Linked Devices → Link a Device
   - Scan the QR code
   - The status updates to "Connected" automatically
6. When already paired, press `R` on the Connection field to **reset** and re-pair
7. Press `Enter` to save

The pairing session persists across restarts: no need to re-scan.

### Manual Configuration (advanced)

```toml
# config.toml
[channels.whatsapp]
enabled = true
allowed_phones = ["+15551234567"]
```

## Configuration

WhatsApp options live under `[channels.whatsapp]` in `~/.opencrabs/config.toml`:

```toml
[channels.whatsapp]
enabled = true
allowed_phones = ["+15551234567"]      # E.164 format
session_idle_hours = 24.0              # idle timeout for non-owner sessions
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the WhatsApp channel |
| `allowed_phones` | `[]` (open replies under `auto`, but **never** owner rights) | E.164 phone numbers. Since v0.5.1 an empty list no longer makes every sender the bot owner, owner commands (`/evolve`, `/rebuild`, `/exit`, `/cd`) and approval answers need the self-chat, `bot_owner`, or the resolver over `bot_owner` + `allowed_phones` |
| `session_idle_hours` | `None` (no timeout) | Idle timeout for non-owner sessions. Owner sessions never expire |
| `response_policy` | `"auto"` | Who the bot responds to. `auto`: reply to all while there is at most one active sender, then switch to mention-only once a second unique sender appears. `owner_only`: only the bot owner. `allowlist`: only allowed phones. `open`: everyone |
| `bot_owner` | `None` (auto-seeded from `allowed_phones[0]`) | Phone number of the bot owner in E.164 format. The owner gets access that other allowlisted users do not. Commands that expose personal data or the host system are owner-only |

## How it works

**One account per instance.** Each OpenCrabs instance supports one WhatsApp account (one companion device). You run the bot AS whatever account you scan: your own number (talk via "Message Yourself"), or any other number you own, including a WhatsApp Business account.

**The bot talks to itself.** If you message the bot's own paired number, the bot replies to you. This is by design. The paired account's self-chat is always allowed, regardless of `response_policy` or `allowed_phones`.

**Allowlist behavior.** Anyone messaging the paired number who is on the `allowed_phones`
list gets a reply. The `response_policy` controls who else can interact beyond the allowlist.

**Ownership (v0.5.1, deny-by-default).** Before v0.5.1 an empty `allowed_phones` under
`response_policy = "auto"` answered every contact *and* elevated them all to owner: one
forgotten allowlist away from handing strangers `/evolve` and `/cd`. That elevation is gone:
being answered no longer means being in charge. Owner status comes from the paired
account's self-chat, a configured `bot_owner`, or ownership resolution over
`allowed_phones` + `bot_owner`: never from an empty list. Tool-approval replies
(Yes/Always/No/YOLO) resolve only for the owner on all four channels.


- **Personal and group chats**: Works in DMs and group conversations
- **Image support**: Send and receive images
- **Voice messages**: STT transcription + TTS response
- **Plain text UI**: No buttons (WhatsApp limitation), uses text-based menus
- **Slash commands**: All built-in and custom commands work

## Outbound rate limiting

WhatsApp is the channel with the heaviest ban consequences, so every outbound send goes
through a budget (since v0.5.1):

```toml
[channels.whatsapp.rate_limit]
messages_per_minute = 30   # token bucket: sustained sends per minute
daily_cap = 800            # rolling 24h hard cap
```

| Mechanism | Behavior |
|-----------|----------|
| **Per-minute pacing** | A token bucket sized to `messages_per_minute` (default 30) refills continuously. An over-budget send is **paced**, not dropped: the caller sleeps for the refill gap and then sends. Set `0` to disable pacing. |
| **Rolling daily cap** | The 24h window allows `daily_cap` sends (default 800). On saturation further sends park in a FIFO queue; a background drainer flushes them as the window slides. A queued quote-reply loses its quote context, the text still arrives. Set `0` to disable the cap. |
| **Owner alert** | The first send queued in a saturation episode alerts the bot owner exactly once (through the WhatsApp session), and the alert re-arms when traffic drops back under the cap. |
| **Owner bypass** | Messages addressed to the owner skip budget consumption entirely, so you always get fast replies; the saturation alert itself is also exempt. |

Streaming answers used to post every chunk as its own message, exactly the volume pattern
Meta's heuristics hunt. Outbound replies now edit one message in place (since v0.5.1), which
keeps chunked turns well inside the budget; the limiter is the safety net underneath.

## Formatting Notes

- No markdown tables: use bullet lists
- No headers: use **bold** or CAPS for emphasis
- Links render natively

## Voice Message Handling

When receiving a voice message:
1. Agent downloads and transcribes via STT
2. Sends text response first (searchable)
3. Optionally generates TTS audio response

## Features

- **Personal and group chats**: Works in DMs and group conversations
- **Image support**: Send and receive images
- **Voice messages**: STT transcription + TTS response
- **Plain text UI**: No buttons (WhatsApp limitation), uses text-based menus
- **Slash commands**: All built-in and custom commands work

## Surface (v0.5.2)

The WhatsApp channel grew to match the phone app's surface:

- **Outbound**: native voice notes (opus), edit-in-place streaming for long replies, pin, forward, and profile actions
- **Inbound**: video, stickers, location, contacts, reactions, and poll votes are parsed into the turn; presence events and media recovery fill gaps after reconnects
- **Sessions**: disappearing messages honored per chat, contact blocking, per-chat history with reaction acknowledgements
- **Surfaces**: status updates and newsletter discovery, native-flow buttons for interactive prompts; follow-up suggestion sets that exceed the native button cap render as a poll, and a vote selects the option (#1616)

## Troubleshooting

### Wrong number replying

Each OpenCrabs instance supports one WhatsApp account (one companion device). If you connected multiple numbers or the wrong number is replying, you need a full reset. The bot always uses the last number you connected.

> **Critical:** Always reset the connection before connecting a new number. OpenCrabs only keeps the last paired number.

**Full reset steps:**

1. **Remove ALL linked devices from WhatsApp.** Open WhatsApp on your phone, go to **Settings > Linked Devices**, and remove every device in the list. Don't bother hunting for the `opencrabs` one. A stale device left over from an earlier pairing is the usual reason the old number keeps replying, so the reliable fix is to clear them all and start fresh.
2. **Reset the connection in OpenCrabs**: in the TUI or from a channel, go to `/onboard:channels` and press **R** to reset the WhatsApp connection. Wait for confirmation that the reset is complete.
3. **Re-pair from scratch**: after the reset is confirmed, go to WhatsApp > **Settings > Linked Devices** > **Link a Device** and scan the new QR code shown by OpenCrabs.

If the bot still shows the old number after resetting, make sure you completed step 1 (removing the device from WhatsApp) before step 2.

### Common issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| Old number still replying | A stale linked device is still paired | Remove **ALL** linked devices from WhatsApp > Settings > Linked Devices, then press **R** in `/onboard:channels` and re-pair |
| QR code doesn't appear | Agent is still connected (no restart triggered) | Press **R** in `/onboard:channels` to force a restart, then wait for the new QR |
| Bot doesn't reply to anyone | `response_policy` is too restrictive | Set `response_policy = "allowlist"` and add phone numbers to `allowed_phones` in `config.toml` |
| Bot replies to everyone | `response_policy` is `open` | Set `response_policy = "allowlist"` or `"owner_only"` in `config.toml` |
| Bot doesn't reply to self-chat | `allowed_phones` doesn't include the paired number | The paired number's self-chat is always allowed, regardless of `allowed_phones`. If it's not working, check that `response_policy` isn't too restrictive |
