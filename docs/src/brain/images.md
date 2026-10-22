# Image Generation & Vision

OpenCrabs supports **image generation** (text-to-image and img2img) and **vision analysis** (image-to-text). Vision works through two paths, pick whichever fits your provider setup.

## Vision: Two Paths

### Path A: `vision_model` on Your Active Provider (Preferred)

Set `vision_model = "<model>"` inside the provider block you're already using. Works for **every** built-in and custom provider. No second API key needed, the agent calls `analyze_image` against the vision model on the **same provider endpoint**.

```toml
# keys.toml
[providers.openrouter]
api_key = "sk-or-..."

# config.toml
[providers.openrouter]
model = "anthropic/claude-sonnet-4"
vision_model = "google/gemini-2.5-flash"   # ← any vision-capable model on the same endpoint
```

When a user sends an image and the chat model can't handle it natively, the agent routes the image through `vision_model`, gets a text description back, and replies with that context.

**Example:** User sends an image while you're on MiniMax M2.5 (no native vision). The agent calls the vision tool, which sends the image to `MiniMax-Text-01` (or any model you set), gets the description, and M2.5 replies using that context.

**Why this is preferred:**
- Single API key, single billing account
- Works on any OpenAI-compatible endpoint (OpenRouter, Ollama, LM Studio, vLLM, Groq, custom)
- No extra onboarding step: just add one line to your existing provider block

### Path B: Gemini Global Fallback

Use this only when your active provider has **no** vision-capable model. Gemini acts as a dedicated vision+image backend, independent of your chat provider.

```toml
# keys.toml
[image]
api_key = "AIza..."    # ← MUST go here. See gotcha below.

# config.toml
[image.generation]
enabled = true
model = "gemini-3.1-flash-image-preview"

[image.vision]
enabled = true
model = "gemini-3.1-flash-image-preview"
```

Get a free API key from [aistudio.google.com](https://aistudio.google.com). Configure interactively with `/onboard:image`.

### ⚠️ Gotcha: `#[serde(skip)]` on `[image.vision] api_key`

The `api_key` field under `[image.vision]` in `config.toml` is silently ignored: it's marked `#[serde(skip)]` in the source. **Always put the Gemini key in `keys.toml` under `[image]`**, never in `config.toml`. If vision reports as unavailable despite a key being set, this is almost always the cause.

### Diagnostic: Why Is Vision Unavailable?

`is_vision_available` logs the exact reason at INFO level with `target=vision`. Search your daily log:

```bash
grep 'target=vision' ~/.opencrabs/logs/opencrabs.$(date -u +%Y-%m-%d)
```

Common causes surfaced:
- Missing `vision_model` on active provider (the only field actually required since v0.3.64)
- Missing `api_key` for that provider
- Missing Gemini `[image] api_key` in `keys.toml`
- Key placed in `config.toml` where `#[serde(skip)]` drops it

> **v0.3.64 change:** setting `vision_model` on your active provider is now sufficient to enable vision. The `enabled` flag under `[image.vision]` is no longer required. The vision roll-through tries each provider endpoint and falls back to Gemini last.

## Agent Tools

When vision or image generation is enabled, these tools become available:

| Tool | Description |
|------|-------------|
| `generate_image` | Generate an image from a text prompt: saves to `~/.opencrabs/images/`; walks a provider chain with vision parity, so generation falls back the same way vision does (v0.5.4, #1672) |
| `analyze_image` | Analyze an image file or URL via the active vision path (Path A provider or Gemini fallback) |

**Example prompts:**
- *"Generate a pixel art crab logo"*: agent calls `generate_image`, returns file path
- *"What's in this image: /tmp/screenshot.png"*, agent calls `analyze_image`

## img2img: Edit Images with Context

`generate_image` accepts an optional `image` parameter (local file path or HTTPS URL). When provided, the model modifies, restyles, or composites onto that image instead of generating from scratch.

```text
User: "Make this logo darker and add a border"
Agent: generate_image(prompt="dark background with thin white border", image="/tmp/logo.png")
```

- **Gemini backend**: full img2img support via `inlineData`
- **OpenAI-shaped backends**: reject with a clear error pointing at Gemini (img2img not supported)

Use cases: replace elements, restyle photos, composite logos onto backgrounds, modify user-uploaded images in-place.

## Incoming Images from Channels

When a user sends an image from any channel, it arrives as `<<IMG:/tmp/path>>` in the message. The file is already downloaded, the agent can:

- See it directly (if the chat model supports vision natively)
- Pass the path to `analyze_image` for vision processing
- Use the path in `bash` commands or any tool that accepts file paths
- Reference it in replies with `<<IMG:path>>` to forward to channels

## analyze_video: Frame-Extraction Fallback (v0.3.36)

The `analyze_video` tool sends the full video to Gemini's video API for multimodal analysis. When that fails (network error, upload timeout, unsupported format), the agent falls back to **ffmpeg frame extraction**:

1. Extract frames at **1 fps** via ffmpeg
2. **Cap at 30 frames** (one frame per second for the first 30 seconds)
3. Send each frame to Gemini vision via `analyze_image`
4. Combine per-frame descriptions into a **chronological summary**

This means `analyze_video` works even when the video API is unreachable, as long as ffmpeg is installed and at least one vision path (Path A or Path B) is configured. The fallback activates automatically. No user configuration needed.

## Model Choices

- **Path A**: any vision-capable model on your active provider. On OpenRouter: `google/gemini-2.5-flash`, `anthropic/claude-sonnet-4`, `openai/gpt-4o`. On Ollama: `llava`, `bakllava`. On custom endpoints: whatever the server offers.
- **Path B**: `gemini-3.1-flash-image-preview` handles both vision input and image output in a single request.
