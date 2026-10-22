# Split Panes

OpenCrabs supports tmux-style pane splitting in the TUI. Run multiple sessions side by side, each with its own provider, model, and context, all processing in parallel.

## Splitting

| Action | Shortcut |
|--------|----------|
| Split horizontal | `\|` (pipe) |
| Split vertical | `_` (underscore) |
| Cycle focus | `Tab` |
| Close pane | `Ctrl+X` |

## How It Works

Each pane runs an independent session. You can have one pane writing code with Claude while another reviews tests with Gemini. The status bar shows `[n/total]` to indicate which pane is focused.

- **Independent providers**: Each pane can use a different AI provider and model
- **Independent context**: Conversation history is isolated per pane
- **Parallel processing**: All panes process concurrently via Tokio
- **Persistent sessions**: Each pane's session is saved to SQLite like any other session

## Example Layout

```
┌──────────────────────┬──────────────────────┐
│  Session 1 (Claude)  │  Session 2 (Gemini)  │
│  Writing code...     │  Reviewing PR...     │
├──────────────────────┴──────────────────────┤
│  Session 3 (OpenRouter)                      │
│  Running tests...                            │
└──────────────────────────────────────────────┘
```

Split vertically with `_`, then horizontally with `|` in the top pane.

## Persistent Layout

Split pane configuration (splits, sizes, focused pane) saves to `~/.opencrabs/pane_layout.json` on quit and Ctrl+C. On restart, your layout is restored exactly as you left it. Each restored pane preloads its session messages from the database, so content is visible immediately instead of blank.

## Non-Focused Panes

Non-focused panes show compact tool call summaries and stripped reasoning text. Tool groups display as single collapsed lines matching the focused pane style. All panes auto-scroll to the bottom when new messages arrive.

v0.2.92 fixed several rendering issues:
- Tool calls no longer show a perpetual "running" spinner after completion
- Scroll position correctly tracks for inactive panes
- Stale cache is cleared when sessions are updated or deleted

## State Management

Deleting a session now properly cleans up cached pane state. Previously, deleting a session left stale entries in the pane cache, which could cause phantom panes on restart.

## Live Background Updates (v0.3.36)

Inactive panes now update **live** in the background. Previously, non-focused panes only refreshed when you switched focus to them. Now a background-session live-state cache routes `IntermediateText` and `QueuedUserMessage` events into per-session deltas, so you can watch tool calls and responses appearing in other panes in real time without switching focus.

**Provider/model contamination prevention:** When closing or switching panes, the old session's provider is captured *before* the switch. This blocks cross-provider model leaks at 27 call sites throughout the codebase. The footer always shows the correct provider+model for the focused pane.

**Ctrl+N** binds the focused pane and live-refreshes the footer title, so new sessions show up immediately in the status bar.

## Reasoning Expand (v0.3.74)

Reasoning blocks now cycle through **three states** instead of toggling open/closed. Click a reasoning block (or press `Ctrl+O`) to cycle: **collapsed → capped → full**. The capped middle state shows a bounded preview so a long chain-of-thought never floods the viewport (#727, #726).

Two refinements make expanding predictable:

- **Anchor block on expand**: the clicked block's header is pinned to its screen row, so expanding or collapsing grows the block in place instead of jumping the viewport (#728).
- **Click vs drag on mouse-up**: click-vs-drag is decided on mouse-up, so a click-drag selects text (for copying) instead of toggling the expand state (#726).

## Version in the UI (v0.3.74)

The running OpenCrabs version now shows on the **TUI header** and in channel `/help` and `/usage` output (#696), so you always know which build a session is on.

## Double-Escape Returns the Query (v0.3.74)

Pressing `Esc` twice before any reply now **returns the query to the input box and removes it** from the conversation (#698), so a half-typed prompt you cancel doesn't linger as an empty turn.

## Per-Turn Headers (v0.3.75)

Every turn now gets a **one-line header** summarising the work that turn did, and the turn's working-out folds up into it. OpenCrabs infers turn boundaries so each turn can be grouped, then folds the narration, intermediate text, and tool calls underneath the header.

The view stays clean by default:

- **Fold every turn by default**: turns fold as soon as they settle (and even while still running), so settled work collapses out of the way instead of scrolling off screen.
- **Fold live**: intermediate text folds while the turn is still running, and stale narration the model kept thinking past is collapsed.
- **Readable folded turn**: a folded turn stays readable and holds its place when toggled, and the tool-call summary stays at full visibility.
- **Short live-thinking excerpt**: live thinking shows a short excerpt with room to finish a thought, not a scrolling wall.
- **Labelled token counter**: the live token counter is labelled as a turn total and shows the ctx budget; ctx dropped from the spinner since it already shows under the input.

Expanding a turn is smooth: the header is kept after expand, and expanding no longer scrolls the view up by its own size.

## TUI Improvements (v0.3.76–v0.3.78)

### Steps AND Tools Display (v0.3.76)
The TUI now shows **both steps and tool calls** in the turn view. Previously it was either/or: if tool calls existed, steps were hidden. Now both render, with steps providing the narrative and tool calls showing the mechanical detail.

### Background Task Position (v0.3.77)
Background task feedback moved from the **left border to the right border** of the input box. This prevents it from colliding with the context budget footer on the left.

### Duplicate-Submit Guard (v0.3.76)
Pressing Enter twice quickly no longer submits the same message twice. The TUI drops a re-submitted message if the running turn is already answering it.

### Command Labelling (v0.3.76)
Commands in the TUI are now labelled by **what they run** (e.g. "cargo test") instead of the working directory. Makes the activity feed readable at a glance.

### Deliverable Report Visibility (v0.3.78)
When a turn produces a deliverable report (audit, comparison, analysis), the report **stays visible** even when the turn folds. Previously, folding a turn would hide the report along with the working-out.

### Background Running Indicator (v0.3.78)
The "running" indicator is now **cleared before delivering** the final response. Previously, the indicator could linger for a moment after the response appeared, creating a visual glitch.

## Limits

There is no hard limit on pane count -- you can run as many as your terminal fits. Each pane is a full session with its own token tracking and working directory.
