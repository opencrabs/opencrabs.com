# Custom Commands

Define your own slash commands in `~/.opencrabs/commands.toml`. Commands work from the TUI and all channels (Telegram, Discord, Slack, WhatsApp).

## Configuration

```toml
# ~/.opencrabs/commands.toml

[commands.credits]
description = "Show remaining API credits"
action = "prompt"
value = "Check my API credit balance across all providers and give me a summary"

[commands.deploy]
description = "Deploy to production"
action = "prompt"
value = "Run the production deployment pipeline: git pull, build, test, deploy"

[commands.status]
description = "Show system status"
action = "system"
value = "System is operational. All channels connected."
```

## Action Types

| Action | Behavior |
|--------|----------|
| `prompt` | Sends the `value` as a message to the agent: the agent processes it like any user message |
| `system` | Displays the `value` directly as a system message: no agent involvement |

## Using Commands

Type `/commandname` in the TUI or any connected channel:

```
/credits     → agent checks API balances
/deploy      → agent runs deployment
/status      → shows static system message
```

## Visibility

Custom commands appear in:
- `/help` output (TUI and channels) under a "Custom Commands" section
- TUI slash autocomplete when typing `/`

Commands are sorted alphabetically and show their description.
