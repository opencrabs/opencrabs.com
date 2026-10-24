# AI Provider Setup

OpenCrabs supports 19 providers (18 built-in + Custom OpenAI-Compatible). Configure them through the **onboarding wizard** or manually via `config.toml` and `keys.toml` at `~/.opencrabs/`.

## Setup via Onboarding Wizard

The fastest way to configure a provider is the interactive wizard. Run `/onboard:provider` (or `/onboard` and navigate to step 3).

### Keyboard Navigation

| Key | Action |
|-----|--------|
| `↑` / `↓` or `j` / `k` | Move between providers / models |
| `Enter` or `Tab` | Advance to next field |
| `BackTab` or `Shift+Tab` | Go back to previous field |
| `Esc` | Go back to previous wizard step |
| Type any character | Filter model list (when on model picker) |

### Live-Fetch Providers (recommended)

For providers with a `/v1/models` API endpoint, the wizard fetches the model list live after you enter your API key.

**Supported:** Anthropic, OpenAI, OpenRouter, Gemini, MiniMax, Qwen (DashScope), Ollama, z.ai GLM

**Flow:**

1. Use `↑`/`↓` to select your provider (e.g. **OpenRouter**)
2. Press `Enter`: advances to the API key field
3. Paste your API key (e.g. `sk-or-...`)
4. Press `Enter`: triggers a live fetch from the provider's `/v1/models` endpoint
5. Use `↑`/`↓` to browse models, or **type to filter** (case-insensitive substring match)
6. Press `Enter` on your chosen model: saves config and advances

> **Tip:** If you've already configured a key, the wizard detects it (shown as `••••••••`) and skips straight to the model picker. Press `Enter` to re-fetch models with the existing key.

### OAuth Providers (GitHub Copilot, Codex)

No API key needed: authenticate through your browser.

**Flow:**

1. Select **GitHub Copilot** or **Codex**
2. Press `Enter`: starts the device-code PKCE flow
3. A user code and URL appear (e.g. `github.com/login/device`)
4. Open the URL in your browser, enter the code, and authorize
5. Tokens are saved automatically to `~/.opencrabs/auth/`
6. Models are fetched live: pick one and press `Enter`

### CLI Providers (Claude CLI, OpenCode CLI, Codex CLI)

Use your existing CLI subscription: no separate API key.

**Flow:**

1. Select the CLI provider (e.g. **Claude CLI**)
2. Press `Enter`: skips the API key field (none needed)
3. Models are fetched from the local binary (`claude models`, `opencode models`, etc.)
4. Pick a model and press `Enter`

**Requirements:** The CLI binary must be installed and authenticated in your PATH.

### z.ai GLM (Zhipu AI)

z.ai has two endpoint types. The wizard asks which one before the API key.

**Flow:**

1. Select **z.ai GLM**
2. Press `Enter`: advances to **Endpoint Type** selector
3. Use `↑`/`↓` to choose `API` (general) or `Coding` (CodeGeeX)
4. Press `Enter`: advances to API key field
5. Paste your key, press `Enter`, fetches models
6. Pick a model and press `Enter`

### Custom OpenAI-Compatible

For Ollama, LM Studio, LocalAI, Groq, NVIDIA, vLLM, or any OpenAI-compatible endpoint.

**Flow:**

1. Select **Custom OpenAI-Compatible** (last in the list)
2. Press `Enter`: advances to **Name** field
3. **Name**: type a provider identifier (e.g. `ollama`, `lm-studio`, `nvidia`). Press `Enter`, normalized to a TOML-safe key
4. **Base URL**: paste your endpoint (e.g. `http://localhost:1234/v1`). Press `Enter`
5. **API Key**: paste if required, or leave empty for local endpoints. Press `Enter`
6. **Model**: you have two options:
   - **Type or paste a model name**: use this for newly-launched models not yet available on the live API (e.g. `qwen3.6-35b-a3b-gguf`)
   - **Press Enter on empty field**: triggers a live fetch from `{base_url}/models`, then pick from the list
7. **Context Window**: enter the token limit (e.g. `128000`). Press `Enter`, saves and advances

> **Context Window Recommendation:** Set to **200000** (200k tokens) for best results. OpenCrabs handles large contexts gracefully with smart auto-compaction that keeps you always up to date without manual intervention.
>
> **Local LLMs:** No API key needed, just set base URL and model name. If the model is already running, paste the name directly. If you want to browse available models, leave the Model field empty and press Enter to fetch the list from your local server.

### Re-running Provider Setup

| Command | What it does |
|---------|-------------|
| `/onboard:provider` | Jump to provider setup, return to chat when done |
| `/models` | Switch provider/model for the current session |
| `/onboard` | Full wizard (all steps) |

## Manual Configuration (advanced)

If you prefer editing files directly, configure providers in `config.toml` and `keys.toml`.

---

### Key Handling (v0.3.81)

`keys.toml` no longer drops provider API keys at runtime (#1066): four providers could silently lose their keys during config reloads. The stored-key marker has **one owner**, and keys already poisoned by the old marker are **self-healed** on load. If a provider worked yesterday and 401s today, re-check `keys.toml`: the fix also repairs the file in place.

## OpenCrabs (built in)

This is the default a fresh install lands on: no account, no key to paste, no billing. It is the only provider that is both free and set up out of the box.

**What it serves on day 0**

| Capability | Model | Notes |
|---|---|---|
| Chat | `deepseek-v4.1-flash` | 200k context window, fixed |
| Vision | `deepseek-v4.1-flash` | `vision_model` points at the same model, so attached images work immediately |
| Image generation | `gemini-2.5-flash-image` | Image-only: never offered as a chat model |
| Voice in | `qwen3.8-omni-flash` | The endpoint answers no `/v1/audio/*` route, so audio rides an `input_audio` part on the chat request |
| Voice out | not served | A spoken reply needs another TTS engine (see [Voice](../features/voice.md)) |

**The context window is fixed at 200k.** The endpoint is free and serves one model, so `context_window` on this section is ignored rather than clamped.

**How the key works.** There is no key to find. On first use the install generates its own ID and asks `inference.opencrabs.com` for a caller token, then stores it in `keys.toml` under `[providers.opencrabs]`. A stored install is reused as-is, so the second launch never calls the proxy. The binary holds no shared upstream key, and the server keeps only a hash of each token. Lose `keys.toml` and the install requests a new token on the next launch.

**Cost and limits.** Free, with no usage caps during the experimental phase. To keep it that way the proxy rate limits or blocks requests that flood the service or mint installs in bulk. Normal use does not hit those protections. If limits change, the README says so before they apply.

**Where your data goes.** Chat and image requests are sent to `inference.opencrabs.com`, which forwards them to the upstream model host. If that is not acceptable for a conversation, switch providers.

**Switching.** Type `/models` and pick another provider, or run `/onboard` and choose Advanced. Other providers' keys are untouched.

```toml
# config.toml
[providers.opencrabs]
enabled = false                              # onboarding turns this on; do not flip it by hand
default_model = "deepseek-v4.1-flash"
models = ["deepseek-v4.1-flash"]
vision_model = "deepseek-v4.1-flash"         # vision on day 0
generation_model = "gemini-2.5-flash-image"  # image generation on day 0, served only on the image route
stt_model = "qwen3.8-omni-flash"             # transcription on day 0
```

`enabled = false` is the shipped default, and it means "not the default", not "dead": by-name lookups, `/models`, the fallback chain, and image/vision overrides can all still reach the section. What it cannot do is work without the install token, so turning it on by hand on a home that never onboarded leaves a provider with no credential. Choose it through onboarding instead.

`gemini-2.5-flash-image` is deliberately absent from `models`. It is served only on `/v1/images/generations`, never as a chat model, so listing it there would buy a failed chat request.


## Anthropic Claude

**Models:** `claude-opus-4-6`, `claude-sonnet-4-5`, `claude-haiku-4-5`, and legacy models, fetched live from the API.

```toml
# keys.toml
[providers.anthropic]
api_key = "sk-ant-..."
```

```toml
# config.toml
[providers.anthropic]
enabled = true
default_model = "claude-sonnet-4-20250514"
```

**Features:** Streaming, tool use, extended thinking, vision, 200K context window.

## OpenAI

**Models:** `GPT-5 Turbo`, `GPT-5`, and others, fetched live.

```toml
# keys.toml
[providers.openai]
api_key = "sk-YOUR_KEY"
```

## OpenRouter: 400+ Models

Access 400+ models from every major provider through a single API key. Includes free models (DeepSeek-R1, Llama 3.3, Gemma 2, Mistral 7B).

```toml
# keys.toml
[providers.openrouter]
api_key = "sk-or-YOUR_KEY"
```

Get a key at [openrouter.ai/keys](https://openrouter.ai/keys). Model list is fetched live, no binary update needed when new models are added.

## Google Gemini

**Models:** `gemini-2.5-flash`, `gemini-2.0-flash`, `gemini-1.5-pro`, fetched live.

```toml
# keys.toml
[providers.gemini]
api_key = "AIza..."
```

```toml
# config.toml
[providers.gemini]
enabled = true
default_model = "gemini-2.5-flash"
```

**Features:** Streaming, tool use, vision, 1M+ token context window.

> Gemini also powers the separate image generation and vision tools. See [Image Generation & Vision](./images.md).

## GitHub Copilot

Use your existing GitHub Copilot subscription: no separate API charges. Authenticates via OAuth device flow.

```toml
# config.toml
[providers.github_copilot]
enabled = true
```

**Setup:** Run `/onboard:providers` → select GitHub Copilot → follow the device code flow at `github.com/login/device`. Models are fetched live from the Copilot API.

**Requirements:** An active [GitHub Copilot](https://github.com/features/copilot) subscription (Individual, Business, or Enterprise).

## z.ai (Zhipu AI)

**Models:** GLM-4-Plus, GLM-4-Flash, GLM-4-0520, CodeGeeX, fetched live. Two endpoint types: General API and Coding API.

```toml
# keys.toml
[providers.zai]
api_key = "your-api-key"
```

```toml
# config.toml
[providers.zai]
enabled = true
default_model = "glm-4-plus"
```

Get your API key at [open.bigmodel.cn](https://open.bigmodel.cn).

## Qwen (DashScope)

Qwen models via DashScope. As of v0.3.81:

- **Reasoning carries across turns** (#1033): `preserve_thinking` is sent so multi-turn conversations keep their reasoning context
- **Thinking knobs are family-gated** (#1034): each model family gets only the knob it actually reads (`enable_thinking`, effort tiers), instead of a shared guess
- **Tiered families default to the recommended effort tier** (#1034)
- **Hosted Qwen reachable everywhere** (#1040): works outside Alibaba networks and with vendor-prefixed model ids

## Claude CLI

Use your existing Claude Code subscription through the local `claude` binary, no separate API key needed. Supports streaming and extended thinking.

```toml
# config.toml
[providers.claude_cli]
enabled = true
```

**Requirements:** The `claude` CLI must be installed and authenticated. Models are detected automatically.

## Ollama

Run any Ollama model natively: no custom provider setup needed. Supports both local (localhost:11434) and cloud (api.ollama.com) instances.

```toml
# config.toml
[providers.ollama]
enabled = true
default_model = "llama3"
```

```toml
# keys.toml (optional, only for cloud Ollama)
[providers.ollama]
api_key = "your-api-key"
```

**Features:** Streaming, tool use, local model tool-call extraction from text. Models are fetched live from the Ollama API.

**Requirements:** Ollama must be running locally (`ollama serve`) or you must have a cloud Ollama API key.

## OpenCode CLI

Use the local `opencode` binary for free LLM completions, no API key or subscription needed. Supports NDJSON streaming and extended thinking.

```toml
# config.toml
[providers.opencode_cli]
enabled = true
```

**Requirements:** The `opencode` binary must be installed and available in your PATH. Models are fetched live via `opencode models`.

## Codex CLI

Use OpenAI's `@openai/codex` CLI as a native provider. User authenticates once via `codex` CLI; OpenCrabs piggybacks on cached credentials, zero API key handling. Non-interactive mode via `codex exec --json` with JSONL streaming.

```toml
# config.toml
[providers.codex_cli]
enabled = true
```

**Models:** GPT-5.5, GPT-5.4, GPT-5.3-Codex

**Requirements:** The `codex` CLI must be installed (`npm install -g @openai/codex`) and authenticated. Models are detected automatically.

## Command Code CLI

Spawns the `command-code` CLI (also installed as `cmd`) in non-interactive mode (`command-code -p`) and uses it as the LLM backend, so an existing Command Code account works with no separate API key. OpenCrabs keeps tools, memory, and context local.

```toml
# config.toml
[providers.command_code_cli]
enabled = true
default_model = "deepseek/deepseek-v4-flash"
```

Auth comes from `~/.commandcode/auth.json`, written by the CLI itself. The model list mirrors `command-code --list-models`, and the CLI validates the name against what your account can reach.

Unlike the Claude and Codex CLIs, `command-code -p` answers in plain text rather than NDJSON, so the provider collects stdout and translates it into one streamed message. That means no token-by-token streaming: the reply arrives in a single chunk.

**Features:** tools handled locally, model list from the CLI, no API key to store


## Codex OAuth

Native OpenAI Codex subscription auth via device-code PKCE flow. No CLI dependency, no API key. User authenticates through browser once; tokens stored in `~/.opencrabs/auth/codex.json` with automatic refresh and background rotation.

```toml
# config.toml
[providers.codex]
enabled = true
```

**Models:** GPT-5.5, GPT-5.4, GPT-5.3-Codex (curated GPT-5 model list)

**Setup:** Run `/onboard:provider` → select Codex OAuth → follow the device code flow at `auth.openai.com/codex/device`. Two-step PKCE exchange: device auth poll → authorization code → token exchange.

**Requirements:** An active OpenAI Codex subscription. No CLI installation needed.

## MiniMax

**Models:** `MiniMax-M2.7`, `MiniMax-M2.5`, `MiniMax-M2.1`, `MiniMax-Text-01`

```toml
# keys.toml
[providers.minimax]
api_key = "your-api-key"
```

Get your API key from [platform.minimax.io](https://platform.minimax.io). Model list comes from `config.toml` (no `/models` endpoint).

## Moonshot Kimi

**Models:** `k3`, `kimi-for-coding`, `kimi-for-coding-highspeed`, fetched live from the API.

```toml
# keys.toml
[providers.moonshot]
api_key = "your-api-key"
```

```toml
# config.toml
[providers.moonshot]
enabled = true
default_model = "k3"
endpoint_type = "api"      # "api" (API plan) or "coding" (Coding plan)
reasoning_effort = "max"   # Sent only when the active model accepts it; a no-op otherwise
```

Two endpoint types, chosen in onboarding or with `/models`:

- **API plan** (`api`): pay-as-you-go at `api.moonshot.ai/v1`, key from [platform.moonshot.ai](https://platform.moonshot.ai)
- **Coding plan** (`coding`): Kimi token subscription at `api.kimi.com/coding/v1`, key from [kimi.com](https://www.kimi.com)

A `base_url` you set always wins over the plan default. Kimi's `reasoning_content` works on both endpoints.

**Features:** streaming, tools, OpenAI-compatible API, live model list, reasoning content


## Custom (OpenAI-Compatible)

Use for Ollama, LM Studio, LocalAI, Groq, or any OpenAI-compatible API.

```toml
# config.toml
[providers.custom.lm_studio]
enabled = true
base_url = "http://localhost:1234/v1"
default_model = "qwen2.5-coder-7b-instruct"
models = ["qwen2.5-coder-7b-instruct", "llama-3-8B"]
```

> **Local LLMs:** No API key needed, just set `base_url` and `default_model`.
>
> **Remote APIs (Groq, etc.):** Add the key in `keys.toml`:
> ```toml
> [providers.custom.groq]
> api_key = "your-api-key"
> ```

### Multiple Custom Providers

Define as many as you need with different names:

```toml
[providers.custom.lm_studio]
enabled = true
base_url = "http://localhost:1234/v1"
default_model = "qwen2.5-coder-7b-instruct"

[providers.custom.ollama]
enabled = false
base_url = "http://localhost:11434/v1"
default_model = "mistral"
```

### Naming Custom Providers

The provider's name is the **last segment of its config section**, nothing else.

```toml
[providers.custom.moonshotai]   # the name is "moonshotai"
```

When you reference the provider anywhere (`self_improvement_provider`, `subagent_provider`, `plan_provider`, `execute_provider`, or the `/models` command) use that bare name:

```toml
[agent]
self_improvement_provider = "moonshotai"        # ✅ bare section name
self_improvement_model = "kimi-k2.5"            # the model has its own key
```

Never carry the `custom` path into the reference, and never append the model:

```toml
self_improvement_provider = "custom:moonshotai"    # ❌ RSI silently never runs
self_improvement_provider = "custom.moonshotai"    # ❌ same
self_improvement_provider = "moonshotai/kimi-k2.5" # ❌ provider-only, model goes in self_improvement_model
```

> **Real case:** a user with `[providers.custom.glm-53-max]` set `self_improvement_provider = "custom:glm-53-max"` and RSI silently never ran, Mission Control showed thousands of tool events recorded and unprocessed. Dropping the `custom:` prefix fixed it instantly ([opencrabs#1314](https://github.com/opencrabs/opencrabs/issues/1314)). Current main auto-corrects misspelled values with a warning; released versions fail silently, so spell it bare.

### Free Prototyping with NVIDIA API

[Kimi K2.5](https://build.nvidia.com/moonshotai/kimi-k2.5) is available for free on the NVIDIA API Catalog, no billing required.

```toml
# config.toml
[providers.custom.nvidia]
enabled = true
base_url = "https://integrate.api.nvidia.com/v1"
default_model = "moonshotai/kimi-k2.5"
```

```toml
# keys.toml
[providers.custom.nvidia]
api_key = "nvapi-..."
```

## Fallback Provider Chain

Configure automatic failover when the primary provider fails (rate limits, outages, errors). Fallbacks are tried in order until one succeeds.

```toml
# config.toml
[providers.fallback]
enabled = true
providers = ["openrouter", "anthropic"]  # Tried in order on failure
```

Each fallback provider must have its API key configured in `keys.toml`. Both `complete()` and `stream()` calls are retried transparently, no changes needed downstream.

Single fallback shorthand:

```toml
[providers.fallback]
enabled = true
provider = "openrouter"
```

Or just ask your Crab: *"Set up fallback providers using openrouter and anthropic"*, it will configure `config.toml` for you at runtime.

### Quota Circuit Breaker (v0.3.79)

Fallback chains now know the difference between a provider that is momentarily busy and one that is genuinely out of quota.

- **Hard quota exhaustion** (your account has run out of credits/allowance for the period) trips a TTL circuit breaker on that provider. While the breaker is open the provider is **skipped in fallback walks** instead of being retried pointlessly, so failover lands on a provider that can actually serve the request.
- **Transient throttles** (short rate limits, 429s that clear in seconds) are classified separately and retried as before, they never trip the breaker.
- When **every provider in the chain is exhausted**, the failure is reported explicitly as chain exhaustion rather than surfacing as a confusing error from the last provider tried.
- **Retry notices** now name the provider and model being retried, and state whether the cause was a quota exhaustion or a transient throttle.

No configuration needed: it is always on whenever a fallback chain is configured.

### Fallback Improvements (v0.3.80)

- **Proactive setup suggestion** (#1008): when no fallback chain is configured, OpenCrabs makes a one-shot suggestion to set one up, ask once, answered once, never nags again.
- **HTTP 400 failover guidance** (#1006, #1007): a 400-series failure now carries guidance on what to check, and a chain-exhaustion ledger records why each provider in the chain was ruled out.
- **Fallback provenance in streaming logs** (#969): each streamed call logs which provider actually served it, so retries and failovers are visible per call.
- **Codex CLI failures route into retry and fallback** (#1004, #1005) instead of surfacing as terminal errors.

## Provider Configuration Fields

Every provider section (`[providers.<name>]`) supports these fields:

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | `true` | Enable/disable the provider |
| `api_key` | string | `None` | API key (loaded from `keys.toml` or env) |
| `base_url` | string | `None` | API base URL override (for custom/local providers) |
| `default_model` | string | `None` | Default model for this provider |
| `models` | array | `[]` | Available models (updated at runtime via `/models`) |
| `force_default` | bool | `false` | Push this provider's default pair to all sessions on reload (see [Force Default Provider](#force-default-provider)) |
| `vision_model` | string | `None` | Vision-capable model override (swaps in when images are present) |
| `generation_model` | string | `None` | Image generation model override (wins over global `image.generation.model`) |
| `context_window` | u32 | `None` | Context window size in tokens (for auto-compaction). Ignored on the built-in OpenCrabs provider, whose window is fixed at 200k |
| `endpoint_type` | string | `None` | Endpoint type for providers with multiple API modes (e.g. z.ai: `"api"` or `"coding"`) |
| `plan` | string | `None` | Subscription tier (Kimi Code: `"moderato"`, `"allegretto"`, `"allegro"`, `"vivace"`) |
| `reasoning_effort` | string | `None` | Reasoning control (e.g. `"max"` for Kimi K3, `"on"`/`"off"` for K2.x) |
| `voice` | string | `None` | TTS voice name (e.g. `"echo"`) |
| `model` | string | `None` | TTS model override (e.g. `"gpt-4o-mini-tts"`) |
| `enable_thinking` | bool | `None` | Thinking mode for reasoning-capable models (Qwen, local providers) |

**Example: full provider config:**

```toml
# config.toml
[providers.minimax]
enabled = true
default_model = "MiniMax-M2.7"
vision_model = "MiniMax-Text-01"
generation_model = "MiniMax-Image-01"
context_window = 200000
force_default = true
```

## Force Default Provider

When you set `force_default = true` on a provider section, a config reload pushes that provider's default pair (provider + model) to **every non-archived session**, overriding their stored pairs. This enforces your chosen default across all active sessions instead of just new ones.

```toml
# config.toml
[providers.minimax]
enabled = true
default_model = "MiniMax-M2.7"
force_default = true  # Pushes MiniMax M2.7 to all sessions on reload
```

**How it works:**
- Only fires when the flagged section is the **active default provider** (set via `/models` or `default_provider` in `[agent]`)
- Archived sessions are never touched
- Sessions already on the target pair are skipped
- Without this flag: defaults apply to new sessions only, existing sessions keep their own provider/model

**Use case:** You want to roll out a new default model (e.g. MiniMax M2.7) across all active sessions immediately, not just new ones. Set `force_default = true` on the MiniMax section, reload config, and every session switches to M2.7.

**Note:** This only reinforces the default, it doesn't override a different active provider. If a session is using Anthropic Claude and you set `force_default = true` on MiniMax, the Claude session stays on Claude (because MiniMax isn't the active default).

## Vision Model

When your default chat model doesn't support vision, set `vision_model` to a vision-capable model on the same provider. This registers a vision tool that the agent can call, it sends the image to the vision model, gets a description back, and the chat model uses that context to reply.

```toml
# config.toml
[providers.minimax]
enabled = true
default_model = "MiniMax-M2.5"
vision_model = "MiniMax-Text-01"  # Agent calls vision tool → this model describes image → M2.5 replies
```

```toml
[providers.openai]
enabled = true
default_model = "gpt-5-nano"
vision_model = "gpt-5-nano"
```

MiniMax auto-configures `vision_model = "MiniMax-Text-01"` on first run. You can also ask your Crab to set it up: *"Configure vision model for MiniMax"*, it will update `config.toml` at runtime.

This is separate from the [Gemini image tools](./images.md) which provide dedicated `generate_image` and `analyze_image` tools.

## Per-Session Providers

Each session remembers its provider and model. Switch to Claude in one session, Gemini in another, switching sessions restores the provider automatically.

## Model Picker (v0.3.75)

The `/models` picker got a real overhaul:

- **Opus 5 surfaced**: Opus 5 now shows up via a third parallel list plus state discovery.
- **Newest-first ordering**: models are ordered newest-first instead of by merge order.
- **claude-cli model discovery**: the claude-cli model list is discovered live from the CLI instead of being hardcoded, with every claude-cli row formatted consistently.
- **Reconciled against live inventory**: the picker reconciles against the live model inventory so it never offers a model that isn't actually available.

## Natural Model Naming (v0.3.76)

The `/models` command now accepts **colloquial model names**. Instead of typing the exact model ID, you can say it the way people talk:

```
/models opus        → claude-opus-4-8
/models sonnet      → claude-sonnet-4-20250514
/models gpt5        → gpt-5
/models qwen max    → qwen3.8-max-preview
```

The resolver normalizes spacing, case, and common abbreviations before matching against the provider's model list. Exact matches still take priority.

## Plan/Execute Provider Routing (v0.3.76)

The `/plan` and `/execute` commands can now route onto **their own dedicated provider and model**, separate from the main chat provider:

```toml
# config.toml
[agent]
plan_provider = "anthropic"
plan_model = "claude-opus-4-8"
execute_provider = "openrouter"
execute_model = "anthropic/claude-sonnet-4"
```

This lets you use a stronger model for planning (where reasoning quality matters most) and a faster/cheaper model for execution (where throughput matters). If unset, both fall back to the session's active provider.
