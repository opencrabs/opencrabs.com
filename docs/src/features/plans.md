# Plans

Plans provide structured multi-step task execution with a live progress widget in the TUI.

## Creating a Plan

Ask the agent to plan a complex task:

```
"Plan the migration from PostgreSQL to SQLite"
```

The agent uses the `plan` tool internally to create a plan with:
- Title and description
- Technical stack
- Risk assessment
- Test strategy
- Ordered tasks with dependencies and complexity ratings

## Plan Lifecycle

1. **Draft** — Agent creates the plan and adds tasks
2. **Finalize** — Agent calls `finalize` which triggers the tool approval dialog
3. **Approved** — You approve in the tool dialog, plan status becomes `Approved`, and the agent begins executing tasks immediately
4. **In Progress** — Tasks execute in dependency order
5. **Completed** — All tasks done

In **ask mode** (default), the `finalize` step triggers the tool approval dialog — you review the full plan before execution begins. In **auto-approve mode**, finalize is auto-approved and the agent plans and executes without pausing.

## Plan Requirement Gate

A turn that crosses a threshold of tool calls has to stop and write a plan before it continues (#2128). This is what keeps a sprawling refactor from being discovered after twenty files have already changed: you see the shape of the work, and you get a chance to say no before it lands.

The gate is configurable under `[agent]`. A turn that already has a live plan is never gated again, and a session configured to run autonomously is not stalled by an approval prompt it was told to skip.

## Task States

Each task in a plan can be:
- `Pending` (·) — Waiting for dependencies
- `InProgress` (▶) — Currently executing
- `Completed` (✓) — Done
- `Skipped` (✓) — Manually skipped
- `Failed` (✗) — Execution failed
- `Blocked` (·) — Dependencies not met

## TUI Plan Widget

When a plan is active, a live checklist panel appears above the input box showing:

- **Plan title** and progress counter (e.g. `3/7`)
- **Progress bar** — Visual `██████░░░░` bar with percentage
- **Task list** — Up to 6 tasks visible with status icons and task numbers
- **Overflow indicator** — `... (N more)` when tasks exceed the visible limit

The widget updates in real-time as the agent completes each task.

## Managing Plans

Plans are managed through natural language:

```
"Approve the plan"
"Reject the plan"
"What's the plan status?"
"Skip task 3"
```

The agent handles plan creation, approval, execution, and status reporting through the `plan` tool.

## Mid-Plan Insertion (v0.3.36)

Tasks can be inserted at any position in an existing plan using `insert_after`:

```
plan(operation: "add_task", insert_after: 3, title: "Re-run tests after fix", ...)
```

This inserts the new task as task #4, and all existing tasks from #4 onward are **renumbered automatically**. Dependencies between tasks are preserved through the renumber.

This is useful when a later task introduces a bug caught by an earlier test. Instead of re-opening the completed test task, insert a fresh re-test task right after the fix.

## Finished Checklist Persistence (v0.3.76)

When a plan completes all tasks, the finished checklist now **stays on screen** instead of disappearing. You can review the final state (all tasks marked ✓) without having to ask "what was the plan?" again. The completed card renders its final state with the full task list and progress bar at 100%.

## Epistemic Orient Gate (v0.3.78)

Plan execution is now gated behind an **epistemic Orient phase** before any task starts. The agent must:

1. **Observe** — gather ground truth (read files, check state)
2. **Orient** — map observations against current beliefs and goals
3. **Decide** — form intention, confirm alignment and safety
4. **Act** — execute through mechanical gates

This prevents the agent from charging into a plan task with stale assumptions. The Orient gate checks whether the agent's beliefs about the codebase/state still hold before writing anything.

## Criteria-Aware Verification (v0.3.78)

The Ralph loop (OpenCrabs' iterative task execution engine) now verifies task completion against **declared acceptance criteria** instead of just "did the command exit 0?". Each task's criteria are checked mechanically:

- If a task declares `acceptance_criteria`, the verification gate checks each one
- Criteria that require specific output (e.g. "clippy passes with zero warnings") are verified against actual tool output
- Tasks without explicit criteria fall back to the standard exit-code check

This means a task marked "complete" actually met its stated goals, not just "something ran."

## Ralph Verification Gate + Iteration Cap (v0.3.78)

The Ralph loop gained a **mechanical verification gate** with an iteration cap:

- After each task execution, the gate checks whether the outcome matches the intention
- If verification fails, the task retries (up to the iteration cap)
- The cap prevents infinite loops on tasks that can never satisfy their criteria
- State lives in files, not context — each iteration reads fresh state from disk

Combined with the epistemic Orient gate, this makes plan execution significantly more reliable: the agent orients before acting, and verifies after acting, with bounded retries.

## Plan State Across Sessions (v0.3.79)

Plans are no longer tied to the lifetime of a single session. Plan state threads across session boundaries, so a plan survives session swaps, restarts, and context compaction. Spawned child sessions resolve their parent's plan file automatically, which is what makes the next feature possible.

### Isolated Plan-Task Execution

Each plan task can now run in a **freshly spawned isolated worker session**: the worker gets only the task brief and the plan file, not the parent conversation's context. The brief is self-contained and the worker reports its verdict back via disk, so nothing leaks between the parent session and the task, and a long-running task can't burn the parent's context window.

- `agent.plan_isolated_execution` is the master switch, and it **defaults ON** since v0.3.79 — Ralph loops run fresh-context by construction, so isolation is the only sane default for autonomous execution.
- An explicit `isolated: true/false` on a `start` call still wins over the default.
- Ralph verification now runs in the **session's own working directory**, not the directory OpenCrabs was launched from — a plan in one repo is verified against that repo's build results, not another repo's (#921).
- The plan gate's `RequireApproval` decision respects `auto_approve`, so autonomous sessions are not stalled by an approval prompt they were configured to skip (#934).

## Type-Aware Verification (v0.3.83)

Acceptance criteria are **checked against the plan's toolchain** (#1133): a Rust task must satisfy cargo-shaped criteria, a Flutter task flutter-shaped ones, and a task whose criteria cannot be verified is marked instead of waved through. Plans also **verify with the project's own toolchain** rather than always cargo, and locate the project from the folder the session is working in. Checklist plans no longer write a design scaffold `.md` (#1145).

## Importing Pre-Defined Plans (v0.3.35)

Plans can be loaded from JSON files for repeatable workflows:

```
plan(operation: "import", file_path: "~/plans/rust-refactor.json")
```

Bundled reference plans ship with OpenCrabs at `~/.opencrabs/profiles/<profile>/plans/` covering common patterns like `rust-fast`, `rust-medium`, `rust-full`, `python-fast`, `python-medium`, `python-full`, and `sample-minimal-plan`.

The JSON format requires a minimum of 6 fields: `title`, `description`, plus 3 fields per task (`title`, `description`, `task_type`). Full schema supports dependencies, complexity ratings, acceptance criteria, and technical stack.

**Security:** Import validates symlinks against the target path only (rejecting ancestor false positives on macOS) and checks for orphan dependencies that reference non-existent tasks.
