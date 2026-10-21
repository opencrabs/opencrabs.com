# Telegram

Connect OpenCrabs to Telegram for DMs and group chats.

## Setup

### Step 1: Create a Bot with BotFather

1. Message [@BotFather](https://t.me/BotFather) on Telegram
2. Send `/newbot` and follow the prompts
3. Copy the bot token (format: `123456:ABC-DEF...`)

### Step 2: Configure via the Onboarding Wizard

Run `/onboard:channels` (or `/onboard` and navigate to the Channels step):

1. Use `↑`/`↓` to focus **Telegram**
2. Press `Space` to toggle it on
3. Press `Enter` to open the Telegram setup screen
4. Fill in the fields:
   - **Bot Token** — paste the token from BotFather
   - **Owner User ID** — your numeric Telegram chat ID
   - **Respond To** — `all`, `dm_only`, or `mention` (when to respond in groups)
5. Press `Enter` on **Test Connection** to verify the bot works
6. Press `Enter` to save and return to the channel list

> Get your chat ID by messaging [@userinfobot](https://t.me/userinfobot) on Telegram.

### Manual Configuration (advanced)

If you prefer editing files directly, the wizard writes to `~/.opencrabs/keys.toml` and `~/.opencrabs/config.toml`:

```toml
# keys.toml
[channels.telegram]
bot_token = "123456:ABC..."

# config.toml
[channels.telegram]
enabled = true
allowed_users = ["123456789"]
respond_to = "all"
```

## Features

- **DMs and groups** — Works in private chats and group conversations
- **Forum topic routing** (v0.3.31) — In supergroups with topics enabled, the bot tracks `thread_id` through the full pipeline. Use `list_topics` action to map topic names (e.g. `#announcements`) to numeric IDs, then pass `thread_id` to `send` / `reply` / `send_photo` to route into a specific topic
- **Context-aware pre-tool status** (v0.3.31) — While a tool runs, the bot shows a live status message naming the tool, elapsed time, and either a reasoning excerpt or an anchored phrase from the user's request
- **Inline ctx budget footer** (v0.3.36) — context budget footer (`ctx: XK/YK Z% | N tok/s`) is now appended to the last response message instead of sent as a separate message. Keeps the chat clean.

## v0.3.76–v0.3.78 Improvements

### /new Gating (v0.3.76)
The `/new` command is now **gated to the bot owner** in group chats. Non-owners cannot create new sessions or reset context. The command menu is also scoped: owners see the full menu, non-owners see only the commands available to them.

### HTML Conversion (v0.3.77)
All `telegram_send` actions now apply **HTML-to-Telegram conversion** consistently. Previously, only the main response path converted HTML entities; proactive sends, follow-ups, and media captions could arrive with raw `<b>` tags or unescaped ampersands.

### Rich Delivery (v0.3.78)
Telegram delivery is unified on **markdown formatting**. The legacy "blocks path" (which sent structured JSON blocks) has been removed in favor of a single rich-markdown renderer. This eliminates formatting inconsistencies between streaming responses and proactive sends.

### Callback Routing (v0.3.78)
Inline keyboard callbacks (plan approvals, follow-up taps, dedup approvals) now route to the **originating session** instead of the chat-bound session. Previously, tapping a button in a group chat could route the callback to the wrong session if multiple sessions shared the same chat.

### Background Streaming (v0.3.78)
Background task results now stream through **one turn per session** instead of spawning a new message per tick. This eliminates the message flood when a background task (build, test suite) produces incremental output.

### Follow-Up Attribution (v0.3.78)
When a user taps a follow-up suggestion button in a group chat, the agent now **names the member** who tapped it. Previously, follow-up taps were anonymous in group contexts.

### Other Fixes
- `/start` recognizes existing members of closed groups (v0.3.76)
- Plan-card serialisation prevents duplicate cards on concurrent refreshes (v0.3.76)
- Follow-up taps run a real tools turn instead of echoing text (v0.3.76)
- Flow block no longer posts a new message every tick (v0.3.77)
- Echoed plan titles dropped from responses (v0.3.77)
- Open-group registration gated on persisted open flag (v0.3.78)
- Cowork members recorded on first message (v0.3.78)
- **Rolling status edit-in-place** (v0.3.36) — tool status messages (⚙️ running, ✅ done) are edited in-place instead of delete+recreate, preventing flicker and preserving scroll position.
- **Bot command hot-reload** (v0.3.36) — bot commands refresh automatically when config or skills change, without restarting the bot.
- **Guard tok/s against burst-delivery** (v0.3.36) — tok/s footer is guarded against burst-delivery artifacts so the number stays stable.
- **`follow_up_question` polish** (v0.3.34) — Telegram keyboard is now single-column with a 40-character label cap (longer options rejected with a clear error). The rolling "Running follow_up_question (16s)" status is suppressed while the keyboard is pending so buttons don't get visually buried. The LLM is instructed to call the tool silently without echoing the question text in surrounding prose
- **Inline buttons** — Provider picker, model picker, session switcher use Telegram inline keyboards
- **Image support** — Send images to the bot, receive generated images
- **Voice messages** — STT transcription + TTS response
- **All slash commands** — `/help`, `/models`, `/new`, `/sessions`, custom commands
- **Owner vs non-owner** — Owner uses the shared TUI session, non-owners get per-user sessions
- **Onboarding overhaul** (v0.3.30) — Auto-detects owner user ID from `getUpdates`, persists partial config on cancel, only Enter on the last step commits (Tab no longer silently rewrites ~30 config keys)
- **Teloxide upgrade + join detection** (v0.3.35) — Upgraded from teloxide 0.13 to 0.17. New members joining a group are now detected before the allowlist check, so the bot can greet or moderate join events. Marathon-bucket rolling status rotates through project-author quip pool for more varied status messages.

## v0.3.81 Improvements

### Mermaid Diagrams as Images (#1044)
Mermaid code fences in rich messages render as **images** via the markdown+media route, while **tables stay native** Telegram formatting instead of being rasterized into the image.

### Delivery and Rate-Limit Hardening
- **Rich blocks decode as a typed AST** (#1058) instead of ad-hoc string surgery, so structured messages can't arrive mangled
- **Inline rate-limit waits capped at 30s** (#1064): a Telegram 429 never parks a turn indefinitely
- **Flow footer + section cap** (#1052, #1053, #1054, #1056): flow logs get a footer and bounded sections instead of unbounded message growth

### Group Safety
- **Only the owner may add the bot to a group** (#1042)
- **Bot-add detection** (#1041): being added by a member is told apart from another bot arriving
- **Unauthorised chats are never recorded** (#1043): chats that were never approved leave no session residue

## v0.3.82 Improvements

### Long Rate-Limit Windows Bail Immediately (#1110)
A Telegram 429 that asks for a long wait no longer sleeps inline. The send returns immediately and the work is rescheduled, instead of the turn parking on a multi-minute (or multi-hour) window. This closes the failure mode where one throttled send held a turn open long enough to look like a hang.

### Delivery Correctness
- **Rich sends stop building a double-slash URL** (#1117): a trailing slash on the API base no longer produces `//bot<token>`
- **Rich API calls honour `set_api_url`** (#1088): the rich path used a hardcoded `api.telegram.org`, so it bypassed a redirected Bot. It now routes through the caller's API URL like every other method, which also makes the path testable without a real network call
- **All six send arms route through thread resolution**: forum-topic targeting is applied consistently instead of on some arms only

### Observability
- **Send-correlation telemetry** (#1085): the three send chokepoints, every `telegram_send` arm, and the final streaming edit-in-place are all logged, so a message that never arrived can be traced to the arm that dropped it

## Configuration

All Telegram options live under `[channels.telegram]` in `~/.opencrabs/config.toml`:

```toml
[channels.telegram]
enabled = true
token = "123456:ABC-DEF..."          # or store in keys.toml
allowed_users = ["123456789"]         # numeric Telegram user IDs
allowed_channels = ["-100123456"]     # restrict to specific group/channel IDs (empty = all)
respond_to = "mention"                # "all", "dm_only", "mention" (default)
session_idle_hours = 24.0             # idle timeout for non-owner sessions
rich_messages = false                 # native Telegram rich messages (Bot API 10.1)
silence_group_start = true            # silently ignore /start from non-allowed users in groups
bot_owner = ["123456789"]             # owner IDs (gated commands, /cd hidden dirs, /profiles)
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Telegram bot channel |
| `token` | `None` | Telegram Bot API token from @BotFather |
| `allowed_users` | `[]` (accept all) | Numeric Telegram user IDs. Accepts int or string arrays. Empty = open mode |
| `allowed_channels` | `[]` (all channels) | Restrict bot to specific channel/group IDs. DMs always pass |
| `respond_to` | `"mention"` | When to respond in groups: `"all"` = every message, `"dm_only"` = ignore groups, `"mention"` = only when @mentioned or replied-to |
| `session_idle_hours` | `None` (no timeout) | Idle timeout in hours for non-owner sessions. Owner sessions never expire |
| `rich_messages` | `true` (since v0.3.64) | Send structured replies as native Telegram rich messages (tables, headings, lists, math). Requires current mobile/desktop Telegram clients. Telegram Web and older clients show a "not supported" placeholder. Toggle via the onboarding checkbox or config |
| `silence_group_start` | `true` | Silently ignore /start from non-allowed users in group chats. Users who need their ID can DM the bot |
| `bot_owner` | `[]` (first allowed_user) | Bot owner user IDs. Owners can access gated commands (/profiles, hidden files in /cd), manage profiles. Defaults to first entry in `allowed_users` |
| `guest_queries` | `false` | Answer Telegram Guest Mode tags: someone tags the bot in a chat it is not a member of and gets one reply there. Callers must be the owner, an admin, or listed for that chat. See [Guest queries](#guest-queries) |

## Per-group access control (per-chat ACL)

Telegram groups can have their own member list, so a user can be allowed in one group without gaining DM access:

- `allowed_users` (channel level) — **admins**: may DM the bot and act in any chat.
- `bot_owner` — the **owner**: always allowed everywhere.
- `[channels.telegram.groups.<chat_id>].allowed_users` — allowed in that group only. These users are refused in DMs unless they are also an admin or the owner, which closes the "DM the bot privately to escape group oversight" bypass.

DMs are gated to admins + owner. If neither `allowed_users` nor `bot_owner` is set, the bot is unconfigured and stays open (no hard lockout); set either one to lock it down.

Each group can also override `respond_to` just for itself.

```toml
[channels.telegram]
allowed_users = ["111"]                  # admins: DM + any chat
respond_to = "mention"                   # global default

[channels.telegram.groups.-1001234567890]
allowed_users = ["222", "333"]           # allowed in this group only, never via DM
respond_to = "all"                       # per-group override of the global respond_to
```

`respond_to` accepts `all`, `mention`, `dm_only`, or `auto` (reply to all while there is at most one active sender, then switch to mention-only once a second unique sender appears).

### Per-group open mode (v0.3.66)

Set `open = true` on a trusted group to serve all members without individual allowlisting:

```toml
[channels.telegram.groups.-1001234567890]
open = true                              # all members can talk, no per-user registration needed
```

Use this for public/community groups where you want the bot available to everyone. Members still can't DM the bot unless they're in the global `allowed_users` or `bot_owner`.

## Guest queries

Telegram Guest Mode (Bot API 10.0) lets someone tag `@yourbot` in a chat the bot is **not** a member of. With `guest_queries = true`, OpenCrabs answers the tag right there using the API method `answerGuestQuery`.

```toml
[channels.telegram]
guest_queries = true                     # default false: guest tags are ignored
allowed_users = ["111"]                  # admins: answered anywhere
[channels.telegram.groups.-1001234567890]
allowed_users = ["222"]                  # this user is answered in this chat only
```

Default is off, and the gate is narrow once it is on. The caller must be the `bot_owner`, an admin (`allowed_users`), or listed for **that** chat. A group's `open = true` grants nothing here: it opens that group to the bot, it does not authorize answering guest tags from its members. Refused queries are logged with the `guest_query_id`, the caller, and the chat.

Boundaries of a guest turn:

- The bot sees only the tagged message, no history from that chat, and nothing from the guest turn is stored in that chat's history.
- A guest turn runs the same tool surface as a normal turn, with one exception: tools that need approval are denied, because approvals are buttons in a chat and the caller is not the machine's owner. The tool set is capped at what a stranger should reach (#2134).
- The live status block and rich rendering match a normal turn: the guest sees the same progress chrome and the same markdown or rich reply, and an edit replaces the placeholder rather than appending a second message (#2109, #2133, #2135, #2138).
- One guest turn per session at a time, cancellable, behind a rate limit, so a tag storm cannot fan out into parallel turns (#2143).
- A reply with no tool calls is the answer alone, and the answer is never carried twice in one reply (#2113).
- Channel directives (`<<...>>`) are stripped, so a local file path can never leak into someone else's chat.
- An overlong reply is cut and marked `[truncated]` rather than dropped.

Because the pinned `teloxide-core` has no guest update variant, guest queries are read on the raw `getUpdates` layer, and `guest_message` is in the bot's `allowed_updates` list. Nothing to configure for that; it is on whenever the channel runs.

## Voice and file pickup in groups

In mention-only groups (`respond_to = "mention"`), users can share files and voice messages even when the bot isn't directly tagged in the same message:

1. **Fire-and-forget file capture** — The bot downloads ALL incoming voice, video, document, and audio files from group messages to `~/.opencrabs/tmp/`, regardless of whether the bot was mentioned. This happens silently in the background.
2. **Tag-then-ask** — A user sends a voice message, then tags the bot in a follow-up message (e.g. `@bot what did I just say?`). The bot scans the tmp directory for recent voice files from that chat (5-minute window), transcribes the most recent one, and prepends the transcript to the user's message.

This solves the core UX problem in mention-only groups: previously, tagging the bot in the same message as a voice note didn't work because Telegram sends voice and text as separate messages.

The agent can use `telegram_send` with 20+ actions. The `thread_id` field on `send` / `reply` / `send_photo` targets a specific forum topic in supergroups with topics enabled.

| Action | Description |
|--------|-------------|
| `send` | Send text message (with optional `thread_id` for forum topics) |
| `reply` | Reply to a message |
| `send_photo` | Send image file (supports `caption` and `reply_parameters` since v0.3.58) |
| `send_document` | Send document (supports `caption` and `reply_parameters` since v0.3.58) |
| `send_voice` | Send voice message |
| `list_topics` | Returns `(thread_id, topic_name)` pairs the bot has observed — translate `#announcements` into a numeric `thread_id` |
| `pin` / `unpin` | Pin or unpin a message |
| `set_reaction` | Add an emoji reaction |
| And more... | |

## Reactions (v0.3.58)

The bot understands Telegram emoji reactions in both directions:

- **Inbound** — when a user reacts to one of the bot's messages with an emoji, the bot picks up that reaction (it looks up the original bot message by its platform message ID) and can act on it as feedback.
- **Frame reactions (v0.3.61)** — inbound reactions are read by sentiment and the bot addresses the user by first name in its response.
- **Mid-turn reactions (v0.3.61)** — if a user reacts during a running turn, the reaction injects into the current loop instead of spawning a second turn.
- **Reaction-only replies** — when a short acknowledgement says it all (a thumbs-up, a 👀, a 😂), the bot can respond with just a reaction instead of a full text message, keeping the chat uncluttered.
- **Emoji validation** — only real emoji count as reaction directives; code spans and stray characters are ignored, and react directives are stripped from intermediate status messages so they never leak into the visible reply.

Use the `set_reaction` action on `telegram_send` to add a reaction to a specific message from the agent.

## /cowork — Workspace Creation (v0.3.59)

`/cowork` creates a team workspace directly from Telegram. Run it in your DM with the bot:

1. Send `/cowork` in your DM with the bot
2. The bot replies with an invite link and QR code for a new group
3. Scan the QR code or tap the link to create the group
4. Add your friends or teammates to the group
5. Members auto-register to the **group's allow list** (`[channels.telegram.groups.<chat_id>].allowed_users`), not the global one. Both new joiners (via invite link) and existing members (on their first message) get registered. Cowork members can talk in the group but cannot DM the bot privately unless also on the global `allowed_users`. The owner gets a confirmation for each registration.

`/cowork` works from any surface. In Telegram DMs, the native flow activates directly. From the TUI, Discord, Slack, or WhatsApp, the agent calls the `cowork_connect` tool which mints a session, registers it with the bot, and returns the `t.me` deep link plus a scannable QR code PNG.

## Flow Logs (v0.3.63)

Telegram's rich API supports 32K-character processing-log blocks. When the agent runs multi-step operations (tool calls, research, code generation), it streams progress into a single flow-log message that grows as the turn progresses. For shorter messages, HTML formatting falls back to plain text.

Flow logs show:
- Tool calls with parameters and results
- Intermediate reasoning steps
- File edits and git operations
- Build/test progress
- **Wall-clock duration** (v0.3.66) — finished/failed/timeout states carry the total turn time
- **Bash comments as flow status** (v0.3.66) — line-start comments in a bash command surface as live flow status
- **Full status preview** (v0.3.66) — the status preview uses the whole last human-readable line, no truncation

The flow-log message is edited in-place as each step completes, so the channel stays clean. Users see real-time progress without message spam.

Flow blocks **re-stick to the chat bottom** when buried (v0.3.65), so you always see the latest progress without scrolling.

## Forum Topic Management and /clear (v0.5.2)

The agent can create and rename forum topics (not just route into existing ones), and a group's session can bind to a topic so each topic keeps its own conversation context. `/clear` starts a fresh session in place — the current one closes and a blank one begins — without invoking the summariser, so it is instant and free (#1585).

## Group Chat Behavior

In groups, the agent:
- Responds when mentioned by name or replied to
- Stays quiet when the conversation doesn't involve it
- Tracks context from group messages passively
