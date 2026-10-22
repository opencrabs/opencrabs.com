# Discord

Connect OpenCrabs to Discord for server and DM interactions.

## Setup

### Step 1: Create a Discord Bot

1. Go to [discord.com/developers/applications](https://discord.com/developers/applications)
2. Create a new application
3. Go to **Bot** section, create a bot
4. **Enable MESSAGE CONTENT Intent** (required: under Privileged Gateway Intents)
5. Copy the bot token
6. Under **OAuth2 → URL Generator**, select `bot` scope with `Send Messages` and `Read Message History` permissions
7. Use the generated URL to invite the bot to your server

### Step 2: Configure via the Onboarding Wizard

Run `/onboard:channels` (or `/onboard` and navigate to the Channels step):

1. Use `↑`/`↓` to focus **Discord**
2. Press `Space` to toggle it on
3. Press `Enter` to open the Discord setup screen
4. Fill in the fields:
   - **Bot Token**: paste the token from the Developer Portal
   - **Channel ID**: the Discord channel to send the welcome message to (right-click a channel with Developer Mode on → Copy Channel ID)
   - **Allowed Users**: comma-separated Discord user IDs (since v0.5.1 an empty allowlist no longer allows everyone: until you set users, roles, or `bot_owner`, the channel denies all messages)
   - **Respond To**: `all`, `dm_only`, or `mention`
5. Press `Enter` on **Test Connection** to verify
6. Press `Enter` to save and return to the channel list

> Enable Developer Mode in Discord: Settings → Advanced → Developer Mode

### Manual Configuration (advanced)

```toml
# keys.toml
[channels.discord]
token = "your-bot-token"

# config.toml
[channels.discord]
enabled = true
allowed_channels = ["123456789"]
allowed_users = []
respond_to = "all"
```

## Configuration

All Discord options live under `[channels.discord]` in `~/.opencrabs/config.toml`:

```toml
[channels.discord]
enabled = true
token = "your-discord-bot-token"       # or store in keys.toml
allowed_users = ["123456789012345678"]  # Discord user IDs
allowed_channels = ["123456789012345678"]
respond_to = "mention"                  # "all", "dm_only", "mention" (default)
session_idle_hours = 24.0               # idle timeout for non-owner sessions
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Discord bot channel |
| `token` | `None` | Discord bot token from the Developer Portal |
| `allowed_users` | `[]` (denies unless roles/owner configured) | Discord user IDs. Accepts int or string arrays. Any one of `allowed_users`, `allowed_roles`, `bot_owner` configures the channel; a configured one admits allowlisted users, holders of an allowed role (never in DMs), or the owner |
| `allowed_channels` | `[]` (all channels) | Restrict bot to specific channel IDs |
| `respond_to` | `"mention"` | When to respond: `"all"`, `"dm_only"`, `"mention"` |
| `session_idle_hours` | `None` (no timeout) | Idle timeout for non-owner sessions. Owner sessions never expire |

## Features

- **Server channels and DMs**: Works in text channels and direct messages
- **Interactive components**: Select menus, modal forms, button interactions with component TTL and role access
- **Media gallery**: Batch generated files into one multi-attachment message
- **Grouped tool calls**: Consecutive tool calls collapse into one expandable block with Expand/Collapse toggle
- **Reactions**: Agent can add emoji reactions to messages. Emoji reactions from users trigger agent turns (parity with Slack/Telegram)
- **Forum threads**: Support for Discord forum channels
- **Image support**: Send and receive images
- **Embed suppression**: Agent wraps multiple links in `<>` to suppress embeds
- **Slash commands**: All built-in and custom commands work

### Interactive Components (v0.3.63)

Discord buttons, select menus, and modal forms power the provider picker, model picker, and session switcher. Components support:
- **Component TTL**: Components expire after a configurable timeout
- **Role access**: Restrict component interactions to specific Discord roles
- **Forum threads**: Agent can create and manage forum threads

### Media Gallery (v0.3.63)

When the agent generates multiple files (PDFs, spreadsheets, images), Discord batches them into one multi-attachment message instead of sending separate messages for each file.

### Grouped Tool Calls (v0.3.63)

Consecutive tool calls collapse into a single expandable block in Discord. Users can toggle between expanded and collapsed views. This keeps the channel clean during multi-step operations.

### Reactions (v0.3.63)

The agent can add emoji reactions to messages. When a user reacts with an emoji, it triggers an agent turn (parity with Slack and Telegram). This enables quick, lightweight interactions without typing full messages.

## Formatting Notes

- Markdown tables in agent output convert to native Discord tables (v0.5.2); in your own messages, bullet lists are still the safest habit
- Wrap multiple links in `<url>` to suppress embeds

## Live Reply Tracing, Auto-Threading, Native Tables (v0.5.2)

Long turns stream as a live reply that is traced and edited in place instead of staying silent until the end. Answers that belong together are auto-threaded, and markdown tables in agent output render as native Discord tables rather than raw pipes (#1608). A dedup guard on tool turns stops the final response from double-posting content a tool summary already showed (#1603).

## Native Polls, Voice Replies, Slash Projection, Flow Groups (v0.5.5)

`send_poll` posts a native Discord poll with each option as its own answer button, so follow-up option sets render as the platform's own poll UI (#1848). TTS replies are delivered as Discord voice messages instead of attachment links (#1849), and your `commands.toml` is projected onto native slash commands, so your custom commands appear in Discord's command picker (#1850).

Flow groups now open at turn start with a live clock that keeps running while the agent thinks, and the activity text leads the line instead of hiding behind the tool count (#1841, #1843, #1844, #1845). Every outbound write, including the persistent plan card, rides one shared write governor that parks cleanly on 429s (#1910, #1912).

Per-channel control arrived with `respond_to` and `open` modes in the channel config, enforced in the message filter, the ACL and the command gate (#2014). Two commands set them from inside the channel itself: `/respond_to` picks who the crab answers there, and `/cowork` opens the channel or thread to its members (#2014, #2015). Cron reports on Discord arrive as one edited forum post instead of a chain of messages (#1851).

## Sustained Typing Indicator (v0.3.76)

Discord now shows a **sustained typing indicator** while the agent processes a request. Previously, Discord had no typing indicator at all (Telegram and Slack did). The indicator stays alive through background tasks and long tool executions, so users see "OpenCrabs is typing..." for the entire processing duration, not just the first few seconds.
