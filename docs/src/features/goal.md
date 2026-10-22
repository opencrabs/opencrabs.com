# Autonomous Goal Loop (/goal)

The `/goal` command lets you set a high-level goal and have OpenCrabs work toward it **autonomously**, executing actions, self-evaluating with an LLM judge, and continuing until the goal is satisfied or a turn budget runs out.

## How It Works

1. **Set a goal**: `/goal <description of what you want>`
2. **The agent loops**: executes an action, then an LLM judge evaluates whether the goal is met
3. **Self-correction**: if the goal isn't satisfied, the agent continues with a correction prompt
4. **Completion**: the loop ends when the judge says the goal is met, or the turn budget (default 20 turns) is exhausted

This is fully hands-off. Set the goal, walk away, come back to results.

## Usage

```
/goal <text> (Set a new goal and start the autonomous loop
/goal status) Check current goal progress
/goal pause (Pause the autonomous loop
/goal resume) Resume a paused goal
/goal clear: Remove the current goal
```

## Examples

```
/goal Fix all failing tests in the auth module and make sure clippy passes
/goal Research the top 5 Rust web frameworks and write a comparison in research/frameworks.md
/goal Set up a CI pipeline with GitHub Actions for this project
/goal Refactor the database layer to use connection pooling
/goal Find and fix all TODO comments in src/handlers/
```

## Turn Budget

The default turn budget is **20 autonomous turns**. Each turn is one full LLM round-trip (action + evaluation). The agent uses a lightweight LLM judge to evaluate progress, keeping costs low.

If the budget runs out before the goal is satisfied, the agent reports what it accomplished and what remains.

## Best Practices

- **Be specific**: "Fix the login bug" works better than "make the app better"
- **Set boundaries**: mention files, modules, or constraints in the goal
- **Use for multi-step tasks**: `/goal` shines on tasks that need repeated execution and self-correction
- **Check status**: use `/goal status` to monitor progress on long-running goals
- **Pause when needed**: `/goal pause` if you need the agent for something else

## Self-Goaling (v0.3.61)

The agent can also set and drive its own multi-turn goals autonomously via the `goal_manage` tool. This lets the agent break down a complex request into self-directed goal loops without the user needing to invoke `/goal` manually. The agent evaluates its own progress and adjusts course, the same way a user-set `/goal` loop works.

## Availability

`/goal` works across **all channels**: TUI, Telegram, Discord, Slack, and WhatsApp. Set autonomous goals from wherever you talk to your agent.

## When to Use /goal vs /btw vs Plans

| Feature | Use Case |
|---------|----------|
| `/goal` | Autonomous loop for a single clear objective. Agent self-evaluates until done. |
| `/btw` | Spawn a parallel side task while you keep chatting. |
| Plans | Multi-step structured work with explicit tasks, dependencies, and acceptance criteria. |

`/goal` is best when you have a clear outcome and want the agent to figure out the steps itself.
