# Cron Jobs

Schedule tasks to run on a recurring schedule. Cron jobs can run in isolated sessions or wake the main session.

## CLI Management

```bash
# Add a job
opencrabs cron add \
  --name "Morning Report" \
  --cron "0 9 * * *" \
  --tz "Europe/London" \
  --prompt "Check emails, calendar, and give me a morning briefing" \
  --deliver-to telegram:123456

# List all jobs
opencrabs cron list

# Enable/disable (accepts name or ID)
opencrabs cron enable "Morning Report"
opencrabs cron disable "Morning Report"

# Remove (accepts name or ID)
opencrabs cron remove "Morning Report"
```

## Agent Management

The agent can also manage cron jobs via the `cron_manage` tool:

```
"Create a cron job that checks my emails every morning at 9am"
```

Since v0.3.80, `cron_manage` supports an **update action** that patches an existing job in place (#966), change the schedule, prompt, model, or delivery target without deleting and recreating the job.

## Options

| Flag | Description |
|------|-------------|
| `--name` | Job name (unique identifier) |
| `--cron` | Cron expression (e.g. `0 9 * * *`) |
| `--tz` | Timezone (e.g. `America/New_York`) |
| `--prompt` | The prompt to send to the agent |
| `--provider` | AI provider to use (optional) |
| `--model` | Model to use (optional) |
| `--thinking` | Thinking mode: `on`, `off`, `budget_XXk` |
| `--deliver-to` | Channel delivery: `telegram:CHAT_ID`, `discord:CHANNEL_ID`, HTTP webhook URL, or comma-separated multiple targets |
| `--auto-approve` | Auto-approve tool use for this job |

## Multi-Target Delivery

`deliver_to` accepts comma-separated targets to send results to multiple destinations simultaneously:

```bash
opencrabs cron add \
  --name "Morning Report" \
  --cron "0 9 * * *" \
  --prompt "Give me a morning briefing" \
  --deliver-to "telegram:-12345,http://webhook.example.com/notify"
```

Supported targets in any combination:
- `telegram:CHAT_ID` or `telegram:-GROUP_ID`
- `discord:CHANNEL_ID`
- `slack:CHANNEL_ID`
- `whatsapp:PHONE_OR_JID`
- `http://...` or `https://...` (webhook URL)

### Target URLs (v0.5.1)

Since v0.5.1 every delivery field also speaks a unified `oc://` target scheme, resolved by a
central resolver against the live channel bindings (issue #148):

| Form | Meaning |
|---|---|
| `oc://telegram/<chat>[/<thread>]` | Telegram chat (optionally a forum topic) |
| `oc://discord/<channel>` | Discord channel |
| `oc://slack/<channel>` | Slack channel |
| `oc://whatsapp/<phone\|jid>` | WhatsApp contact |
| `oc://session/<uuid-or-prefix>` | Park the result in that OpenCrabs session's notify queue, the agent gets cron output the same way a sub-agent or a detached command reaches its parent (v0.5.1) |
| `here` | This conversation's channel (refused on headless surfaces, which have none) |

Target URLs are resolved **once, at create/update time**, and the concrete channel target is
baked into the job; if an unbaked URL ever reaches fire time the run fails loudly instead of
resolving against a possibly-changed world.

### Forum-topic delivery (v0.5.1, opt-in)

`telegram:CHAT_ID:THREAD_ID` (or the `oc://telegram/<chat>/<thread>` form) delivers into a
specific topic of a forum group. It is deliberately opt-in: the chat must actually be a
forum and the topic must exist: an invalid thread target is **rejected loudly at fire
time**, never silently re-routed to the general topic. When a job is created from inside a
thread (`here`/`oc://` form), the origin thread id is persisted so the result lands back in
the same topic (#104).

Results are stored in the DB via the `cron_results` table regardless of delivery target, so you can query past execution results with `opencrabs cron results <name>`.

## Trigger-gated jobs (v0.5.3)

A job can carry a **pre-flight trigger**: at each fire time the scheduler runs a shell
command first, and only executes the prompt when a condition on that command's result is
met. This turns scheduled jobs from "fires into the void every hour" into "acts when
something actually happened" (a file appeared, a health check failed, a log line matched)
(#233).

Set it through the `cron_manage` tool at create or update time:

| Field | What it does |
|-------|--------------|
| `trigger_cmd` | Shell command run under `/bin/sh -c` before the prompt. 30s timeout; a hung trigger is killed, not orphaned. Either `prompt` or `trigger_cmd` must be set. |
| `trigger_on` | Fire condition (see below). Default `non_empty`. |
| `set_goal` | On fire, set the active goal in the destination session instead of a plain prompt turn. Requires `deliver_to` to target a session; refused for passive channel deliveries. |
| `goal_template` | Template for the goal/notification text. Interpolates `{output}`, `{stdout}`, `{stderr}`, `{exit_code}`. |

### `trigger_on` conditions

| Value | Fires when |
|-------|-----------|
| `non_empty` (default) | stdout or stderr contains any non-whitespace output |
| `exit_zero` | the trigger exits 0 |
| `exit_non_zero` | the trigger exits non-zero (health checks, `grep` misses) |
| `regex:PATTERN` or `re:PATTERN` | combined output matches the regular expression |
| `always` | the trigger finished, regardless of output or exit code |

### Semantics

- **Condition not met** → the run is short-circuited at 0 tokens and recorded as a
  skipped/success run. The prompt never executes, nothing is delivered.
- **Trigger command errors or times out** → the run is recorded as an error; the prompt
  still does not execute.
- **Empty `prompt` with a trigger** → direct 0-token delivery: the formatted trigger
  output (`goal_template`, or the raw output) is delivered to `deliver_to` without an
  agent turn at all.
- **`set_goal`** → the fired trigger dispatches into the target session as an active
  goal, so the session's agent works the condition it detected rather than just echoing
  output.

Put the watching in `trigger_cmd` (deterministic, cheap, script-runner pattern), the
condition in `trigger_on`, and the acting in the prompt or goal.

## Scheduler Lock (v0.3.65)

The cron scheduler uses a **file lock** to prevent duplicate job execution. Only one scheduler instance can run per profile at a time. If you accidentally start OpenCrabs twice, the second instance won't fire duplicate cron jobs.

## Heartbeat vs Cron

**Use heartbeat (`HEARTBEAT.md`) when:**
- Checks are periodic but timing is flexible (~30 min)
- You want to reduce API calls by batching
- Tasks share the main session context

**Use cron when:**
- Exact timing matters ("9:00 AM every Monday")
- Task needs isolation from main session
- You want a different model or thinking level
- Output should deliver to a specific channel
