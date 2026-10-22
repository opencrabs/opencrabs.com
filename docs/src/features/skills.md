# Skills System

Skills are reusable workflow templates that extend OpenCrabs with specialized capabilities. They work across Claude Code, Anthropic managed agents, and OpenClaw using a shared `SKILL.md` format.

## How Skills Work

Each skill lives in its own directory under `~/.opencrabs/skills/`:

```
~/.opencrabs/skills/
├── security-audit/
│   └── SKILL.md
├── cost-estimate/
│   └── SKILL.md
└── my-custom-skill/
    └── SKILL.md
```

## Skill Format

Every skill is a markdown file with YAML frontmatter:

```markdown
---
name: security-audit
description: Language-agnostic security & CVE audit for any codebase
---

# Security Audit

You are a senior security engineer performing a comprehensive
security audit of the codebase in the current working directory...

### Path-Triggered Skills: `globs` (v0.5.1)

A skill can claim file patterns the way editor rule files do, via an opt-in `globs:`
frontmatter field: a comma string, an inline flow list, or a block list:

```markdown
---
name: rust-style
description: Project conventions for Rust edits
globs: ["src/**/*.rs", "build.rs"]
---
```

When the agent is about to touch a matching path **without that skill loaded in the current
context**, the tool call is rejected once, carrying the full skill body in the rejection. The
identical retry then passes: the model has now read the rules. A registry of seen skills
(track of loads, epoch-tracked) re-arms the gate after context compaction, and the master
switch is `[agent] skill_glob_gate` (default: on). The effect: conventions attached to a path
are impossible to edit against by omission.

## Stage 1: Project detection
...
```

The `name` and `description` fields in the frontmatter are required. The markdown body becomes the prompt that gets injected when the skill runs.

## Built-in vs User Skills

Skills come from two sources:

- **Built-in** (orange badge): ship with the OpenCrabs binary via `include_str!`. Always available.
- **User** (teal badge): created by you in `~/.opencrabs/skills/<name>/SKILL.md`. Override built-ins by file presence.

## Built-in Skills

| Skill | Description |
|-------|-------------|
| `opencli` | Reference for all 25+ opencli-rs dynamic tools (news, social, search, web). Use when user asks about trending topics, news, social media, jobs, or web search. |
| `browser-cdp` | Native CDP browser automation reference. Headless/headed Chrome control, screenshots, JS evaluation. |
| `a2a-gateway` | Agent-to-Agent (A2A) protocol gateway reference. JSON-RPC 2.0 peer-to-peer agent communication. |
| `dynamic-tools` | Runtime tool management with tool_manage and tools.toml format. Create, enable, disable, reload tools without restart. |
| `security-audit` | Language-agnostic security & CVE audit. Detects project type from manifests, runs the appropriate scanner, reviews the diff for injection / auth / crypto / deserialization / path-traversal patterns, and scores 0-100. |
| `cost-estimate` | Codebase cost-to-build estimate, AI-assisted ROI breakdown, and fair-market valuation. |
| `repo-audit` | Language-agnostic repository health checks. 5-phase pipeline: language detection, native tool execution, git metrics, AST analysis, scoring + recommendations. Covers Rust, JS/TS, Python, Go. (v0.3.18) |

## Running Skills

### Skills Picker (`/skills`)

Type `/skills` to open the full-screen filterable picker. The top shows a filter bar with the total skill count. The main area lists all skills, each showing:

- **Skill name** as a slash command (e.g. `/security-audit`)
- **Type badge**: orange `built-in` or teal `user`
- **Description** of what the skill does
- **Keywords** for search matching in parentheses

| Key | Action |
|-----|--------|
| `Tab` / `↑↓` | Navigate the skill list |
| `Enter` | Run the selected skill |
| `Esc` | Close the picker |
| Type | Filter skills by name and description (case-insensitive) |

When the filter narrows to a single match, Enter fires it immediately.

### Slash Commands

Type any skill name directly as a slash command:

```
/security-audit
```

### Channels

Skills auto-register as slash commands across all connected channels (Telegram, Discord, Slack, WhatsApp). No `commands.toml` entry needed. Just type `/<skill-name>` in any channel to run it.

## Creating Custom Skills

1. Create a directory under `~/.opencrabs/skills/`:

```bash
mkdir -p ~/.opencrabs/skills/my-skill
```

2. Create `SKILL.md` with frontmatter and prompt:

```markdown
---
name: my-skill
description: What this skill does
keywords: [my-skill, custom, example]
---

# My Skill

Instructions for the agent when this skill runs...
```

3. The skill immediately appears in `/skills` (with a `user` badge) and as `/my-skill` in TUI and all channels.

## Cross-Harness Compatibility

The `SKILL.md` format works identically on:

- **OpenCrabs**: native support via `/skills` picker and slash commands
- **Claude Code**: drop the same `SKILL.md` file into Claude Code's skills directory
- **Anthropic managed agents**: compatible with managed agent skill loading
- **OpenClaw**: works with OpenClaw's skill system

Write a skill once, use it everywhere.

## RSI-Proposed Skills

The RSI engine can propose new skills based on usage patterns it observes in the feedback ledger. For example, if the agent repeatedly performs a multi-step workflow that isn't covered by an existing skill, RSI will draft a skill and file it in the [Mission Control](./mission-control.md) inbox for your review.

This is part of the [RSI Proposals system](./self-improvement.md#rsi-proposals): RSI identifies gaps in the agent's capabilities and drafts solutions, but installation always requires your approval.

## Slash Command Resolution (v0.3.78)

The `slash_command` tool now correctly resolves **skills** and the `/onboard:<topic>` form. Previously, calling a skill via `slash_command` could fail if the skill name didn't exactly match the internal registry key. The resolver now checks skills, built-in commands, and user commands in priority order, and the `/onboard:voice` / `/onboard:provider` / `/onboard:channels` sub-topic forms route correctly.
