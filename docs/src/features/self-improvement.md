# Self-Improvement (RSI)

OpenCrabs improves itself over time through **Recursive Self-Improvement (RSI)**. The agent analyzes its own performance, identifies patterns, and autonomously updates its own brain files.

## How It Works

### 1. Feedback Collection
Every tool execution, user correction, and interaction is automatically logged to the feedback ledger. Categories include:
- `tool_success` / `tool_failure`: whether tool calls worked
- `user_correction`: when you corrected the agent's behavior
- `provider_error`: LLM stream drops, rate limits, timeouts
- `pattern_observed`: recurring behaviors the agent notices

### 2. Pattern Analysis
The agent calls `feedback_analyze` to review its performance:
- Per-tool success rates
- Recent failure patterns
- User correction frequency
- Provider reliability trends

### 3. Autonomous Improvement
When patterns are identified, the agent calls `self_improve` to:
- **read**: Load a brain file (SOUL.md, TOOLS.md, etc.) before modifying
- **apply**: Append new instructions based on observed patterns
- **update**: Surgically replace existing sections that need refinement
- **list**: Show all previously applied improvements

### 4. Change Tracking
Every improvement is logged to `~/.opencrabs/rsi/improvements.md` with:
- Timestamp
- Target file modified
- Description of the change
- Rationale (which feedback event triggered it)

Old improvements are archived to `~/.opencrabs/rsi/history/` to keep the active file lean.

## Example

```
User: "stop including testing steps in your output"
  → feedback_record(event_type="user_correction", dimension="output_hygiene")
  
Agent notices pattern of 5+ corrections on output hygiene:
  → feedback_analyze(query="failures")
  → self_improve(action="apply", target_file="SOUL.md", 
    content="Never include testing steps or verification commands in user-facing output.")
  → Logged to rsi/improvements.md
```

## Key Rules

- **No human approval needed** for self-improvements, the agent identifies patterns and applies fixes directly
- **Surgical updates only**: replaces specific sections, doesn't rewrite entire files
- **Always reads before modifying**: never blindly overwrites brain files
- **Archives old improvements**: keeps the improvement log manageable

## RSI Engine Architecture

The RSI engine is a background task that runs continuously alongside OpenCrabs. Here's how it works at each layer:

### Feedback Ledger

Every tool execution, user correction, provider error, and self-heal event is automatically logged to a SQLite-backed feedback ledger. Event types:

| Event Type | What It Tracks |
|---|---|
| `tool_success` / `tool_failure` | Whether tool calls worked, with args and error details |
| `user_correction` | When you corrected the agent's behavior |
| `provider_error` | LLM stream drops, rate limits, timeouts |
| `pattern_observed` | Recurring behaviors the agent notices |
| `context_compaction` | Context budget exceeded |
| `improvement_applied` | RSI applied a fix to a brain file |
| `self_heal_trigger` | Runtime self-heal caught and fixed an issue |

### Cycle Flow

1. **Startup**: writes a digest of feedback stats to `~/.opencrabs/rsi/digest.md`
2. **Every hour**: checks for new feedback entries since the last cycle
3. **Opportunity detection**: identifies tools with >20% failure rate (7-day window), user correction patterns, and provider errors
4. **Git-aware suppression**: checks if a fix commit already landed for the tool in question. If yes, suppresses the alert instead of re-reporting stale issues
5. **Autonomous agent spawn**: if opportunities are found, spawns a lightweight agent with RSI-only tools (feedback_analyze, self_improve, rsi_propose) that analyzes the data and applies targeted fixes

### Brain File Taxonomy

RSI routes improvements to the correct brain file based on what went wrong:

| Brain File | What It Controls | When RSI Writes Here |
|---|---|---|
| `SOUL.md` | Behavior, tone, reasoning patterns | Phantom tool calls, verbose responses, wrong tone |
| `TOOLS.md` | Tool usage, argument formats, pitfalls | Repeated tool failures with similar args |
| `USER.md` | User preferences and corrections | Repeated user corrections |
| `MEMORY.md` | Persistent knowledge and context | Agent lacks context it should retain |
| `AGENTS.md` | Workspace rules, safety policies | Agent-level behavior issues |
| `CODE.md` | Coding standards | Code quality feedback |
| `SECURITY.md` | Security policies | Security-related feedback |

### Repeat-Violation Escalation

RSI tracks violation counters inline in brain file rules. When a rule keeps getting broken, RSI bumps the counter and appends evidence (dates, session IDs). Rules that keep getting broken get louder, not silenced. This is the escalation pattern that makes RSI effective at fixing persistent bad habits.

## RSI Proposals

The RSI loop can propose new dynamic tools and slash commands based on gaps it observes in the agent's capabilities. Proposals land in TOML inboxes at:

```
~/.opencrabs/rsi/
├── proposed_tools.toml      # pending tool proposals
├── proposed_commands.toml   # pending command proposals
├── applied/                  # accepted proposals (daily archive)
│   ├── 2026-05-01-tools.toml
│   └── 2026-05-01-commands.toml
└── rejected/                 # rejected proposals (daily archive)
    ├── 2026-05-01-tools.toml
    └── 2026-05-01-commands.toml
```

### How Proposals Work

1. RSI analyzes feedback and notices the agent repeatedly working around a missing capability
2. RSI drafts a tool or command definition with a rationale citing the evidence
3. Proposal lands in the inbox: reviewed via Mission Control or the `rsi_proposals` tool
4. User applies or rejects: applied entries go to `tools.toml`/`commands.toml`, rejected entries are archived with an optional reason

### When RSI Proposes a Tool

- A specific bash command appears repeatedly across sessions (e.g. `gh issue list`, `docker ps`)
- The agent calls `http_request` to the same endpoint multiple times with similar payloads
- Only safe-by-default tools are proposed (read-only verbs, GET requests). Shell-based tools always set `requires_approval=true`

### When RSI Proposes a Command

- The user types `/something` repeatedly that doesn't exist
- A common multi-step prompt gets reused verbatim, a slash command saves typing

### Safety Guardrails

- **RSI never installs directly**: proposals require user approval via Mission Control or the `rsi_proposals` tool
- **No destructive proposals**: RSI will never propose `rm`, `dd`, `mv`, or any shell tool with destructive side effects
- **Deduplication**: if a proposal was already filed and not applied, RSI won't repropose it
- **One proposal per cycle**: quality over quantity
- **Evidence required**: every proposal cites the feedback events that drove it

## RSI Hardening (v0.3.13)

- **Append-only brain files**: brain files (SOUL.md, TOOLS.md, etc.) are now append-only with backup-before-write. The agent can only add new content, never delete or overwrite existing lines. This prevents accidental data loss from bad self-improvements.
- **Upstream template sync**: brain file templates are automatically synced from the upstream repo with version gating and append-only diffs. You get the latest improvements without losing your customizations.
- **RSI alert suppression**: alerts are suppressed when the dimension already has a fix commit, preventing noise on already-addressed issues.

## RSI Autonomous Proposals (v0.3.16)

The RSI loop can now propose new tools and slash commands autonomously. Proposals land in the Mission Control inbox for review, the agent identifies gaps from feedback data and drafts solutions, but installation requires human approval via the inbox UI or `/mission-control`.

## RSI Escalation for Repeat Violations (v0.3.17)

RSI now bumps a violation counter on existing rules instead of deduping repeat violations away. When a rule keeps getting broken across multiple sessions, the escalation counter increases and the agent prioritizes fixing that pattern. This prevents persistent bad habits from being silently ignored.

## v0.3.10 Additions

- **Cycle summaries no longer truncated**: full text displays in TUI instead of cutting off mid-sentence
- **Phantom detection reduced to 2-signal requirement**, needs both intent keyphrase AND zero tool calls before flagging, eliminating spurious self-heal triggers
- **Uses active provider**: respects current provider/model config instead of hardcoded Anthropic
- **Persistent session reuse**: one session per cycle, survives app restarts by persisting `last_cycle` timestamp
- **Skips unchanged feedback**: if feedback count hasn't changed, skips analysis to avoid wasted LLM calls

## v0.3.11 Additions

- **DashScope migration**: Qwen OAuth rotation replaced with simple API-key provider, deleting ~2,500 lines of complexity
- **Local model tool-call extraction**: auto-extracts tool calls from text content: bare JSON `{"tool_calls":[...]}`, Claude-style XML `<TOOLNAME><PARAM>value</PARAM></TOOLNAME>`, and Qwen-specific `<!-- tool_calls -->` markers
- **40+ TUI/self-heal fixes**: narrowed phantom gate, split thinking per iteration, anti-code-block nudge for local models, tighter phantom scope, mid-turn "Let me see:" catch, backtick code reference detection
- **Per-session provider isolation**: each session carries its own provider instance; no global swap affecting all sessions
- **Sub-agent `AwaitingInput` state**: `wait_agent` polls state and returns partial progress on timeout instead of deadlocking

## v0.3.20 Additions

- **RSI home directory resolution fixed**: RSI now resolves `~` to the actual home directory instead of using CWD-relative paths, preventing brain file writes to wrong locations
- **Bare tool-call arrays caught**: top-level arrays from models no longer crash RSI's feedback dimension parsing; wrapped correctly before recording

## v0.3.21 Additions

- **Multi-language phantom detection**: compile-time TOML char sets replaced language-specific regex patterns. RSI feedback now works with all supported languages via the new char-set system. Cross-language regression test added.
- **RSI cycle output dedup by hashing**: cycle output dedup now uses hash comparison of assembled opportunities instead of string matching, preventing duplicate cycle reports.
- **Sticky fallback on phantom exhaust**: when phantom detection exhausts retries, RSI applies sticky fallback provider to prevent cascading failures.
- **Phantom iterations not persisted**: phantom iterations no longer written to DB, keeping history clean of failed self-heal attempts.
- **OpenAI-compatible image generation**: image generation via any `/v1/images/generations` endpoint with configurable `generation_model` override.

## v0.3.23 Additions

- **Brain file clobber guardrail**: generic `write_file` / `edit_file` now refuse protected brain files, routing through `write_opencrabs_file` which enforces append-only, dedup-aware shrink, and `.bak` snapshots.
- **A2A approval policy**: A2A tasks now resolve approval policy correctly. `auto-always` and `auto-session` policies work for remote agents.

## v0.3.25 Additions

- **Brain file cleanup_intent**: `write_opencrabs_file` accepts `cleanup_intent` flag for user-driven brain file maintenance. RSI agent explicitly blocked from shrinking brain files, preventing autonomous self-improvement from accidentally wiping content (issue #103).
- **RTK Token Savings integration**: bundled RTK binary (4MB, v0.40.0) as default feature with zero-config. Works as direct proxy: agent runs `git status`, RTK intercepts output through Rust, filters it, returns token-optimized version. 100+ commands supported (git, cargo, npm, pnpm, docker, kubectl, grep, find, ls, tree, curl), blocklist for interactive/REPL commands (vim, ssh, python, mysql). Binary discovery checks bundled location first, falls back to PATH. `/rtk` slash command shows savings stats. Real-world results: 53.5% token savings across 180 commands (PR #102).

## v0.3.19 Additions

- **RSI feedback records actual model used**: when helpers remap a mismatched model to the provider default, RSI now records the resolved model instead of the impossible original pair. All 3 recording sites in `tool_loop.rs` now resolve the actual model before constructing the feedback dimension
- **Tool loop reasoning markers persisted**: reasoning content persisted in non-CLI content column so thinking state survives across tool loop iterations
- **@ file picker fixed for large repos**: recursive walk now skips `.git`/`.hg`/`.svn` directories and raised result cap from 5k to 20k, preventing pack/ref files from exhausting the cap

## v0.3.26 Additions

- **RSI brain file hygiene**: rejects raw failure-event logs from being written to brain files. Feedback dimensions are sanitized before persisting, preventing noise accumulation in SOUL.md and TOOLS.md
- **Hashline collision escalation**: when hashline_edit detects a collision (two lines with identical content hashes), RSI escalates to `edit_file` fallback instead of applying a corrupted edit
- **Dynamic help screen**: help screen auto-generates from `SLASH_COMMANDS` constant, so new commands appear automatically without manual help text updates

## v0.3.28 Additions

- **Brain backup rotation**: max 5 backups per file, max 7 days old. Prevents unbounded `.bak` accumulation in `~/.opencrabs/` from repeated RSI writes
- **Profile brain-template seeding**: `profile create` now seeds 8 brain file templates automatically, with recovery path for empty profiles. Ensures new profiles start with complete brain file sets
- **Auto-title retry on LLM failure**: auto-title no longer gives up on first LLM error; retries with backoff before falling back to truncated first message

## v0.3.30 Additions

- **RSI rejects trivial content**: `self_improve` apply action now rejects trivial test content before it can pollute brain files, preventing noise from accumulating in SOUL.md and TOOLS.md

## v0.3.31 Additions

- **RSI skill proposals**: `skill` is now a third proposal kind alongside `tool` and `command`. When RSI identifies a multi-stage workflow pattern that recurs across sessions, it proposes a `SKILL.md` file instead of a simple tool or command. Applied skill proposals write to `~/.opencrabs/skills/<name>/SKILL.md` and become immediately invocable as `/<name>` across all channels.
- **Bash command visibility**: RSI now sees the actual bash command text plus a subsystem classifier (git, cargo, docker, npm, etc.) in feedback events. This lets RSI identify recurring shell patterns more accurately and propose targeted tools or skills.
- **Successful patterns surface as proposals**: RSI doesn't only react to failures. When a tool/command/skill pattern works reliably across multiple sessions, RSI surfaces it as a proposal to make the pattern more discoverable or ergonomic.

## v0.3.34 Additions

- **Brain dedup scan (closes #147)**: new RSI proposal kind `BrainDedup` that scans all 11 brain files daily, clusters duplicate lines (minimum 10 chars, skips structural markdown like headings and separators), and files dedup proposals into Mission Control with a soft purple badge. Runs every 24 RSI cycles (about once per day at 1-hour intervals), never auto-applies, human approval required through the existing `rsi_proposals` apply/reject flow. Core scan logic in `dedup_scan.rs` (393 lines), hooked into the RSI cycle with periodicity gating, 14 regression tests covering empty files, short-line filtering, cross-file detection, proposal format, and canonical selection.
- **Skill description injection (closes #151)**: skill descriptions were documented in TOOLS.md as LLM auto-invoking triggers but were never actually injected into the system prompt, so the LLM could not auto-invoke from description alone. Added `push_skills_section()` to `prompt_builder.rs` that loads all skills via `crate::brain::skills::load_all_skills()` and formats each as `- skill_name: description`, appending an `## Available Skills` block to both `build_core_brain()` and `build_system_brain()`. 2 regression tests.
- **RSI decorative counters removed (closes #149, PR #150)**: removed the counter-bumping logic that incremented inline counters in SOUL.md like `phantom_tool_call: 219`. These counters were decorative only, nothing read them, and the real canonical source is the SQLite feedback ledger at `~/.opencrabs/feedback.db`. Counters went stale (ledger showed 302, SOUL.md showed 219) and got wiped by upstream template sync. Replaced with evidence appends (date/session). DB stays the single source of truth. Follow-up commit escaped unescaped double quotes the PR introduced in the prompt string literal and added text regression tests.

## v0.3.35 Additions

- **`cycle_number` persistence**: `cycle_number` now persists to `~/.opencrabs/rsi/cycle_number` across TUI restarts. Previously it reset to 0 on every process restart, meaning the brain dedup scan (which fires every 24 cycles) would never trigger if the TUI restarted more frequently than 24 hours. Now the cycle counter survives restarts and the daily dedup scan fires as intended.

## v0.3.66 Additions

- **Skill candidates from tool sequences**: RSI detects recurring tool sequences as skill candidates. When the agent repeatedly calls the same sequence of tools (e.g. `read_file` → `edit_file` → `bash` with specific patterns), RSI proposes a skill that encapsulates the workflow.
- **Slash commands from repeated asks**: RSI proposes slash commands from repeated user requests. When users type the same multi-word prompt repeatedly, RSI suggests a `/command` that captures the pattern.
- **RSI staleness indicator**: Mission Control shows a staleness indicator for RSI cycles, plus a provider-creation fallback when the primary provider is unavailable.

## v0.3.75 Additions

- **Brain dedup system**: a new `/dedup` command runs an on-demand cross-file brain dedup scan, backed by a post-write scan that fires after brain writes and a weekly cron safety net that catches cross-file duplication. The scanner ignores fenced code and table rows so it only flags real prose duplication.
- **Inline-keyboard dedup approval**: dedup proposals can be approved straight from a Telegram inline keyboard.
- **Brain-file hints on tool misses**: when a tool call misses or errors (including approval-branch errors), the relevant brain notes surface automatically so the agent can self-correct (#767).
- **Wider TOOLS.md load trigger**, routing, skill, and cron questions now proactively load TOOLS.md.

## v0.3.76 Additions

- **RSI /evolve proposal**: when a new OpenCrabs release is available, RSI surfaces it as an `/evolve` proposal in the Mission Control inbox instead of silently ignoring it. The proposal cites the version delta and links to the release notes.
- **Config-example tracking**: RSI tracks config examples so pricing fixes and new provider entries reach users through template sync.

## v0.3.78 Additions

- **TOML brain verification**: a new `brain_verify.toml` defines post-write verification rules. After any brain file write, the system re-reads the file and checks required anchors (H1 title, `**Owns:**` header) are still present. If verification fails, the write is flagged.
- **brain_verify Orient gate**: `self_improve` now routes through the epistemic Orient gate before applying changes. The agent must orient (verify current state) before writing, preventing RSI from applying fixes based on stale beliefs about brain file content.
- **RSI digest totals**: the RSI digest now surfaces raw failure totals alongside surfaced opportunities, giving better visibility into what's actually failing vs what RSI chose to act on.
- **Rule size cap**: brain file rules are now capped in size. A narrow consolidation path allows merging related rules, preventing unbounded growth from repeated RSI appends.
- **Contradiction scope fix**: brain_verify contradiction matching is now scoped per-entry instead of whole-file, eliminating false positives where unrelated entries triggered contradiction warnings.

## Self-Healing vs Self-Improvement

| Self-Healing | Self-Improvement |
|---|---|
| Fixes runtime errors (config corruption, DB issues) | Fixes behavioral patterns (bad habits, user corrections) |
| Automatic, no analysis needed | Requires feedback analysis first |
| Protects the system from crashing | Makes the agent better over time |
| Immediate | Accumulates across sessions |

## RSI Convergence Gates (v0.3.80)

The self-improvement engine now knows when to hold back (#977):

- **Pauses on convergence self-reports**: when a cycle reports its findings have converged, the RSI agent pauses instead of churning
- **Interval backs off on zero-improvement streaks**: repeated cycles that improve nothing automatically lengthen the wait before the next cycle
- **Findings hashed by stable identity**: the same finding keeps the same identity across cycles, so no description churn and no duplicate proposals
- **Gated on actionable feedback deltas**: cycles only run when there is new actionable feedback to work with
- **Session history sealed each cycle** with a compaction marker, so every cycle starts from a clean read
- **Cycle provider resolved from user config**, not registry order

## RSI Engine Gate (v0.3.81)

The autonomous RSI engine is gated behind `rsi_enabled` (#1063). In the TUI nothing changes, but **headless daemons default RSI off**, an unattended service no longer rewrites its own brain files by default. Opt in explicitly:

```toml
[agent]
rsi_enabled = true
```
