# Brain Files

Brain files define the agent's personality, knowledge, and behavior. They live at `~/.opencrabs/` and are loaded on every session start.

## Startup Read Order

1. `SOUL.md`: Personality and values
2. `USER.md`: Your profile and preferences
3. `memory/YYYY-MM-DD.md`: Today's notes
4. `MEMORY.md`: Long-term memory
5. `AGENTS.md`: Agent behavior guidelines
6. `TOOLS.md`: Tool reference and custom notes
7. `CODE.md`: Coding standards and file organization
8. `SECURITY.md`: Security policies
9. `HEARTBEAT.md`: Periodic check tasks

## File Reference

### SOUL.md
Agent personality. Core truths: strong opinions, brevity, resourcefulness, honesty. Hard rules: never delete files without approval, never send emails without request, never commit code directly.

### USER.md
Your profile: name, location, timezone, role, specialties, communication preferences, pet peeves.

### AGENTS.md
Comprehensive agent behavior docs: memory system, safety rules, git rules, workspace vs repository separation, cron best practices, platform formatting, heartbeat guidelines.

### TOOLS.md
Tool parameter reference, system CLI tools, provider configuration, integration details for all channels and services.

### CODE.md
Coding standards brain template. Enforces: no file over 500 lines (target 100–250), types in `types.rs`, one responsibility per file, mandatory tests for every feature, security-first patterns. Rust-first philosophy, single binary, no runtime dependencies. The agent follows these rules when writing or reviewing code.

### SECURITY.md
Security policies: third-party code review, attack playbook awareness, network security, data handling, incident response.

### HEARTBEAT.md
Tasks for periodic proactive checks. Keep empty to skip heartbeat API calls. Add tasks for the agent to rotate through (email checks, calendar, weather, etc.).

### BOOT.md
Startup procedures: check git log, verify build, greet human with context awareness.

## Query-Based Loading (v0.3.76)

`load_brain_file` now accepts an optional `query` parameter that returns **only the sections matching the query** instead of the entire file. This saves significant context budget:

```
load_brain_file(name="TOOLS.md", query="telegram")
→ Returns only the Telegram-related sections, not the full 2000-line file
```

Whole sections are returned (never cut mid-section), so a rule is never truncated. This is especially useful for large brain files like TOOLS.md and AGENTS.md where loading the full file wastes 10K+ tokens on irrelevant content.

## Customization

These files are **yours**. The agent reads them but you control the content. Templates are at `src/docs/reference/templates/` in the source repo, compare your local files against templates when updating to pick up new sections without losing custom content.

> **New installs (v0.2.72+):** CODE.md and SECURITY.md are automatically seeded on first run. Existing users can ask their crab: *"Check my brain templates and update them if any are missing or outdated."*
>
> **Upgrading:** Brain files are never overwritten by `/evolve` or `/rebuild`. After updating, ask your crab to compare templates against local files and patch in new sections.

### User-owned files are never synced (v0.3.82)

`SOUL.md`, `USER.md` and `MEMORY.md` are **excluded from upstream template sync** (#1119). They hold your agent's personality, your identity, and what it has learned about you, so upstream has no authority over them. Before this, every startup appended upstream sections into them, which grew one user's SOUL.md from under 2 KB to nearly 5 KB, re-appending after each manual prune.

Seeding is unaffected: all three are still created on first run. Only the merge is forbidden. The remaining brain files (AGENTS, TOOLS, CODE, SECURITY, BOOT, HEARTBEAT) continue to sync normally.

Related (#1121): rules belong in `AGENTS.md`, not `MEMORY.md`. MEMORY.md is on-demand, reached only through `memory_search`, so a rule written there does not bind on a cold session and does not survive compaction. Facts go in MEMORY.md; anything that must always hold goes in AGENTS.md, which is always loaded.
