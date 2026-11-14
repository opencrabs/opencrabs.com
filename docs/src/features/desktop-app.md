# The Desktop App

OpenCrabs ships two apps over the same agent. `opencrabs` is the terminal app you already know. `opencrabs-gui` is a native window with the same brain, the same sessions and the same config, rendered as cards instead of panes. Neither is a cut-down version of the other: the window runs the full agent, serves your channels and runs cron.

Both live in every release archive (v0.6.0). On macOS the window has its own bundle, `OpenCrabs Desktop.app`, so a double click opens the window instead of the terminal. On Linux the archive carries `opencrabs-gui.desktop` and an icon, so the window gets a menu entry with a real icon. On Windows the zip carries `opencrabs-gui.exe` beside the CLI.

## One profile, one owner

Launching either app claims the profile. If a background daemon is already serving that profile, the app you just opened takes it over: the daemon is asked to stop and the interactive app inherits the channels, the cron scheduler and the background work. That is the same rule the terminal app has always followed, and it means you never end up with two agents answering the same Telegram message.

Starting a second daemon is refused outright, and the error names the PID holding the profile.

## What the window gives you

| Surface | What it is for |
|---|---|
| Conversation | The transcript, with a turn folding and unfolding per `ctrl+o`, attachments rendering in place and code blocks keeping their own scroll |
| Rail | Sessions, pinned first, with a find field and a filter for automation sessions |
| Topics | A tab row per conversation, placed top, left, bottom or right |
| Split panes | Two or more conversations side by side, each with its own composer and transcript |
| Projects | Project and session cards, with batch assignment over a selection |
| Seeker | One field that finds a conversation, a command, a screen, a skill or a docs page |
| Settings | The agent's own config and credentials, written to `config.toml` and `keys.toml` |
| Themes | Four palettes, a font row and whole-window transparency |

## The chords

The window keeps the terminal's chords where they make sense and moves the rest onto the platform key. `ctrl+h` opens the help panel, which lists every chord the window answers, grouped the way the terminal groups them.

| Chord | What it does |
|---|---|
| `alt+space` (or `cmd+k`) | Open Seeker |
| `ctrl+,` | Open the settings card |
| `ctrl+shift+t` | Choose a theme |
| `alt+p` | Open projects |
| `ctrl+n` | New session |
| `ctrl+s` | List sessions |
| `ctrl+f` | Find in the rail |
| `ctrl+g` | Files for the selected session |
| `ctrl+shift+p` | Pin the selected session |
| `ctrl+d` | Delete the selected session (asks first) |
| `ctrl+shift+\|` / `ctrl+shift+_` | Split the window |
| `alt+tab` / `alt+shift+tab` | Rotate the caret between panes |
| `ctrl+o` | Fold or unfold the newest turn |
| `ctrl+k` | Clear the session |
| `ctrl+plus` / `ctrl+minus` / `ctrl+zero` | Zoom the window |
| `ctrl+h` | Help |
| `ctrl+shift+u` | Usage: tokens, cost and context |

Bare letters belong to the rail's find field once it is focused, which is why rename, topics and close-topic stay terminal-only: a typed name cannot double as a command. The row menu carries those actions in the window.

## Settings without a text editor

`ctrl+,` opens one card over the conversation, and every row writes through the same validated config path the terminal uses, so a value saved in the window is a value the terminal reads on its next boot.

The card covers the agent's approval policy, completion and failure notifications, redaction for groups and direct messages, lazy tool loading, the thinking-loop timeout, time markers, debug logging, the memory knobs, and a credentials section that writes an API key into `keys.toml` while keeping it masked on screen.

## Projects

`alt+p` opens the projects screen. Projects and sessions are cards, and the selection is what an action applies to: assigning, moving or clearing runs as one guarded operation over everything picked, so a batch cannot half-apply. The screen is backed by the same project service the terminal uses, which means a project you create in the window is the project `opencrabs -p` lists.

## Related

- [Themes](./themes.md) for the four window palettes, the font row and transparency
- [Seeker](./seeker.md) for what the card searches
- [Message actions](./message-actions.md) for editing, reacting, replying and forwarding a row
- [Split Panes](./split-panes.md) for the pane layout both apps share
