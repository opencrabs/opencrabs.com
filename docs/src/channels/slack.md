# Slack

Connect OpenCrabs to Slack workspaces.

## Setup

### Step 1: Create a Slack App

1. Go to [api.slack.com/apps](https://api.slack.com/apps)
2. Create a new app (From Scratch)
3. Enable **Socket Mode** under Settings
4. Generate an **App-Level Token** (Settings → Basic Information → App-Level Tokens) with `connections:write` scope
5. Under **OAuth & Permissions**, add bot scopes: `chat:write`, `channels:history`, `groups:history`, `im:history`, `reactions:write`
6. Install the app to your workspace (**Install App** → **Install to Workspace**)
7. Copy both tokens: they live in **different places** in the Slack dashboard:
   - **Bot Token** (`xoxb-...`): under **Install App**, in the *OAuth Tokens for Your Workspace* section. It only appears **after** you install the app to the workspace.
   - **App-Level Token** (`xapp-...`): under **Basic Information** → **App-Level Tokens**. If you already generated it in step 4, Slack only lists the token **name** here; **click the name** to expand it and reveal the actual `xapp-...` value to copy.

### Step 2: Configure via the Onboarding Wizard

Run `/onboard:channels` (or `/onboard` and navigate to the Channels step):

1. Use `↑`/`↓` to focus **Slack**
2. Press `Space` to toggle it on
3. Press `Enter` to open the Slack setup screen
4. Fill in the fields:
   - **Bot Token**: the `xoxb-...` token
   - **App Token**: the `xapp-...` token
   - **Channel ID**: right-click a channel → View channel details → copy the Channel ID at the bottom
   - **Allowed Users**: comma-separated Slack user IDs (Profile → ⋯ → Copy member ID)
   - **Respond To**: `all`, `dm_only`, or `mention`
5. Press `Enter` on **Test Connection** to verify
6. Press `Enter` to save

### Manual Configuration (advanced)

```toml
# keys.toml
[channels.slack]
bot_token = "xoxb-..."
app_token = "xapp-..."

# config.toml
[channels.slack]
enabled = true
allowed_channels = ["C12345678"]
allowed_users = []
respond_to = "all"
```

## Configuration

All Slack options live under `[channels.slack]` in `~/.opencrabs/config.toml`:

```toml
[channels.slack]
enabled = true
token = "xoxb-your-bot-token"          # or store in keys.toml
app_token = "xapp-your-app-token"      # Socket Mode token
allowed_users = ["U12345678"]           # Slack user IDs
allowed_channels = ["C12345678"]
respond_to = "mention"                  # "all", "dm_only", "mention" (default)
session_idle_hours = 24.0               # idle timeout for non-owner sessions
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Slack bot channel |
| `token` | `None` | Bot token (`xoxb-...`) |
| `app_token` | `None` | App-level token for Socket Mode (`xapp-...`) |
| `allowed_users` | `[]` (unconfigured: denies everyone) | Slack user IDs (`U12345678`). Since v0.5.1 a workspace with no allowlist and no `bot_owner` denies all messages; once configured it admits allowlisted users or the owner |
| `allowed_channels` | `[]` (all channels) | Restrict bot to specific channel IDs |
| `respond_to` | `"mention"` | When to respond: `"all"`, `"dm_only"`, `"mention"` |
| `session_idle_hours` | `None` (no timeout) | Idle timeout for non-owner sessions. Owner sessions never expire |

## Features

- **Channels and DMs**: Works in public/private channels and direct messages
- **Action buttons**: Provider picker, model picker, session switcher use Slack action buttons
- **Thread support**: Responds in threads when appropriate
- **Slash commands**: All built-in and custom commands work
- **Reactions**: Agent can add emoji reactions. Emoji reactions from users trigger agent turns (react-back)
- **Grouped tool calls**: Consecutive tool calls collapse into one edited-in-place message with Expand/Collapse toggle
- **TTS voice replies**: Voice responses sent as OGG/Opus files via `files.upload` with inline waveform UI

### Grouped Tool Calls (v0.3.62)

Consecutive tool calls collapse into a single message in Slack. The message is edited in-place as each tool executes. Users can toggle between expanded and collapsed views with an Expand/Collapse button. This keeps channels clean during multi-step operations.

### Reactions (v0.3.62)

The agent can add emoji reactions to messages. When a user reacts with an emoji, it triggers an agent turn (react-back). This enables quick, lightweight interactions without typing full messages. Parity with Telegram and Discord.

## Reactions (v0.3.62)

Slack supports emoji reactions for lightweight acknowledgements:

- **React-back**: When you react to a bot message, the bot acknowledges it (e.g., adds a 👍 to show it saw your reaction)
- **Reaction turns**: Certain reactions can trigger agent actions (configurable)
- **Grouped tool calls**: Multiple tool calls in one turn collapse into a single collapsible message, keeping the channel clean

## Native Rendering (v0.3.80)

Slack messages render in Slack's own shape instead of raw markdown (#1016):

- **Tables** are converted to Slack-native layout instead of arriving as pipe characters
- **Headings** are rendered as Slack formatting rather than `#` hashes

Step-group narration is folded on the character stream rather than on paragraphs, and the final message no longer repeats the step-group narration it already sent (#1010).

## Socket Mode

Slack uses Socket Mode (WebSocket) instead of HTTP webhooks, no public URL or ngrok needed. The connection is outbound from your machine.
