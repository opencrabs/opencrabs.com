# OpenCrabs Translation Glossary (LOCKED)

Terms that NEVER get translated, in any locale. They are product vocabulary.

| Term | Rule |
|------|------|
| OpenCrabs | Never translate, never transliterate (except Cyrillic: ОпенКрэбс is forbidden too, keep Latin "OpenCrabs") |
| crab | Product metaphor: keep English in technical/product context |
| brain / brain files | Keep English (SOUL.md, MEMORY.md etc. are "brain files") |
| channel | Keep English (Telegram/Discord/Slack are "channels") |
| skill | Keep English (slash-command workflows) |
| session | Keep English |
| TUI / CLI | Keep as-is |
| RSI | Recursive Self-Improvement: translate the expansion, keep the acronym |
| Ralph / Ralph Loop | Keep English (proper noun) |
| OODA | Keep acronym, translate expansion |
| cron | Keep English |
| owner / bot owner | Keep "owner" in English |
| OpenCrabs | (repeat guard) always Latin script |

## Commands and code
- Slash commands are NEVER translated: `/onboard`, `/restart`, `/exit`, `/models`, `/usage`, `/compact`, `/evolve`, `/rebuild`
- Config keys, file paths, tool names: never translated (`config.toml`, `keys.toml`, `MEMORY.md`)
- Cargo commands, binary names: never translated

## Locale-specific rules
- **pt-PT**: European Portuguese (PT-PT), NEVER Brazilian. "ficheiro" not "arquivo", "utilizador" not "usuário", "ecrã" not "tela", "you (informal) = tu" for the product voice is acceptable; prefer "agent" → "agente", "landing page" stays English.
- **fr**: Standard French, "vous" form. Product metaphors keep English (brain, channel, skill).
- **ru**: Russian, formal "вы". Keep product terms Latin per table above.
- **id**: Bahasa Indonesia (not Malay). Keep product terms English per table above.

## Style
- Match the English tone: direct, technical, no fluff.
- Code blocks: only comments and string literals are extracted; translate those, keep code identical.
- Never translate content inside `{{#include}}` markers or URLs.
