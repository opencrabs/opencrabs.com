# Multi-Agent Orchestration

OpenCrabs supports spawning specialized sub-agents that run autonomously in isolated sessions. Each child agent gets its own context, tool registry, and cancel token. Introduced in **v0.2.97** with a typed agent system and team orchestration.

## Agent Types

When spawning an agent, an `agent_type` parameter selects a specialized role with a curated tool set:

| Type | Role | Tools |
|------|------|-------|
| `general` | Full-capability agent (default) | All parent tools minus recursive/dangerous |
| `explore` | Fast codebase navigation (read-only) | `read_file`, `glob`, `grep`, `ls` |
| `plan` | Architecture planning (read + analysis) | `read_file`, `glob`, `grep`, `ls`, `bash` |
| `code` | Implementation (full write access) | All parent tools minus recursive/dangerous |
| `research` | Web search + documentation lookup | `read_file`, `glob`, `grep`, `ls`, `web_search`, `http_request` |

Each type receives a role-specific system prompt that shapes its behavior. Explore agents are fast and lightweight -- they only read files. Code agents can modify anything. Research agents can search the web but not touch your filesystem.

### Safety: ALWAYS_EXCLUDED Tools

No agent type has access to these tools, preventing dangerous or recursive operations:

- `spawn_agent` -- no spawning agents from agents
- `resume_agent`, `wait_agent`, `send_input`, `close_agent` -- no managing siblings
- `rebuild` -- no building from source
- `evolve` -- no self-updating

## Five Orchestration Tools

| Tool | Description |
|------|-------------|
| `spawn_agent` | Create a typed child agent to handle a sub-task autonomously in the background |
| `wait_agent` | Wait for a spawned agent to complete and return its output (configurable timeout) |
| `send_input` | Send follow-up instructions to a running agent (multi-turn conversation) |
| `close_agent` | Terminate a running agent and clean up its resources |
| `resume_agent` | Resume a completed or failed agent with a new prompt (preserves prior context) |

### Spawn an Agent

```
spawn_agent(
  label: "refactor-auth",      # Human-readable label
  agent_type: "code",          # general | explore | plan | code | research
  prompt: "Refactor auth...",  # Task instruction
  include_brain: false         # opt in to workspace brain pre-injection (v0.5.1)
)
```

The agent runs in its own session with auto-approved tools. No blocking -- it executes in the background while the parent continues.

### Lean children and `include_brain` (v0.5.1)

Sub-agents start **lean**: no pre-injected workspace brain (SOUL.md, USER.md, AGENTS.md,
project directives). Pass `include_brain: true` to `spawn_agent` (or per-agent in
`team_create`) to opt into pre-injection; the choice is frozen for the agent's lifetime and
`resume_agent` re-attaches accordingly. When a child runs lean, its prompt **discloses the
gap**: the child is told the workspace brain is not loaded and pointed at `load_brain_file`
for on-demand access, so neither side operates on the assumption rules are ambiently known (#145).

### Headless children (v0.5.1)

Child agents are **headless by law**: interactive-only tools (`session_notify`,
`suggest_options`) are stripped from the child's tool surface and the registry refuses to
register them there: a call returns a hard tool error, not a silent no-op. The headless
tool roster excludes interactive-only tools wholesale, and headless prompts (spawn and
resume, cron/daemon, CLI one-shot) carry a preamble saying so, so the child never reaches for
a button it does not have (#129). The same backstop covers cron's `here` target: it is
refused on headless surfaces.

### Wait for Completion

```
wait_agent(
  agent_id: "abc-123",
  timeout_secs: 300            # Max wait time (default: 300s)
)
```

### Multi-Turn with send_input

After spawning, you can send additional instructions without restarting:

```
send_input(
  agent_id: "abc-123",
  text: "Also add unit tests for the new module"
)
```

The child agent processes the input on its next iteration. This enables iterative workflows -- review the agent's output, then ask it to refine or continue.

### Resume a Completed Agent

```
resume_agent(
  agent_id: "abc-123",
  prompt: "Now port the same changes to the other two files"
)
```

The agent continues in its original session, preserving all prior context. No need to re-explain the codebase.

## Team Orchestration

The `TeamManager` coordinates named groups of agents for parallel execution. Three team-specific tools:

### Create a Team

```
team_create(
  team_name: "backend-refactor",
  agents: [
    { label: "auth", agent_type: "code", prompt: "Refactor auth module" },
    { label: "tests", agent_type: "code", prompt: "Write tests for auth" },
    { label: "docs", agent_type: "general", prompt: "Update documentation" }
  ]
)
```

All agents spawn simultaneously and run in parallel. Returns the team name and all agent IDs.

### Broadcast to a Team

```
team_broadcast(
  team_name: "backend-refactor",
  message: "Use the new AuthError enum instead of plain strings"
)
```

Sends a message to all running agents in the team. Non-running agents are skipped. Useful for sharing context or direction changes.

### Delete a Team

```
team_delete(team_name: "backend-refactor")
```

Cancels all running agents and cleans up resources. Completed agents are left in the subagent manager for reference.

## Subagent Provider/Model Config

By default, every spawned agent inherits the parent session's provider and model. You can override this globally in `config.toml` so child agents route to a different (usually cheaper or faster) backend:

```toml
[agent]
subagent_provider = "openrouter"   # Provider for child agents
subagent_model    = "qwen/qwen3-235b" # Model override

# Omit both keys and child agents inherit the parent session's provider
# and run on that provider's default model.
```

The override applies to `spawn_agent`, `resume_agent`, and every member of a `team_create` team. Changes take effect on next session start; running sessions keep their existing provider.

### Per-Call Overrides (v0.3.35)

`spawn_agent`, `resume_agent`, and `team_create` now accept optional `provider` and `model` fields that override config defaults for a single call. This enables mixed-model teams:

```
team_create(
  team_name: "mixed-stack",
  agents: [
    { label: "planner", provider: "zhipu", model: "glm-5", prompt: "Architect the feature" },
    { label: "coder", provider: "deepseek", model: "deepseek-coder", prompt: "Implement it" },
    { label: "reviewer", provider: "kimi", model: "kimi-k2.5", prompt: "Review the code" }
  ]
)
```

Precedence order (highest first): per-call `provider`/`model` > `[agent] subagent_provider`/`subagent_model` in config > parent session's provider.

### Why It Matters

The common pattern is **premium parent, cheap children**. Your main conversation stays on a reasoning-capable model (Opus, GPT-5, Gemini 2.5 Pro) while subtasks (file exploration, test writing, web research, bulk refactors) run on a faster, cheaper model. With a 4-agent team running 10 minutes each, the cost delta between Opus and Qwen on the children is roughly 50x.

### Concrete Examples

**OpenRouter parent, Qwen children**, best bang-for-buck on mixed workloads:
```toml
[providers.openrouter]
enabled = true
api_key = "sk-or-..."
model = "anthropic/claude-opus-4"

[agent]
subagent_provider = "openrouter"
subagent_model    = "qwen/qwen3-235b-a22b-instruct"
```

**Kimi on custom OpenCode provider**: fast code generation for `code` and `explore` agents:
```toml
[providers.opencode-kimi]
enabled = true
base_url = "https://api.kimi.com/v1"
api_key  = "..."
model    = "kimi-k2.5"

[agent]
subagent_provider = "opencode-kimi"
subagent_model    = "kimi-k2.6"
```

**Local Ollama children**: zero cost, fully offline, good for `explore` agents that just read files:
```toml
[providers.ollama]
enabled = true
model = "qwen3:14b"

[agent]
subagent_provider = "ollama"
subagent_model    = "qwen3:14b"
```

**Gemini parent, Gemini Flash children**, single billing account, reasoning on main, flash on team:
```toml
[providers.gemini]
enabled = true
model = "gemini-2.5-pro"

[agent]
subagent_provider = "gemini"
subagent_model    = "gemini-2.5-flash"
```

### Gotchas

- The subagent provider must be enabled and have a valid API key (or be a CLI/none-auth provider). Missing keys cause the spawn to fail with a provider resolution error.
- `subagent_model` must be a model the provider actually serves. `qwen/qwen3-235b` works on OpenRouter, not on Anthropic. Check `/models` on the target provider to confirm.
- `team_create` members all share the same subagent config. If you need heterogeneous routing (e.g. a research agent on web-search model, a code agent on code-specialized model), spawn them individually with `spawn_agent` under different config profiles.
- The CLI model override is surfaced in the `spawn_agent`, `resume_agent`, and `team_create` tool descriptions themselves, so the LLM knows to mention these keys to you instead of inventing per-call overrides.

If `subagent_provider` or `subagent_model` is not set, the spawned agent loads from the parent session's provider and runs on that provider's default model.

## Workflow Patterns

### Parallel Research + Implementation

```
team_create("feature-research", [
  { label: "research", agent_type: "research", prompt: "Find best practices for rate limiting in Rust" },
  { label: "explore", agent_type: "explore", prompt: "Find all middleware files in the codebase" }
])
```

Wait for results, then spawn a code agent with the combined context.

### Iterative Code Review

```
# 1. Spawn a code agent
spawn_agent(label: "impl", agent_type: "code", prompt: "Implement rate limiting middleware")

# 2. Wait for completion
wait_agent(agent_id: "impl-id")

# 3. Resume with refinements
resume_agent(agent_id: "impl-id", prompt: "Add tests for the edge cases we discussed")
```

### Large-Scale Refactoring

```
team_create("refactor-team", [
  { label: "module-a", agent_type: "code", prompt: "Refactor module A to use the new trait" },
  { label: "module-b", agent_type: "code", prompt: "Refactor module B to use the new trait" },
  { label: "module-c", agent_type: "code", prompt: "Refactor module C to use the new trait" },
  { label: "tests", agent_type: "code", prompt: "Update all tests for the new trait signature" }
])
```

## Testing

84 tests cover the entire multi-agent system:
- Manager state machine (spawn, wait, close lifecycle)
- `SendInput` wiring and input loop
- `CloseAgent` cleanup
- `WaitAgent` timeout behavior
- `AgentType` tool filtering
- `TeamManager`, `TeamDelete`, `TeamBroadcast`
- Registry exclusion (ALWAYS_EXCLUDED enforcement)

## Sub-Agent Result Reporting (v0.3.81)

A finished agent **reports its result back to the spawning session** (#1036): the parent sees the outcome in-turn instead of polling or reading status files. Orphaned sub-agent status files from crashed runs are **reconciled at startup** (#1038), so the registry never shows ghosts of agents that no longer exist.

## Sub-Agent Worktree Isolation (v0.3.83)

Each spawned agent gets **its own worktree and branch** instead of sharing the parent's working tree: a fan-out of several agents on one repository can no longer clobber each other's checkouts or builds. The parent's tree stays untouched while children work in isolation, and results come back through the normal git flow.
