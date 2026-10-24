# Supported AI Providers

OpenCrabs supports 18 built-in providers + Custom OpenAI Compatible. Switch between them at any time via `/models` in the TUI or any channel.

| Provider | Auth | Models | Streaming | Tools | Notes |
|----------|------|--------|-----------|-------|-------|
| [OpenCrabs (built in)](./providers.md#opencrabs-built-in) | None: automatic install token | deepseek-v4.1-flash (200k ctx, multimodal) | Yes | Yes | Free, no account, no key. Set up on first run. Also serves vision, image generation and voice input |
| [Anthropic Claude](./providers.md#anthropic-claude) | API key | Claude Opus 4.6, Sonnet 4.5, Haiku 4.5 | Yes | Yes | Extended thinking, 200K context |
| [OpenAI](./providers.md#openai) | API key | GPT-5 Turbo, GPT-5, o3/o4-mini | Yes | Yes | Models fetched live |
| [GitHub Copilot](./providers.md#github-copilot) | OAuth | GPT-4o, Claude Sonnet 4+ | Yes | Yes | Uses your Copilot subscription: no API charges |
| [OpenRouter](./providers.md#openrouter--400-models) | API key | 400+ models | Yes | Yes | Free models available. Reasoning output support (Qwen 3.6 Plus, etc.) |
| [Google Gemini](./providers.md#google-gemini) | API key | Gemini 2.5 Flash, 2.0, 1.5 Pro | Yes | Yes | 1M+ context, vision, image generation |
| [MiniMax](./providers.md#minimax) | API key | M2.7, M2.5, M2.1, Text-01 | Yes | Yes | Competitive pricing, auto-configured vision |
| [Moonshot Kimi](./providers.md#moonshot-kimi) | API key | k3, kimi-for-coding, kimi-for-coding-highspeed | Yes | Yes | API plan + Coding plan (Kimi token subscription), reasoning content on both |
| [Xiaomi MiMo](./providers.md#xiaomi-mimo) | API key (auto) | MiMo V2.5 Pro, MiMo V2 Pro, MiMo V2.5, MiMo V2 Omni, MiMo V2 Flash | Yes | Yes | Default provider for new users. Keyless mode with automatic token provisioning during collab windows. 30 models across 5 tiers. |
| [z.ai GLM](./providers.md#zai-zhipu-ai) | API key | GLM-4.5 through GLM-5 Turbo | Yes | Yes | General API + Coding API endpoints |
| [Claude CLI](./providers.md#claude-cli) | CLI auth | Via `claude` binary | Yes | Yes | Uses your Claude Code subscription |
| [Codex](./providers.md#codex-oauth) | OAuth (PKCE) | GPT-5.5, GPT-5.4, GPT-5.3-Codex | Yes | Yes | Native Codex subscription auth via device-code PKCE, no CLI, no API key |
| [Codex CLI](./providers.md#codex-cli) | CLI auth | Via `@openai/codex` binary | Yes | Yes | Uses your Codex subscription: free tier available |
| [Command Code CLI](./providers.md#command-code-cli) | CLI auth | Via `command-code` binary | No | Yes | Uses your Command Code account, no API key stored |
| [Qwen/DashScope](./providers.md#qwen-dashscope) | API key | qwen3.6-plus (default) | Yes | Yes | DashScope API-key provider (replaced OAuth rotation). Local model tool-call extraction from text (bare JSON, Claude-style XML, Qwen formats). Prompt caching via `cache_control`, rate limit retry with exponential backoff |
| [Ollama](./providers.md#ollama) | Optional | Any Ollama model | Yes | Yes | Native local provider: run any model via Ollama API |
| [OpenCode](./providers.md#opencode) | None | Any OpenAI-compatible model | Yes | Yes | Non-CLI OpenAI-compatible provider |
| [OpenCode CLI](./providers.md#opencode-cli) | None | Free models (Mimo, etc.) | Yes | Yes | Free: no API key or subscription needed |
| [Custom](./providers.md#custom-openai-compatible) | Optional | Any | Yes | Yes | LM Studio, Groq, NVIDIA, any OpenAI-compatible API |

## How It Works

- **One provider active at a time** per session: switch with `/models`
- **Per-session isolation**: each session remembers its own provider and model. Changing provider in the TUI does not affect other active sessions (Telegram, Discord, Slack)
- **Fallback chain**: configure automatic failover when the primary provider goes down
- **Models fetched live**: no binary update needed when providers add new models
- **Function calling detection**: OpenCrabs detects when a model doesn't support tool use and warns you with a model switch suggestion, rather than silently failing
- **`tool_choice: "auto"`**: sent automatically for OpenAI-compatible providers when tools are active, enabling function calling on models that require explicit opt-in
- **`vision_model` works on ANY provider**: add `vision_model = "..."` to any built-in or custom provider block and the agent routes incoming images through that model on the same endpoint. No second API key, no Gemini dependency. See [Image Generation & Vision](./images.md) for the full two-path setup

## Custom Provider Onboarding (v0.3.24)

Adding a custom OpenAI-compatible provider is now smoother:

- **Paste-by-default**: `Ctrl+V` / `Cmd+V` on the API key field pastes immediately, no need to tab into the field first
- **Enter-to-load**: type a model name not in the fetched list and press Enter, it's added to the list and selected
- **Field refresh**: saved values (base URL, API key, model list) appear instantly without restarting the dialog

## Provider Registry (v0.3.34)

All provider resolution now routes through a single registry source of truth: no more hardcoded if-else ladders scattered across the codebase. The registry correctly enforces `api_key` requirements for API providers (Anthropic, OpenAI, GitHub Copilot, Gemini, OpenRouter, MiniMax), so resolution skips them cleanly when keys are missing instead of silently falling back. Adding a new provider is now a one-file change.

## Retry & Fallback Overhaul (v0.3.36)

The provider retry and fallback system was rebuilt for reliability:

- **Patient backoff**: defaults changed from 100ms hammering to 1s / 2s / 4s / 8s, giving rate-limited endpoints time to recover
- **In-place rate limit retry**: hits 3 retries on the same provider before falling through the fallback chain, preventing unnecessary provider switches
- **Full fallback chain on any HTTP error**: previously only 5xx/429 triggered fallback; now any HTTP error walks the chain
- **Fail fast on hard-down endpoints**: DNS failures and connection refused bail immediately instead of wasting retries
- **Retry events surfaced to user**: each retry shows as `RetryAttempt 1/3 - rate_limit_exceeded` so you always know what's happening
- **Transient 4xx HTML pages retried**: Cloudflare/nginx error pages that look like 4xx but are actually transient infra issues are now retried

## Qwen Cache Auto-Enable (v0.3.30)

Custom providers targeting Qwen-shaped endpoints (base URLs containing `dashscope`, `aliyun`, `aliyuncs`, `dialagram`, or models prefixed with `qwen-*`) automatically get ephemeral `cache_control` markers on the system prompt, last streaming message, and last tool call. Zero-config cost savings for Qwen custom providers, no API key or flag needed.

## Qwen Tool-Call Leak Strip (v0.3.35)

Qwen models sometimes emit bare JSON tool-call objects like `{"name":"bash","arguments":{"command":"ls"}}` directly in the content text instead of through the proper tool-call API. A new `bare_tool_call_extractor` detects and strips these leaks so they don't appear as raw JSON in the chat output.

## `/models` Picker (v0.3.30)

The `/models` command now surfaces **every known provider** including unconfigured ones, marked with a 🔒 lock icon and setup help text. This helps users discover available providers without needing to know which ones need API keys. Custom providers with no configured models show a helpful empty-state message instead of an inert button.

## OpenRouter Reasoning

For models that support extended reasoning (e.g. Qwen 3.6 Plus), OpenCrabs sends `include_reasoning: true` automatically when using OpenRouter. Thinking/reasoning output is displayed in collapsible sections:

```
▶ Thinking... (click to expand)
  The user wants to refactor...
```

Reasoning text wraps to screen width instead of truncating.

See [Provider Setup](./providers.md) for configuration details and API key setup.
