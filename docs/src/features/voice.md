# Voice (TTS & STT)

OpenCrabs supports text-to-speech and speech-to-text with five provider tiers: **Off**, **Groq** (API), **OpenAI-compatible** (any `/v1/audio` endpoint), **Voicebox** (self-hosted), or **Local** (on-device, zero cost).

## Quick Setup

Run `/onboard:voice` in the TUI to configure everything interactively. The voice screen has radio selectors for both STT and TTS, with fields shown/hidden based on the selected provider. API keys are wired to `keys.toml` automatically.

## Speech-to-Text (STT)

### Providers

| Provider | Engine | Cost | Latency | Setup |
|----------|--------|------|---------|-------|
| **Groq** | Whisper (`whisper-large-v3-turbo`) | Per-minute pricing | ~1s | API key in `keys.toml` |
| **OpenAI-compatible** | Any Whisper-compatible endpoint | Varies | ~1-3s | `stt_base_url` + `stt_model` + API key |
| **Voicebox** | Self-hosted open-source | Free | ~2-5s | `voicebox_stt_enabled=true` + `voicebox_stt_base_url` |
| **Local** | whisper.cpp (on-device) | Free | ~2-5s | Auto-downloads model |

### Local STT Models

| Model | Size | Quality | Speed |
|-------|------|---------|-------|
| `local-tiny` | ~75 MB | Good for short messages | Fastest |
| `local-base` | ~142 MB | Better accuracy | Fast |
| `local-small` | ~466 MB | High accuracy | Moderate |
| `local-medium` | ~1.5 GB | Best accuracy | Slower |

Models auto-download from HuggingFace to `~/.local/share/opencrabs/models/whisper/` on first use.

### Configuration

```toml
# config.toml
[voice]
stt_enabled = true
stt_mode = "local"              # "api" or "local"
local_stt_model = "local-tiny"  # local-tiny, local-base, local-small, local-medium
```

For API mode:
```toml
# keys.toml
[providers.stt.groq]
api_key = "your-groq-key"       # From console.groq.com
```

## Text-to-Speech (TTS)

### Providers

| Provider | Engine | Cost | Voices | Setup |
|----------|--------|------|--------|-------|
| **OpenAI** | `gpt-4o-mini-tts` | Per-character pricing | alloy, echo, fable, onyx, nova, shimmer | API key in `keys.toml` |
| **OpenAI-compatible** | Any `/v1/audio/speech` endpoint | Varies | Varies by server | `tts_base_url` + `tts_model` + `tts_voice` + API key |
| **Voicebox** | Self-hosted async `POST /generate` | Free | Configurable profiles | `voicebox_tts_enabled=true` + `voicebox_tts_base_url` + `voicebox_tts_profile_id` |
| **Local** | Piper (on-device) | Free | 6 voices | Auto-downloads model |

### Local TTS Voices (Piper)

| Voice | Description |
|-------|-------------|
| `ryan` | US Male (default) |
| `amy` | US Female |
| `lessac` | US Female |
| `kristin` | US Female |
| `joe` | US Male |
| `cori` | UK Female |

Models auto-download from HuggingFace to `~/.local/share/opencrabs/models/piper/`. A Python venv is created automatically for the Piper runtime.

### Configuration

```toml
# config.toml
[voice]
tts_enabled = true
tts_mode = "local"              # "api" or "local"
local_tts_voice = "ryan"        # ryan, amy, lessac, kristin, joe, cori
```

For API mode:
```toml
# config.toml
[voice]
tts_mode = "api"
tts_voice = "echo"              # OpenAI voice name
tts_model = "gpt-4o-mini-tts"   # OpenAI model
```

```toml
# keys.toml
[providers.tts.openai]
api_key = "your-openai-key"
```

## Full Configuration Reference

```toml
# config.toml
[voice]
# Speech-to-Text
stt_enabled = true
stt_mode = "groq"                 # "groq", "openai_compatible", "voicebox", "local"
local_stt_model = "local-tiny"    # local-tiny, local-base, local-small, local-medium
stt_base_url = "https://..."      # OpenAI-compatible STT endpoint
stt_model = "whisper-1"           # OpenAI-compatible STT model
voicebox_stt_enabled = false
voicebox_stt_base_url = "https://..."

# Text-to-Speech
tts_enabled = true
tts_mode = "openai"               # "openai", "openai_compatible", "voicebox", "local"
tts_voice = "echo"                # OpenAI TTS voice name
tts_model = "gpt-4o-mini-tts"     # OpenAI TTS model
local_tts_voice = "ryan"          # Local mode: Piper voice
tts_base_url = "https://..."      # OpenAI-compatible TTS endpoint
tts_model = "tts-1"               # OpenAI-compatible TTS model
voicebox_tts_enabled = false
voicebox_tts_base_url = "https://..."
voicebox_tts_profile_id = "profile-id"
```

```toml
# keys.toml
[providers.stt.groq]
api_key = "your-groq-key"

[providers.stt.openai_compatible]
api_key = "your-api-key"

[providers.tts.openai]
api_key = "your-openai-key"

[providers.tts.openai_compatible]
api_key = "your-api-key"
```

## OpenAI TTS Onboarding (v0.3.78)

The onboarding wizard (`/onboard:voice`) now includes a dedicated **OpenAI TTS voice selector** and API key field. When you select OpenAI as your TTS provider during onboarding:

1. A voice picker appears with all available OpenAI voices (alloy, echo, fable, onyx, nova, shimmer)
2. An API key field is shown if no key is configured yet
3. Selection is saved to `config.toml` (`tts_voice`) and `keys.toml` (API key) automatically

This removes the need to manually edit config files for the most common TTS setup path.

## How Voice Messages Work

When a voice message arrives on Telegram, WhatsApp, Discord, or Slack:

1. Audio is decoded (OGG/Opus or WAV)
2. Transcribed via STT (local whisper.cpp or Groq API)
3. Agent processes the text and generates a response
4. Response is converted to speech via TTS (local Piper or OpenAI API)
5. Audio is encoded as OGG/Opus and sent back as a voice message

Local mode handles everything on-device: no API calls, no cost, no data leaves your machine.

## Hardware Requirements

| Feature | CPU Requirement | Notes |
|---------|----------------|-------|
| Local STT (rwhisper) | AVX2 (Haswell 2013+) | Metal GPU on macOS Apple Silicon |
| Local TTS (Piper) | No restrictions | Tested on 2007 iMac: works on any x86/ARM |
| Local embeddings | AVX (Sandy Bridge 2011+) | Falls back to FTS-only search |

OpenCrabs detects CPU capabilities at runtime and hides unavailable options in the onboarding wizard. Local TTS (Piper) has no CPU limitations and should work on virtually any machine.

## Building Without Voice

Voice features are enabled by default. To build without them (smaller binary):

```bash
cargo build --release --no-default-features --features telegram,whatsapp,discord,slack,trello
```

Feature flags: `local-stt` (whisper.cpp), `local-tts` (Piper).

## Voice Dialog Key Handling (v0.3.81)

The voice settings dialog no longer **hides stored API keys** behind placeholder text or **overwrites them** when you save unrelated changes (#1039). What is in `keys.toml` stays there unless you explicitly replace it.
