# Trello

OpenCrabs integrates with Trello for board and card management via the `trello_send` tool.

## Setup

### Step 1: Get Trello Credentials

1. Go to [trello.com/power-ups/admin](https://trello.com/power-ups/admin)
2. Create a new Power-Up to get your **API Key**
3. Click the "Token" link next to your API key to generate an **API Token**

### Step 2: Configure via the Onboarding Wizard

Run `/onboard:channels` (or `/onboard` and navigate to the Channels step):

1. Use `↑`/`↓` to focus **Trello**
2. Press `Space` to toggle it on
3. Press `Enter` to open the Trello setup screen
4. Fill in the fields:
   - **API Key**: from the Trello Power-Up admin page
   - **API Token**: generated alongside the API key
   - **Board ID**: board name or 24-character hex ID (names are resolved automatically)
   - **Allowed Users**: Trello member IDs allowed to interact with the bot (leave empty for all members)
5. Press `Enter` on **Test Connection** to verify board access
6. Press `Enter` to save

### Manual Configuration (advanced)

```toml
# keys.toml
[channels.trello]
api_key = "your-api-key"
token = "your-token"

# config.toml
[channels.trello]
enabled = true
boards = ["Board Name or ID"]
allowed_users = []
# poll_interval_secs = 30  # Poll for new card comments
```

## Configuration

All Trello options live under `[channels.trello]` in `~/.opencrabs/config.toml`:

```toml
[channels.trello]
enabled = true
token = "your-trello-api-token"        # or store in keys.toml
app_token = "your-trello-api-key"      # stored as app_token for keys.toml symmetry
allowed_users = ["memberId1"]           # Trello member IDs
board_ids = ["boardId1", "boardId2"]   # boards to monitor (also accepts allowed_channels)
poll_interval_secs = 60                 # polling interval (absent or 0 = tool-only mode)
session_idle_hours = 24.0               # idle timeout for non-owner sessions
```

| Field | Default | Description |
|-------|---------|-------------|
| `enabled` | `false` | Enable the Trello channel |
| `token` | `None` | Trello API token |
| `app_token` | `None` | Trello API key (stored as `app_token` for keys.toml symmetry) |
| `allowed_users` | `[]` (accept all) | Trello member IDs |
| `board_ids` | `[]` (all boards) | Board IDs to monitor for @mentions. Also accepts `allowed_channels` as alias |
| `poll_interval_secs` | `None` (tool-only) | Polling interval in seconds. Absent or 0 = no polling (tool-only mode) |
| `session_idle_hours` | `None` (no timeout) | Idle timeout for non-owner sessions. Owner sessions never expire |

## Tool Actions

The `trello_send` tool supports 22 actions:

| Action | Description |
|--------|-------------|
| `create_card` | Create a new card |
| `get_card` | Get card details |
| `update_card` | Update card fields |
| `move_card` | Move card to another list |
| `archive_card` | Archive a card |
| `find_cards` | Search for cards |
| `add_comment` | Add a comment to a card |
| `get_card_comments` | Read card comments |
| `add_checklist` | Add a checklist to a card |
| `add_checklist_item` | Add an item to a checklist |
| `complete_checklist_item` | Mark checklist item done |
| `add_label_to_card` | Add a label |
| `remove_label_from_card` | Remove a label |
| `add_member_to_card` | Assign a member |
| `remove_member_from_card` | Unassign a member |
| `add_attachment` | Attach a file or URL |
| `list_boards` | List accessible boards |
| `list_lists` | List columns in a board |
| `get_board_members` | Get board members |
| `search` | Search across boards |
| `get_notifications` | Get notifications |
| `mark_notifications_read` | Mark notifications read |

## Behavior

- **Tool-only by default**: The agent acts on Trello only when explicitly asked
- **Optional polling**: Set `poll_interval_secs` to enable monitoring for `@bot_username` mentions
- **Image attachments**: Generated images are sent as card attachments with embedded previews
- **File attachments**: Card attachments (images, documents) are fetched and processed through the vision pipeline
