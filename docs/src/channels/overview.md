# Channel Integrations

OpenCrabs connects to multiple messaging platforms simultaneously. All channels share the TUI session by default, with per-user sessions for non-owners.

## Setting Up Channels

Channels are configured through the **onboarding wizard**, not by editing TOML files manually.

### Running the Wizard

- **First launch**: the wizard runs automatically
- **Re-run**: type `/onboard` in chat, or `/onboard:channels` to jump straight to the channels step
- **Quick jump**: `/onboard:channels` opens the channel picker and returns to chat when done

### Navigation

The channel picker is a keyboard-driven TUI screen:

| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Move focus between channels |
| `Space` | Toggle the focused channel on/off |
| `Enter` on an enabled channel | Open that channel's setup screen |
| `Enter` on **Continue** | Skip remaining setup and advance |
| `Tab` | Same as Continue: advance to the next wizard step |
| `Esc` | Go back to the previous step |

### Channel Setup Screens

When you press `Enter` on an enabled channel, a dedicated setup screen opens with the fields needed for that platform (bot token, channel ID, allowed users, etc.). Each field:

- **Auto-detects existing values** from `config.toml` / `keys.toml` (shown as masked `••••••••` for secrets, plain text for IDs)
- **Tab** moves to the next field
- **Enter** on the last field (or the Test Connection button) saves and returns to the channel list
- **BackTab** moves to the previous field

### The Five Channels

| # | Channel | Setup Fields | Test |
|---|---------|-------------|------|
| 0 | Telegram | Bot Token, Owner User ID, Respond To | Send test message |
| 1 | Discord | Bot Token, Channel ID, Allowed Users, Respond To | Send test message |
| 2 | WhatsApp | QR Code scan, Phone Allowlist | Connection status |
| 3 | Slack | Bot Token, App Token, Channel ID, Allowed Users, Respond To | Send test message |
| 4 | Trello | API Key, API Token, Board ID, Allowed Users | Board access check |

After enabling and configuring your channels, the wizard saves everything to `config.toml` and `keys.toml` automatically. You can always re-run `/onboard:channels` to modify settings.

## Supported Channels

| Channel | Protocol | Images In | Voice In | Image Gen Out | Setup |
|---------|----------|-----------|----------|---------------|-------|
| [Telegram](./telegram.md) | Long polling | Vision pipeline | STT | Native photo | Bot token |
| [Discord](./discord.md) | WebSocket | Vision pipeline | STT | File attachment | Bot token |
| [Slack](./slack.md) | Socket Mode | Vision pipeline | STT | File upload | Bot + App token |
| [WhatsApp](./whatsapp.md) | QR pairing | Vision pipeline | STT | Native image | QR code |
| [Trello](./trello.md) | REST API | Card attachments | - | Card attachment | API key + token |

## Cross-Channel Session Resolution (v0.3.29)

All messaging channels now share a stable `[chat:<id>]` suffix pattern for reliable session lookup. Previously only Telegram had this; Discord, Slack, and WhatsApp used exact-title matching which broke when the agent auto-renamed sessions (creating duplicates on every message).

The shared `channels::session_resolve` module provides:
- **Suffix-first lookup**: fast path using `[chat:discord-dm-<user_id>]`, `[chat:slack-<channel_id>]`, `[chat:wa-<phone>]` etc.
- **Legacy forward-migration**: pre-suffix rows are migrated to the suffix format on first lookup
- **`/sessions` binding**: explicit chat→session binding on switch so user choices win over suffix lookup

## Follow-Up Cancel (v0.3.30)

Sending a message while the agent is mid-run now acts as **ESC x2** (cancel current run) across all channels. The cancelled partial content is preserved, and the new message starts a fresh agent turn.

## ZIP Attachments (v0.3.30)

ZIP file attachments from users are extracted and processed inline:
- Text files are inlined into the conversation
- Images get vision markers for multimodal processing
- PDFs get text extraction
- Capped at **50 files / 10 MB** per ZIP entry

## Group Onboarding & /cowork (v0.3.74)

Getting a bot into a group and making it useful is now a single flow:

- **/cowork opens the group**: `/cowork` sets the target group `open = true` and persists it (#718).
- **/cowork admin deep link**: the `/cowork` deep link requests admin rights, so the bot joins already promoted (#709).
- **Group onboarding greeting**: the bot greets the group with an onboarding nudge when it joins (#707, #708).
- **Per-group /start registration**: `/start` in a group registers the member per-group instead of the stale `config.toml` lecture (#708).
- **Register joining members**: any member who joins a bot group is auto-registered (#710).
- **Secure by default**: member auto-registration is gated to `open = true` groups (#717).

## Follow-Up Suggestions (v0.3.74)

- **follow_up_question degrades gracefully**: returns the question as plain text on non-interactive surfaces (#716).
- **Telegram follow-up taps**: `suggest_followups` renders last, fixing taps doing nothing (#723, #724).
- **Accurate description**: the `suggest_followups` tool description is channel-accurate, so the model invokes it instead of writing plain-text suggestions (#706).
- **Outbound media dedup**: an identical file+caption re-uploaded to the same chat within a short window is collapsed, so the agent can't send it twice (#721).

## Common Features

All messaging channels support:

- **Shared session** with TUI (owner) or per-user sessions (non-owners)
- **Slash commands**: `/help`, `/models`, `/new`, `/sessions`, custom commands
- **Inline buttons**: Provider picker, model picker, session switcher (Telegram, Discord, Slack)
- **User allowlists**: Restrict access by user ID, chat ID, role, or phone number. Since
  v0.5.1 the allowlists are **deny-by-default** and are the access boundary: on Discord and
  Slack an unconfigured channel (no `allowed_users`, no `allowed_roles`, no `bot_owner`)
  refuses every message, matching Telegram; on WhatsApp an empty `allowed_phones` no longer
  grants anyone owner rights. Because the default `approval_policy` is `auto-always`, the
  allowlist is what decides who can drive tools at all, set it deliberately
- **`respond_to` filter**: `all`, `dm_only`, or `mention` (respond only when @mentioned)

## File & Media Support

| Channel | Images (in) | Text files (in) | Documents (in) | Audio (in) | Image gen (out) |
|---------|-------------|-----------------|----------------|------------|-----------------|
| Telegram | Vision pipeline | Extracted inline | PDF note | STT | Native photo |
| WhatsApp | Vision pipeline | Extracted inline | PDF note | STT | Native image |
| Discord | Vision pipeline | Extracted inline | PDF note | STT | File attachment |
| Slack | Vision pipeline | Extracted inline | PDF note | STT | File upload |
| Trello | Card attachments → vision | Extracted inline | - | - | Card attachment |
| TUI | Paste path → vision | Paste path → inline | - | STT | `[IMG: name]` display |

Images are passed to the active model's vision pipeline if it supports multimodal input, or routed to the `analyze_image` tool (Google Gemini vision) otherwise. Text files are extracted as UTF-8 and included inline up to 8,000 characters.

## Proactive Channel Tools

The agent can send messages and take actions proactively:

| Tool | Actions |
|------|---------|
| `discord_send` | 17 actions: send, reply, react, edit, delete, pin, create_thread, send_embed, etc. |
| `slack_send` | 17 actions: send, reply, react, edit, delete, pin, set_topic, send_blocks, send_file (TTS voice via OGG/Opus) |
| `trello_send` | 22 actions: create_card, move_card, add_comment, add_checklist, search, etc. |

## Channel Voice Parity

All four messaging channels (Telegram, Discord, WhatsApp, Slack) now share a single code path via `crate::channels::voice::{transcribe, synthesize}`. Bot replies are recorded in the `channel_messages` table for conversation context, previously only user messages were stored.

## Typing Indicator Persistence (v0.3.76)

Typing indicators now **survive background tasks**. Previously, when the agent spawned a background task (long build, test suite), the typing indicator would drop because the foreground turn ended. Now the indicator stays alive until the final response is delivered, regardless of how many background tasks run in between.

## Smart Chunk Boundaries (v0.3.78)

When a response exceeds platform message limits and must be split into chunks, the splitter now **prefers boundaries that leave no markup open**. Previously, a chunk could end mid-bold (`**text`) or mid-code-fence, causing rendering glitches on the receiving platform. The splitter now scans for the nearest safe break point where all markdown delimiters are balanced.

## Owner-Gated Channel Commands (v0.3.80)

User-defined slash commands and skills are **owner-gated on the channel catch-all arm** (#975). In a group chat, only the bot owner can trigger custom commands and skills; other members' messages fall through to normal conversation. This closes a gap where any group member could run owner-defined automation through the catch-all routing path.
