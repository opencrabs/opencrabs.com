# Self-Healing

OpenCrabs monitors its own health and automatically recovers from failures without user intervention. All recovery events surface as visible notifications across TUI and all channels.

## How It Differs from Crash Recovery

OpenCrabs has had **crash recovery** since early versions -- if the process dies mid-request, pending requests are tracked in SQLite and automatically resumed on restart (see [Pending Request Recovery](#pending-request-recovery) below).

**Self-healing** (v0.2.92) goes further: the agent detects and fixes problems *while it's still running* -- corrupted config, degraded providers, context overflow, stuck streams, DB corruption -- without restarting. Crash recovery is the safety net; self-healing prevents the fall.

## Config Recovery

Every successful write to `config.toml` creates a snapshot at `~/.opencrabs/config.last_good.toml`. When the config becomes corrupted or unparseable, OpenCrabs restores from the last-known-good snapshot automatically.

```
⚠️ Config was corrupted: restored from last-known-good snapshot (2 minutes ago)
```

A `CONFIG_RECOVERED` atomic flag tracks whether recovery happened during the current session, so downstream code can react accordingly.

### Unknown Key Detection

Unknown top-level keys in `config.toml` trigger a startup warning listing the unrecognized entries. This catches typos like `[teelgram]` or `[a2a_gatway]` before they cause silent misconfiguration.

Known valid sections: `[crabrace]`, `[database]`, `[logging]`, `[debug]`, `[providers]`, `[channels]`, `[agent]`, `[daemon]`, `[a2a]`, `[image]`, `[cron]`.

The `[a2a]` section also accepts `gateway` as an alias via serde, deduplicating a common typo.

### Custom Provider Name Normalization

Provider names with mixed case or whitespace (e.g. `"My Provider"` vs `"my provider"`) are normalized on load and save, preventing duplicate entries that would confuse the provider registry.

## Provider Health Tracking

Per-provider success/failure history is persisted to `~/.opencrabs/provider_health.json`. Each provider tracks:

- `last_success` and `last_failure` (epoch seconds)
- `last_error` (truncated to 200 chars)
- `consecutive_failures` count (resets on success)

```json
{
  "anthropic": {
    "last_success": 1743250500,
    "consecutive_failures": 0
  },
  "openai": {
    "last_success": 1743249800,
    "last_failure": 1743249700,
    "last_error": "rate_limit_exceeded",
    "consecutive_failures": 0
  }
}
```

The `/doctor` command surfaces health stats for every configured provider. Combined with the fallback provider chain, OpenCrabs detects degraded providers and routes to healthy ones automatically.

**Source:** `src/config/health.rs` (120 lines), integrated into `src/brain/agent/service/helpers.rs`.

## DB Integrity Check

SQLite `PRAGMA integrity_check` runs at startup. If corruption is detected, a notification appears in TUI and all connected channels instead of silently failing.

## Error Surfacing

v0.2.92 eliminated 14+ instances of silently swallowed errors across:

- Config writes
- Channel sends (Telegram, Discord, Slack, WhatsApp)
- Tool connections (Slack, WhatsApp, Trello connect tools)
- Pane state persistence

**Before:** `let _ = ...` and `.ok()` everywhere, errors vanish.
**After:** Every error surfaces via logging or user notification.

Onboarding config writes use `try_write!` macros that batch errors during wizard steps and report them all at the end, so users see exactly what failed.

### AgentService Config Propagation

`AgentService::new()` now requires an explicit `&Config` parameter instead of calling `Config::load()` internally. This eliminates hidden I/O, makes dependencies explicit, and enables test injection via `AgentService::new_for_test()`.

Render, dialogs, messaging, and cron modules no longer call `Config::load()` internally -- errors propagate up the call stack instead of being swallowed.

## Context Budget Management

The agent enforces a **65% context budget threshold**. When token usage reaches 65% of the effective context window (context limit minus tool schema overhead), automatic LLM compaction fires:

1. **Detect** context usage ≥ 65% of effective max tokens
2. **Compact** via LLM summarization (preserves meaning, not just truncation)
3. **Retry** up to 3 times if compaction fails
4. **Second pass** with tighter budget if still over threshold

The 65% threshold exists because providers like MiniMax degrade on function-calling quality well before hitting theoretical context limits -- tool calls break around ~133k tokens of a 200k limit.

### Async Proactive Compaction (v0.3.16)

At 65% context, compaction now runs **asynchronously in the background** instead of blocking the chat. The agent continues processing while the LLM summarizes older messages. Once compaction completes, the context is swapped seamlessly. No more frozen UI during compaction.

**Source:** `src/brain/agent/service/tool_loop.rs` (lines 14-112)

## Emergency Compaction (ARG_MAX Recovery)

When CLI provider conversation context exceeds the OS `ARG_MAX` limit (~1MB on macOS), the agent recovers with a 3-stage fallback:

1. **Catch** the "Argument list too long" or "prompt too large" error
2. **Emergency compact** the conversation with an LLM summarization pass
3. **Insert** a system marker so the agent knows context was compacted
4. **Retry** the request

If compaction still fails, **hard truncation** kicks in -- keeps last 24 messages (12 conversation pairs) with a marker telling the agent to use `search_session` for older context. Both markers persist to DB for recovery across sessions.

Both actions emit `SelfHealingAlert` progress events so users see exactly what happened.

**Source:** `src/brain/agent/service/tool_loop.rs` (lines 550-687), tested with `ArgTooLongMockProvider` and `ContextLengthMockProvider` in `src/tests/cli_arg_too_long_test.rs` (352 lines).

## Stream Resilience

### Stuck Loop Detection

Some streaming providers (notably MiniMax) occasionally loop the same content indefinitely without sending a stop signal. The agent detects this:

- Maintains a **2048-byte rolling window** of recent streamed text
- When a **200+ byte substring** from the second half appears in the first half, it's a repeat
- Stream is terminated immediately and retry logic fires

**Source:** `src/brain/agent/service/helpers.rs` -- `detect_text_repetition()`, tested in `src/tests/stream_loop_test.rs` (15 tests)

### Idle Timeout

If a stream goes silent for **60 seconds** (API providers) or **10 minutes** (CLI providers) with no events, it's treated as a dropped connection.

CLI providers (Claude CLI, OpenCode CLI) run internal tools (cargo builds, tests, `gh` commands) that can take several minutes without producing stream events. The 60-second timeout caused premature termination on these, so CLI providers now get a 10-minute window before timeout fires.

If a stream goes silent:

```rust
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
```

The `tokio::select!` loop races the stream against the timeout and the user's cancellation token. Timeout triggers retry, not a hard error.

## Pending Request Recovery

Crash recovery tracks every in-flight agent request in a `pending_requests` SQLite table. When a request starts, a row is inserted; when it completes (success or failure), the row is deleted.

On startup, any surviving rows mean the process crashed mid-request:

1. Query `pending_requests` for interrupted rows
2. Clear all rows (prevents double-recovery if this run also crashes)
3. Dedup by session_id (resume each session only once)
4. Spawn background tasks with a continuation prompt:
   > *"A restart just occurred while you were processing a request. Read the conversation context and continue where you left off naturally."*
5. Emit `TuiEvent::PendingResumed` so the TUI shows a recovery notification

**Source:** `src/db/repository/pending_request.rs`, `src/cli/ui.rs` (lines 705-790)

### Cross-Channel Crash Recovery (v0.2.93)

Before v0.2.93, pending request recovery always responded via the TUI, even if the original request came from Telegram, Discord, Slack, or WhatsApp. The resumed response would appear in the wrong place.

Now each channel passes its name and `chat_id` into `run_tool_loop`, which stores them in `pending_requests`. On restart, recovery routes responses back to the originating channel:

| Original channel | Recovery response goes to |
|---|---|
| Telegram | Same Telegram chat |
| Discord | Same Discord channel |
| Slack | Same Slack channel |
| WhatsApp | Same WhatsApp chat |
| Trello | Same Trello board |
| TUI | TUI (as before) |

The `pending_requests` table gained `channel` and `channel_chat_id` columns via a DB migration. `get_interrupted_for_channel` lets each channel handler query only its own pending rows. Selective `delete_ids` prevents one channel from clearing another channel's recovery entries.

## State Cleanup

Session deletion triggers **cascade deletes** across all related data:

- Messages (full conversation history)
- Usage ledger entries (token/cost records)
- Channel messages (Telegram, Discord, Slack, WhatsApp delivery records)
- Plans (autonomous plans created in the session)
- Cron jobs (scheduled tasks bound to the session)
- Cached pane state (stale split pane entries)

Custom provider names are **normalized** on load and save (`"My Provider"` → `"my-provider"`), preventing duplicate entries that would confuse the provider registry.

## Model Selector Safety

Pressing Enter in the model selector no longer clears existing API keys. The selector preserves current configuration while switching models.

Model switching errors now surface the actual error with a `⚠️` prefix on all channels, instead of always showing "Model switched" even on failure.

## UTF-8 Safety

`split_message()` across all 5 channel handlers (Telegram, Discord, Slack, WhatsApp, Trello) now uses `is_char_boundary()` to find safe split points, preventing panics on multi-byte characters (emojis, CJK, accented characters).

## Cancel Persistence (v0.2.97)

When a user double-Escapes to abort a streaming response, the partial content is now **persisted to the database before `handle.abort()` fires**. This means cancelled content survives a session reload -- you can scroll back and see exactly what the agent was saying before you stopped it.

### Claude CLI Subprocess Cleanup

Previously, aborting a Claude CLI request would orphan the underlying `claude` subprocess. Now the stream reader loop monitors `tx.closed()` via `tokio::select!` and kills the child process when the receiver drops, preventing leaked subprocesses accumulating in the background.

### Telegram Stale Delivery Suppression

When a request is cancelled mid-flight, the agent sometimes continued processing and delivered a stale response to Telegram. A `cancel_token.is_cancelled()` guard now fires before final delivery, preventing old agent results from posting after cancellation.

### Config Overwrite Protection

The onboarding wizard previously overwrote existing channel settings on every save, causing data loss when re-running `/onboard`. `apply_config()` now scopes writes to only the current onboarding step. `from_config()` sets `EXISTING_KEY_SENTINEL` for all existing channel data, ensuring untouched fields are never overwritten.

### Tool Description Wrapping

Tool call descriptions were previously truncated at 80 characters in the TUI. `render_tool_group` now **wraps** description headers and value lines to terminal width, and the 80-char pre-truncation of bash commands in `format_tool_description` has been removed. Long commands and file paths display fully.

## Auto-Fallback on Rate Limits (v0.2.98)

When the primary provider hits a rate or account limit mid-stream, OpenCrabs catches the `RateLimitExceeded` error, saves the current conversation state, and **resumes the same conversation on a fallback provider** configured in `[providers.fallback]`:

```toml
[providers.fallback]
enabled = true
providers = ["openrouter", "anthropic"]  # tried in order
```

The fallback chain reads from config at startup. `has_fallback_provider()` and `try_get_fallback_provider()` are available at runtime for dynamic queries.

### Two-Tier Context Budget Enforcement

Compaction budget scales proportionally to `max_tokens` instead of a hardcoded 170k, supporting custom providers with different context windows:

- **65% soft trigger**: LLM compaction with retries (preserves meaning)
- **90% hard floor**: Forced truncation to 75% (cannot fail)
- Pre-truncate target: 85% of max_tokens
- Compaction is **silent to user**: summary written to memory log only, no chat spam

### Mid-Stream Decode Retry (v0.3.0)

Transient stream decoding errors now trigger a **3x backoff retry** before falling back to the provider fallback chain. This reduces false provider switches caused by momentary network glitches.

### SIGINT Handler + Panic Hook (v0.3.0)

Proper terminal restoration on crash or Ctrl+C via custom SIGINT handler and panic hook. No more garbled terminal after interrupt, the handler restores raw mode, cursor visibility, and alternate screen before exiting.

### Proactive Rate Limiting (v0.2.99)

For OpenRouter `:free` models, OpenCrabs paces requests automatically using a shared global static limiter to avoid account-level bans. The rate limiter's first-call sentinel (`last_granted=0`) no longer causes an unnecessary sleep.

## RSI Alert Suppression (v0.3.13)

RSI alerts are now suppressed when the feedback dimension already has a fix commit in the recent git history. This prevents the agent from alerting on issues that have already been addressed. Stale alerts also age out via a sliding window on tool failure stats.

## Expanded Phantom Detection (v0.3.17)

The phantom detector now catches additional patterns:
- **"Now \<file-op gerund>" phantoms**: catches phrases like "Now creating...", "Now writing...", "Now editing..." where the model narrates a file operation without actually executing it
- **Build/deploy intent + past-tense completion claims**, catches when the model claims to have built or deployed something without running the actual commands
- **Module extraction**: gaslighting and phantom detectors extracted into their own dedicated module for cleaner maintenance

## RSI Escalation for Repeat Violations (v0.3.17)

RSI now bumps a violation counter on existing rules instead of deduping repeat violations away. Rules that keep getting broken get louder, not silenced. This prevents the agent from ignoring persistent failure patterns.

## Partial JSON Repair (v0.3.17)

A new `json_repair` module automatically fixes common JSON corruption:
- Closes unterminated strings
- Balances brackets
- Strips trailing commas
- Drops trailing keys-without-value

Wired into 5 drop sites across OpenAI-compatible providers and the ContentBlockStop finalizer. Unrecoverable input returns a `{"_partial": ..., "_repair_failed": true}` envelope instead of crashing the turn.

## Upstream Template Sync (v0.3.15)

Brain file templates are now automatically synced from the upstream OpenCrabs repo. The sync uses version gating (only applies templates from newer versions) and append-only diffs (never overwrites existing content). This ensures you always get the latest brain file improvements without losing your customizations.

## Browser Resilience (v0.3.18)

Multiple browser reliability improvements:
- **Network idle wait after navigate**: now waits for `networkIdle` instead of just CDP `load` event, catching async fetches
- **CDP manager lock released before await**: lock was held during screenshot await, blocking concurrent browser operations
- **CDP pre-flight health check**: added health check before screenshot capture to prevent stale connection failures
- **Browser navigate errors logged**: navigate errors no longer silently swallowed with `let _ =`, now logged at WARN

## Cloud Handshake Timeout (v0.3.18)

Bumped cloud provider handshake timeout from 30s to 60s. Routing proxies like dialagram legitimately take 20-45s; 30s was killing mid-request on slow-but-healthy providers.

## Gemini API Key Security Fix (v0.3.18)

Fixed CodeQL #64 (HIGH): Gemini API key was leaked in URL query string (`?key=...`) in `analyze_video`'s resumable upload init and file-state polling. Moved to `x-goog-api-key` header, matching `analyze_image` and `generate_image`.

## Stream & TUI Fixes (v0.3.18)

- **File paths starting with `/` no longer treated as slash command typos**, `/Users/.../file.pdf yo crabs check this` triggered "Unknown command". Added `looks_like_file_path()` helper gating both TUI and channel handlers.
- **Truncation continuations no longer trigger provider fallback**, mid-sentence continuations should stay on the same provider. Fallback now skipped for truncation paths.
- **Fallback error reason surfaced in TUI**: when fallback fired, the underlying error was swallowed. Now shows as a system message.
- **Pipe-delimited rows hard-broken**: when not recognized as a table, pipe rows ran together. Added hard-break between rows.

## v0.3.25 Fixes

- **Compaction dropped 55% kept-tail**: summary IS the conversation now, no more redundant tail retention
- **Self-heal 5-nudge budget**: reasoning-only turns get 5 nudges before sticky fallback, preventing empty replies from silently dropping
- **Completion-escape clause**: phantom enforcement messages now have escape clause to prevent infinite loops
- **Scroll fixes**: removed `load_more_history()` from scroll handler (overshoot fix), preserved scroll during streaming, skip first-render compensation
- **Brain file cleanup_intent**: `write_opencrabs_file` now accepts `cleanup_intent` flag for user-driven maintenance. RSI agent blocked from shrinking brain files (issue #103)
- **Channel improvements**: WhatsApp photo batching for multi-image uploads, Telegram `media_group_id`-based batching, Gemini schema strips `default`/`example` from tool schemas (#101, @leshchenko1979)
- **Custom provider model selection persistence**, properly saves and displays custom provider model selection
- **Compaction prompt dominance fix**: plan tool descriptions and scroll sensitivity improvements

## v0.3.23 Fixes (Hotfix Release)

- **Phantom detection restored**: v0.3.21's turn-level `tools_executed_this_turn` gate was too aggressive: once any tool ran in a turn, phantom detection went silent for the rest of the turn, letting fabricated wrap-up text reach the TUI. Dropped the gate from all three phantom branches.
- **Self-heal never aborts**: stuck-intent-loop now fast-escalates to sticky fallback instead of aborting; cap-exhaustion resets retry counter and injects hard nudge; `phantom_retries_used` now tracks consecutive phantoms since last real tool. Recovery always retries or falls back.
- **Brain file guardrail**: generic `write_file` / `edit_file` now refuse to modify protected brain files (SOUL.md, USER.md, TOOLS.md, etc.), preventing accidental clobber. Routed through `write_opencrabs_file` instead.
- **A2A approval policy wired**: A2A `message/send` tasks now resolve approval policy via `check_approval_policy()`. With `auto-always` set, tools auto-approve; otherwise returns warning. Fixes "Tool requires approval but no approval mechanism configured" errors.
- **Channel `/new` session switching fixed**: `/new` now uses per-message resolver's title format everywhere (Telegram, Discord, Slack), so session switching works across all channels.
- **Version-aware model sort**: when OpenAI-compatible servers return zero or identical `created` timestamps, extracts numeric segments from model names and sorts newest version first. Fixes meaningless model lists on vLLM/llama.cpp.

## v0.3.22 Fixes

- **Compaction typing without banner**: reverted the visible "🗜️ Compacting context" banner text. Now uses typing-only refresh (Telegram `send_chat_action(Typing)`, Discord `broadcast_typing` loop) keeping the "is typing" indicator alive during the 10-60s compaction window silently.
- **Channel `/new` archive consistency**: unified archive behavior across all channels: non-owner sessions get archived (so next title lookup resolves cleanly), owner sessions stay non-archived and remain visible in `/sessions`.

## v0.3.21 Fixes

- **Multi-language phantom detection via compile-time TOML**, replaced regex patterns per language with TOML-defined char sets compiled into build-time match arms. New languages added by editing TOML, no Rust changes. Cross-language regression test added.
- **Self-heal pipeline hardened**: phantom detection gated on turn-level tool execution, phantom iterations no longer persisted to DB, phantom text stripped from context before next turn, sticky fallback applied on exhaust.
- **OpenAI-compatible image generation**: new image generation backend calling any `/v1/images/generations` endpoint. Providers override generation model independently via `generation_model` config field.
- **Working directory visible across tools**: working directory now visible to all tools within the same iteration.
- **Compaction banner stripped from context**: compaction banner text no longer fed to LLM context, preventing models from echoing it back.
- **Pipe-separate model callback**: custom-provider model callbacks now pipe-separated so colons in provider names (e.g. "Qwen: DashScope") survive parse.
- **Custom-provider model selection persists**: `/models` dialog now correctly saves and syncs live model list for custom providers.
- **`one_shot_pct` display corrected**: fixed incorrect percentage display in usage dashboard.
- **Session `updated_at` touched on switch**: session last-modified timestamp updated when switching sessions via Telegram, preventing stale session resolution.

## v0.3.19 Fixes

- **Cron provider/model cross-contamination fixed**, cron's `execute_job` called global `swap_provider()` instead of session-scoped `swap_provider_for_session()`, so concurrent cron jobs on the shared `Cron` session overwrote each other's provider. Now each job swaps on its own session ID.
- **Cron mismatched pair validation**: reversed cron config (e.g. `default_model = "zhipu"` where `zhipu` is a provider name) produced impossible pairs like `dialagram/zhipu` that timed out with no diagnostics. Added validation: if `effective_model` is not in the provider's `supported_models()`, the job is skipped with a loud error.
- **Windows CI test failures fixed**: `tool_loop_helpers_test.rs` used hardcoded Unix `/tmp/` paths and `/etc/hosts` assertions. Added platform-specific test variants with `#[cfg(unix)]` / `#[cfg(windows)]`.
- **CI Node 24 forced upgrade removed**: removed `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24: true` env var that broke `actions/cache@v4` with `punycode` deprecation on Node 21+.
- **Codex OAuth device flow field names fixed**: OpenAI's device auth API uses non-standard field names (`device_auth_id` instead of `device_code`, string `interval` instead of number, `expires_at` instead of `expires_in`). Fixed with serde aliases and custom deserializer.
- **Codex OAuth verification URL corrected**: was hardcoded to non-existent `auth.openai.com/verify`, changed to `auth.openai.com/codex/device` matching Codex CLI.
- **Codex OAuth model list curated**: `/models` dialog showed non-OpenAI models (Phi-4, Llama, Mistral) because the `codex` provider ID wasn't mapped to the curated GPT-5 model list.

## v0.3.26 Fixes

- **Hashline collision detection** (#105): pure content hash prevents line-shift avalanche when lines are inserted/deleted above a hash anchor. On collision, escalates to `edit_file` fallback instead of corrupting the edit
- **RSI brain file hygiene** (#111): rejects raw failure-event logs from being written to brain files. RSI now sanitizes feedback dimensions before persisting
- **Tool error output** (#113): tool errors now include stdout/stderr in error content, ANSI escape sequences stripped, 8000 char cap to prevent context blowout

## v0.3.27 Fixes

- **Ctx budget baseline on channel `/new`**: shows calibrated baseline immediately after `/new` instead of waiting for first message
- **Auto-title session fix** (#114): preserves `[chat:ID]` suffix to prevent title duplication on subsequent auto-title fires
- **Sessions display** (#115): arrow prefix + "current" label instead of checkmark for clearer session switching UI

## v0.3.28 Fixes

- **Voicebox + STT/TTS fallback chains**: 2s liveness probe detects dead audio devices, librosa error translator surfaces actionable messages instead of Python tracebacks, per-provider fallback chains configured in `config.toml`
- **Browser multi-step navigation hardening** (6 commits): `text=`/`xpath=` selector prefixes, recovery hints on click failures, semantic loop detection (4+ screenshots in 8 iterations triggers abort), no-op screenshot rejection, same-URL short-circuit
- **Tool-call shape recovery**: dict-by-call-id extraction for Qwen-3.7-max-preview regression where tool calls arrive as flat dicts instead of nested arrays
- **Edit tool improvements** (#117): fuzzy line-sequence fallback when exact match fails, hashline docs clarification
- **Brain backup rotation**: max 5 backups per file, max 7 days old, preventing unbounded `.bak` accumulation
- **Auto-title fixes** (#118, #120): fires on FIRST turn (not second), retries on LLM failure instead of giving up
- **Ctx counter real-time only** (#119): ripped out calibration system entirely, uses provider-reported `input_tokens` verbatim. No more "0/max" for uncalibrated providers
- **Profile brain-template seeding**: seeds 8 templates on `profile create`, recovery path for empty profiles

## v0.3.29 Fixes

- **Auto-title thinking-block fallback** (#121): reasoning models returning only a Thinking block (no Text block) now get a title extracted from the thinking content instead of dropping silently. `extract_title_candidate` falls back to `pluck_title_from_thinking` (last quoted phrase, then last short sentence)
- **Telegram label-drift fix** (PR #123 @leshchenko1979): auto-titled sessions no longer overwritten on every subsequent message. `should_refresh_label` policy only refreshes default→default-different or group label changes, never auto-titled or custom titles. Chat→session binding on `/sessions` switch

## v0.3.30 Fixes

- **5-language deferment stall detection**: self-heal catches "I need to X" / "I have to X" / "I must X" / "I should X" patterns in English, Spanish, Portuguese, French, and Russian
- **Follow-up message = ESC x2 cancel**: all four channel handlers treat a follow-up message during an active agent run as double-Escape cancel, then starts fresh
- **Dynamic Telegram status messages**: replaced hardcoded quips with context-aware messages showing actual tool being called, tokens streamed, and elapsed time
- **rename_session rejects empty titles** (#128): whitespace-only titles rejected so sessions can't become unidentifiable

## v0.3.31 Fixes

- **Fun POST-COMPACTION PROTOCOL prompts**: after compaction, the agent receives a playful system prompt instead of a sterile summary marker. These rotating prompts (e.g. "You just woke up from a nap. The summary above is everything you remember.") make the post-compaction experience less robotic. Users can opt out with `[agent] silent_compaction = true` in config.toml.
- **Telegram forum topic routing**: in supergroups with topics enabled, `thread_id` is tracked through the full pipeline. The agent can use `list_topics` to map topic names to IDs, then route responses to specific topics via `thread_id` on send/reply/send_photo.
- **PDF `page_range` param**: `parse_document` now accepts `page_range` strings like `"1-30"`, `"5,7,10-15"`, or `"3"` for targeted extraction. Text-first routing skips Gemini for text-native PDFs. Inline preview cap raised to ~60 pages.

## v0.3.32 Fixes

- **Evolve hardening** (#136): the `/evolve` command now handles busy Linux binaries with a remove+rename dance (can't overwrite a running binary on Linux), delayed `systemd-run` restart to let the current process finish cleanly, structured tracing for better error diagnosis, and a pre-flight `count_matching_systemd_units` check to avoid restarting when multiple OpenCrabs instances are running.

## v0.3.33 Fixes

- **User-correction metadata** (#138, PR #140): `display_text_override` now captures the actual user message text instead of the 236-character Telegram channel prefix that was previously stored. This makes user correction entries in the feedback ledger readable and actionable.

## v0.3.34 Fixes

- **`follow_up_question` race fix (closes #142)**: all four channels (Telegram, Discord, Slack, WhatsApp) now flush intermediate text handles before presenting the follow-up keyboard. Prevents the race where the bot's in-progress message got orphaned or duplicated when the user tapped a button mid-stream. Each channel got its own atomic commit with per-channel regression tests pinning the flush-before-keyboard sequence.
- **`follow_up_question` display polish (closes #148)**: Telegram keyboard is now single-column with a 40-character label cap (rejects options longer than 40 chars in the tool validator with a clear error). Rolling "Running follow_up_question (16s)" status is suppressed while the keyboard is pending, and the LLM is now instructed to call the tool silently without echoing the question text in surrounding prose. Discord left alone due to its 5-ActionRow-per-message hard limit.
- **Phantom detector hardening**: two narration shapes had been leaking past the phantom detector: pronounless deferment (`Need to read the X`) and bare gerunds (`Reading the current state of the affected files`). Added 28 pronounless EN variants, 15 telegraphic FR `besoin de` variants, and gerund+determiner bigrams. New regression file pins both leaked sentences verbatim. Follow-up fixed French accent detection: `detect_language` missed `é/è/ë/ü`, so French narration fell through to English and the new `besoin de` phrases never matched. Added the 4 markers.
- **Fallback provider cascade (closes #152)**: `/models` swaps and session restores were storing a raw provider instead of wrapping it in `FallbackProvider`, so the fallback cascade could not fire on 5xx/429 errors after a model switch. Every active provider now gets wrapped unless it is already a chain or no fallbacks are configured. 174-line integration test simulating 5xx cascades across swapped providers.
- **Error persistence**: agent failures now persist as permanent chat bubbles with actionable wording on TUI and channels instead of vanishing after the turn. UTF-8 panic after redact-prefix scan fixed by snapping to char boundary.
- **FINISHING A TURN rewrite**: split the brain preamble directive into side-effect vs analysis response shapes, added a nudge on empty data-fetch closes so the agent never ends a turn silently after running research tools, and requires an explicit acknowledgement sentence instead of letting `finish_reason: stop` with no content reach the user.
- **Claude CLI model auto-learn**: footer showed Opus 4.7 after Anthropic shipped 4.8 because `default_for_alias` hardcoded `opus -> opus-4-7`. Now the provider learns the CLI-resolved version from `message_start` events, persists to `~/.opencrabs/claude_cli_models.json` (rewriting only when the value changes), and the TUI refreshes the session model live so the footer self-corrects to the actual version without code changes. `default_for_alias` prefers the learned cache and falls back to a build-time seed only on a fresh install.
- **tok/s in channel footers**: channel context budget footers showed only `ctx: XK/YK Z%` while the TUI also showed `| N tok/s`. Added `tokens_per_second: Option<f64>` to `AgentResponse`, extended `format_ctx_footer` to accept a third `tps` parameter, computed tok/s from `total_output_tokens / turn_duration` across the whole turn in `tool_loop.rs`, and wired it through all four channels.

## Unreleased (post-v0.3.33)

- **Phantom post-success exemption**: the phantom detector used to fire on short completion acknowledgments like "Pushed.", "Done.", or "Committed as abc123" because those look like past-tense completion claims without a tool call. But when the agent just finished a real tool run, that one-line ack is the *correct* behavior. A turn-scoped `tool_calls_completed_this_turn` counter and a `phantom_eligible` gate now suppress phantom detection once real tool calls have landed in the current turn. The complementary `FINISHING A TURN` brain preamble directive tells the agent to reply with one short ack, skip verification re-runs, and stop restating conclusions in different wording.
- **`follow_up_question` intermediate flush** (issue #142): when the agent called `follow_up_question` after typing an explanatory preamble, Telegram/Discord/Slack/WhatsApp sometimes delivered the button block *before* the preamble text because intermediate text sat in a 500ms-polled queue while `follow_up_question` sent directly. All four channel handlers now flush pending intermediate `JoinHandle`s before dispatching the question, guaranteeing the explanatory text renders above the buttons.

## v0.3.36 Fixes

- **Near-miss tool name self-heal (closes #176)**: when the model guesses a wrong tool name (e.g. `tg_send_message` instead of `telegram_send`), the tool registry now tries three fallback strategies before returning NotFound: (1) normalized match (strip underscores, lowercase), (2) abbreviation expansion (`tg` → `telegram`, `wa` → `whatsapp`), (3) typo fallback (Levenshtein distance). Conservative: only heals on a unique high-confidence match, so ambiguous guesses still return an error.
- **Retry overhaul**: patient backoff defaults changed from 100ms hammering to 1s/2s/4s/8s. Rate limits now retry in-place (3 retries) before falling through the fallback chain. Hard-down endpoints (DNS failure, connection refused) fail fast instead of wasting retries. Transient 4xx HTML infra pages (Cloudflare, nginx error pages) are retried. Each retry surfaces as a `RetryAttempt N/M - reason` event to the user.
- **Provider/model contamination prevention**: atomic provider+model swap across 27 call sites prevents the footer from showing a stale provider after switching panes or sessions.
- **Secret redaction expanded**: query-param keys (`?api_key=...`) and URL passwords (`https://user:pass@host`) now redacted alongside Bearer tokens and API key patterns. RSI TUI notifications also redacted.

## v0.3.35 Phantom Hardening

- **Re-engaged on forward intent**: phantom detector now re-engages self-heal when forward intent is detected after a successful tool call, preventing the agent from narrating what it's about to do instead of just doing it.
- **Five-language cleanup**: destructive verb intent phrases cleaned up across EN, ES, FR, PT, and RU. Prevents the agent from narrating destructive actions ("I'll now delete the file") when it should just execute the tool.

## Thinking-Loop Timeout (v0.3.78)

A new **thinking-loop timeout** catches the agent stuck in an infinite reasoning loop (producing thought tokens but never emitting a tool call or response). When the timeout fires:

1. The stuck stream is terminated
2. Phantom enforcement retry fires (nudges the agent to produce real output)
3. If still stuck after retries, sticky fallback provider kicks in

This eliminates the "agent appears to be thinking forever but never does anything" failure mode.

## Expanded Phantom Detection (v0.3.76–v0.3.78)

The phantom detector gained significant coverage across three releases:

### v0.3.76 Additions
- **File-delivery claims**: catches "I've sent the file" / "document delivered" when no file-sending tool ran
- **Null-effect tool calls**: tools that execute but produce no observable effect no longer grant phantom immunity
- **Command verification**: quoted commands are checked against what the turn actually ran
- **Evidence checking**: quoted evidence (output, logs) is verified against actual tool returns
- **Investigation claims**: "I investigated" / "I looked into" counts as a completion claim requiring evidence
- **Both-ends check**: examines both the lead-in AND the wrap-up of a turn, not just the opening
- **Self-update claims**: "I updated my memory" / "I noted this" counts as a completion claim

### v0.3.78 Additions
- **Shipped-and-tracked claims**: "shipped and tracked in issue #N" without actual gh/git calls
- **Colloquial delivery claims**: "that's out the door", "it's in your inbox", casual completion language
- **Media-delivery claims**: "I've attached the image" / "video sent" without media tools running

## v0.3.75 Hardening

- **Bounded self-heal loop**: the phantom self-heal loop is now bounded instead of re-nudging forever, and discarded phantom self-heal narration is stripped from the live buffer so it never reaches the chat.
- **Patient mid-stream retry**: mid-stream retry is more patient: 5 attempts with exponential backoff instead of giving up early.
- **Flaky 404 retried**: a flaky-provider 404 is retried instead of being treated as a permanent failure.
- **Multilingual hallucination detection**: image-generation hallucination detection (a media claim with no marker and zero tool calls) now works across languages, and self-heal no longer delivers an image hallucination when it gives up.
- **Poisoned-session recovery**: auto-recovers from a repetitive-tool-call poisoned session instead of looping on the same broken call.
- **Phantom verdict on recovery paths**: the turn-end phantom verdict now catches narration delivered via recovery paths, and issue-tracker actions count as completion claims.

## Loop Guards (v0.3.80)

Three guards stop the agent repeating itself (#957, #961):

- **Cross-turn announcement loop guard**: detects when the agent keeps announcing the same action across turns without actually doing it
- **Near-match loop guard**: catches reworded repetitions of the same tool call, generalized to ALL tools, not just bash
- **Mid-turn announcement checks**: reworded announcement loops are caught inside the turn, not only between turns

When the loop detector kills a stalled turn, that kill **reaches the fallback chain** (#1023): it is raised as a provider-attributable error so the turn can retry with another provider instead of surfacing as a hard failure, and it is no longer misreported as a provider fault. Spent thinking-loop budget is routed into the fallback chain the same way (#1021).

## Notifications

All self-healing events are delivered to:
- TUI (status bar notification)
- Telegram, Discord, Slack, WhatsApp (if connected)

Nothing happens silently. If the crab fixes itself, it tells you what it fixed.

## Early Loop Detection (v0.3.81)

The loop detector now notices a **repeating tool round before the provider rejects the turn**. Instead of burning the context window on identical rounds and surfacing a provider error, the detector catches the pattern early and routes it through the existing loop-kill path (which reaches the fallback chain since v0.3.80).

## Fact-Based Detectors Fire the Heal (v0.3.81)

Phantom-detection refinements (#1073, #1074):

- **Fact-based detectors self-heal**: detectors that verify against ground truth (files, command output) now fire the self-heal path directly, not just gate the claim
- **Wider command allowlist**: inspection-style claims are checkable, so "I ran X" verification covers more real phrasings
- **Dotted commands survive the sentence split**: `some.command --flag` is no longer torn in half when the claim is parsed
