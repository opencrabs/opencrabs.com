# Onboarding

A first run does **not** open the wizard. OpenCrabs goes straight to the chat, with its built-in provider already provisioned and the agent greeting you. The **onboarding wizard** is opt-in: run `opencrabs --onboard`, or `/onboard` from inside a session.

Video walkthroughs are on the [Onboarding Videos](./videos.md) page.

## First Run: You Land In The Chat

There is no wizard on a fresh install. OpenCrabs opens the chat, brings the built-in provider online and says hello:

1. `config.toml` and `keys.toml` are written from the shipped example if they are not there yet.
2. `[providers.opencrabs]` is turned on. The example ships it disabled, so this is what makes it the active provider.
3. The install asks `inference.opencrabs.com` for its own token and saves it in `keys.toml`. The token is reused on every later launch, so this happens once.
4. The agent reads its brain files and greets you with the model it is running on.

If the token request fails (offline, proxy unreachable), you still land in a working chat with a line saying what happened. Type `/onboard` to pick another provider, or fix the connection and restart. Onboarding is only marked complete once the built-in provider actually came up, so the next launch retries instead of leaving you on a chat with no provider.

### What You Get Without Any Setup

| Capability | Model | Notes |
|---|---|---|
| Chat | `deepseek-v4.1-flash` | 200k context, fixed |
| Vision | `deepseek-v4.1-flash` | Attach an image and it works |
| Image generation | `gemini-2.5-flash-image` | Served on the image route only, never as a chat model |
| Voice in | `qwen3.8-omni-flash` | A voice note is transcribed and answered in text |
| Voice out | not served | Needs another TTS engine, see [Voice](../features/voice.md) |

Free, no account, no key to paste, no usage caps during the experimental phase. The proxy rate limits abuse (request floods, bulk install minting) and normal use does not hit it. Requests go to `inference.opencrabs.com`, which forwards them to the upstream model host. Full detail, including where the token lives and how to leave: [OpenCrabs (built in)](../brain/providers.md#opencrabs-built-in).

To move to another provider later, type `/models` and pick one, or run `/onboard` and choose Advanced.

## Quick Start (the wizard, 1 minute)

The wizard is opt-in: `opencrabs --onboard` before the first launch, or `/onboard` inside a session. Pick **QuickStart** to keep the defaults and skip channel and voice setup.

| Step | Action |
|------|--------|
| 1. Mode | Arrow to **QuickStart**, hit `Enter` |
| 2. Workspace | Hit `Enter` to accept the default path |
| 3. Provider | Arrow to the provider you want, hit `Enter`. **OpenCrabs (built in)** needs no key; anything else asks for one |
| 4. API Key | Only for a provider that needs one. Paste it (`Cmd+V` / `Ctrl+V`), hit `Enter`. The model list loads live |
| 5. Model | Arrow to your model, hit `Enter` |
| 6. Daemon | Arrow to select whether to run as a background daemon, hit `Enter` |
| 7. Vibe Check | All checks should show ✅. Hit `Enter` |
| 8. About You | Write something about yourself. The more the agent knows, the better. Hit `Enter` |
| 9. About Agent | Write something about the agent's personality. Hit `Enter` |
| 10. Chat | You're in. The agent greets you and asks only what is still missing |


## Advanced Setup (7 minutes)

Full setup with Telegram, local voice, vision, and image generation.

| Step | Action |
|------|--------|
| 1. Mode | Arrow to **Advanced**, hit `Enter` |
| 2. Workspace | Hit `Enter` to accept the default path |
| 3. Provider | Arrow to your provider (e.g. z.ai), hit `Enter`. Arrow down to select a plan, hit `Enter` |
| 4. API Key | Paste your key, hit `Enter`. Model list loads live |
| 5. Model | Arrow to your model, hit `Enter` |
| 6. Channels | Arrow to **Other**, hit `Space` to select Telegram, hit `Enter`. Paste your bot token, follow the instructions to get your chat ID, hit `Enter`. Select mention mode, hit `Enter`. Once it says **Connected**, hit `Enter` again. Arrow down to **Continue**, hit `Enter` |
| 7. STT | Select **Local**, hit `Enter`. Pick model size (e.g. tiny for speed), hit `Enter` |
| 8. TTS | Select **Local** again, hit `Enter`. Pick a voice (e.g. Ryan), hit `Enter`. Wait for the model download, arrow down to **Continue**, hit `Enter` |
| 9. Image | Hit `Space` to select Vision and Image Generation, hit `Enter`. Paste your Gemini API key, hit `Enter` |
| 10. Daemon | Arrow to select whether to run as background daemon, hit `Enter` |
| 11. Vibe Check | All checks should show ✅. Hit `Enter` |
| 12. About You | Write something about yourself, the more the agent knows, the better. Hit `Enter` |
| 13. About Agent | Write something about the agent's personality. Hit `Enter` |
| 14. Chat | You're in. Start talking to your agent |

## Onboarding Flow

The wizard is a keyboard-driven TUI with 8 steps. Navigate with arrow keys, `Tab` to advance, `Esc` to go back.

| Step | Screen | What you do |
|------|--------|-------------|
| 1 | **Mode Select** | Choose QuickStart (skip channels) or Advanced |
| 2 | **Workspace** | Pick a working directory for file operations |
| 3 | **Provider & Auth** | Select provider → paste API key → pick model (fetched live) |
| 4 | **Channels** | Space to toggle channels on/off → Enter on each to configure |
| 5 | **Voice** | STT provider (Groq, local Whisper, or off) + TTS voice |
| 6 | **Image** | Vision toggle + generation model + API key |
| 7 | **Daemon** | Install background daemon (optional) |
| 8 | **Brain Setup** | Auto-generate SOUL.md from your profile |

### Channel Setup (Step 4)

The channels screen lists 5 integrations: Telegram, Discord, WhatsApp, Slack, Trello.

- `Space` toggles a channel on/off
- `Enter` on an enabled channel opens its setup screen (token, IDs, allowlists)
- `Enter` on **Continue** or `Tab` skips to the next step
- Each channel setup screen has a **Test Connection** button

See [Channels Overview](../channels/overview.md) for the full navigation guide.

### Re-running Setup

You can jump to any step without re-running the full wizard:

| Command | Step |
|---------|------|
| `/onboard` | Full wizard |
| `/onboard:provider` | Provider & model selection |
| `/onboard:channels` | Channel picker |
| `/onboard:voice` | Voice setup |
| `/onboard:image` | Image setup |
| `/onboard:brain` | Brain file generation |

After onboarding, your agent boots up and introduces itself. It reads its brain files (`SOUL.md`, `AGENTS.md`, `TOOLS.md`) and starts a conversation.

## Bootstrap

On the very first run, the agent goes through a **bootstrap** phase:

- Gets to know you (name, preferences, work style)
- Establishes its identity (name, personality, emoji)
- Opens `SOUL.md` together to discuss values
- Sets up `USER.md` with your profile

The bootstrap file (`BOOTSTRAP.md`) deletes itself when complete.

## Migrating From Another Tool

Already using ClaudeCode, OpenClaw, Hermes, or any other AI agent harness? Your agent can migrate your existing data (memory, skills, custom commands, preferences) into its own brain files using natural language. No manual file shuffling needed.

This works with **any** agent or coding harness that stores config locally. The migration searches your filesystem for the other tool's config directories, reads their contents, and maps them into OpenCrabs' own brain file format (`SOUL.md`, `USER.md`, `TOOLS.md`, `MEMORY.md`, `AGENTS.md`, `CODE.md`).

### Hand-Held Migration

If you want to review what gets migrated before it happens:

> Search for my ClaudeCode/OpenClaw/Hermes data locally (or any other agent harness I was using) and audit a migration to our own brain files, report back once its done, execute when I confirm and approve.

The agent will:
1. Search your filesystem for the other tool's config directories
2. Read and parse all relevant files (memory, commands, skills, preferences)
3. Produce an audit showing exactly what maps where
4. Wait for your confirmation
5. Execute the migration and report a full breakdown

### Autonomous Migration

If you just want it done:

> Search for my existing agent/harness data locally and migrate anything to our own brain files, no need my approval, just go, plan and execute, report back once its done with a full breakdown.

The agent skips the audit step, plans the migration internally, executes it, and gives you a summary of everything that moved.

### What Gets Mapped

| Source | Destination | Example |
|--------|-------------|---------|
| Memory / CLAUDE.md / context files | `MEMORY.md` | Project decisions, past context |
| Custom commands | `commands.toml` | Slash commands and their definitions |
| Skills / agent instructions | `AGENTS.md`, `TOOLS.md` | Workflow rules, tool configs |
| User preferences / profile | `USER.md` | Name, timezone, coding style |
| Coding standards / linting rules | `CODE.md` | Style guides, conventions |

> **Tip:** Run migration right after onboarding while the context is fresh. The agent already has your `USER.md` and `SOUL.md` from the wizard, so it can merge intelligently instead of overwriting.

## Key Commands

| Command | Description |
|---------|-------------|
| `/help` | Show all available commands |
| `/models` | Switch provider or model |
| `/new` | Create a new session |
| `/sessions` | Switch between sessions |
| `/cd` | Change working directory |
| `/compact` | Manually compact context |
| `/evolve` | Download latest version |
| `/rebuild` | Build from source |
| `/approve` | Set approval policy |

## Approval Modes

Control how much autonomy the agent has:

| Mode | Behavior |
|------|----------|
| `/approve` | Ask before every tool use (default) |
| `/approve auto` | Auto-approve for this session |
| `/approve yolo` | Auto-approve always (persists) |

## Working Directory

The agent operates within a working directory for file operations. Change it with:
- `/cd` command in chat
- Directory picker in the TUI (Tab to select)
- `config_manager set_working_directory` tool

The working directory is persisted per-session. Switching sessions restores the directory automatically.

## Model Filter Hint (v0.3.78)

The onboarding model picker now **always renders a filter hint** above the model list ("Type to filter..."). Previously, the hint only appeared after you started typing, so users did not know filtering was available. Additionally, an **exactly-typed model ID** now beats a substring match: if you type `claude-opus-4-8` exactly, it selects that model even if other models contain the same substring.

