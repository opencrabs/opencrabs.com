# Memory System

OpenCrabs uses a 3-tier memory system for persistent context across sessions.

## Memory Tiers

### 1. Daily Notes (`memory/YYYY-MM-DD.md`)

Automatic daily files for session-specific observations:

```
~/.opencrabs/memory/2026-03-07.md
```

The agent writes here during conversations: new integrations, bugs fixed, decisions made, server changes.

### 2. Long-term Memory (`MEMORY.md`)

Curated knowledge that persists across all sessions:
- Server details, SSH access, credentials locations
- User preferences and workflows
- Integration configurations
- Lessons learned from debugging

### 3. Semantic Search (`session_search`)

Full-text search across all past sessions stored in SQLite. The agent can query:
- Previous conversations
- Tool execution history
- Past decisions and context

## Memory Search

The agent uses `session_search` for fast memory lookups (~500 tokens) instead of reading full memory files (~15K tokens). This is the primary recall mechanism.

### Chunk-Hash Caching (v0.3.82)

Embedding is skipped for chunks whose content has not changed (#1107). Each chunk carries a hash; on write, only chunks whose hash differs from the stored one are re-embedded. Editing one line of a large brain file no longer re-embeds the whole document.

### Scoped Search (v0.3.80)

`memory_search` takes a `scope` that picks which corpus to search (#1020):

| Scope | Corpus | Use it for |
|-------|--------|------------|
| `memory` (default) | Daily logs | History: what happened, when, what was decided |
| `brain` | Brain files (SOUL, USER, AGENTS, TOOLS, CODE, SECURITY, MEMORY, BOOT, HEARTBEAT) | Rules and policy: does a rule about this ALREADY exist, and which file owns it |
| `all` | Both | "Have I ever written about this anywhere" |

Picking the wrong scope is the usual reason a search comes back with nothing useful. Daily logs outnumber brain files and reuse the same words for unrelated things, so searching `memory` for a rule usually fails: history outranks policy and you get confident irrelevant hits. Search `brain` before appending any rule: a hit tells you WHICH file owns it, and then `load_brain_file` with a `query` reads the whole section.

### Chunked Retrieval and Index-on-Write (v0.3.80)

Memory retrieval was rebuilt around chunking (#998-1002, #1018):

- **Documents are chunked before embedding**, so chunks past the first are searchable. Previously a long document effectively hid everything after its opening.
- **Lexical hits are narrowed to the matching chunk**, so keyword results point at the relevant slice instead of the whole document.
- **The index updates on write**: memory is searchable immediately, not from a boot-time snapshot. Brain files are indexed into the brain collection on write too.
- **The store resolves per profile**, so profiles never share or cache each other's memory index.
- **The chunker is multi-byte safe**: no more panics on UTF-8 boundaries.
- **MEMORY.md recall is ranked with BM25** instead of shared-word counts (#996), and recall folds Latin diacritics so accented queries match.

### In-tree Memory Store (v0.3.81)

The qmd binary dependency is gone (#1032). OpenCrabs now owns its memory store in-tree (`src/memory/`): SQLite FTS5 for lexical search plus vector embeddings in one store, resolved per profile. One fewer external binary to install or break, and the store schema ships with the binary.

### External Index Paths (v0.3.81)

Memory search can read indexes that live outside the profile directory (#1051, #1055):

```toml
[memory]
# Each entry is a bare path string or a { path, pattern } table.
# Relative paths resolve against the OpenCrabs home (not the session cwd),
# so the index stays stable across /cd and profile switches.
extra_paths = [
  "~/knowledge/product-docs",
  { path = "~/knowledge/website-mirror", pattern = "docs/**/*.md" },
]
```

| Field | Default | Description |
|-------|---------|-------------|
| `extra_paths` | `[]` | Paths indexed into the `external` collection. Bare strings or `{ path, pattern }` tables |
| `exclude` | VCS/build/secret globs | Glob excludes applied to external indexing, global across all entries (`.git`, `node_modules`, `.env*`, `*.pem`, `.ssh/**`, `*credential*`, ...) |
| `external_allowed_in_shared` | `false` | Shared/group sessions are **default-deny**: external content stays out of `memory_search` results unless you opt in |
| `sweep_interval_secs` | `300` | Seconds between freshness sweeps that pick up added/removed files. Modified files are also caught lazily at search time |

Indexed files are searchable within one sweep interval (~5 min), no restart needed. Results come back as ranked excerpts with file paths via `memory_search scope="external"`; the agent can then read the full file for verbatim quotes.

**Gotchas:** UTF-8 text only (binary files are skipped with a warning); HTML files index as raw markup, so mirror sites to `.md` first; nested roots and symlinks are skipped by design; test external scope from an owner session, group sessions return nothing unless you set `external_allowed_in_shared = true`.

#### Build a knowledge base from any docs site

`web_scrape` sitemap mode pairs with `extra_paths` into a complete, zero-cost pipeline: scrape a site to markdown, index it, then ask questions answered verbatim from those docs (see [Web Scraping](./web-scrape.md)):

```toml
[memory]
extra_paths = ["scrapes/docs.example.com"]
```

1. `web_scrape url="https://docs.example.com" mode=sitemap export=true`: every sitemap page saved as clean markdown under `scrapes/`, local extraction, no AI/API cost (hard cap: 100 pages per scrape)
2. Point `extra_paths` at the scrapes directory (relative paths resolve against the OpenCrabs home)
3. Ask anything: answers are grounded in and quoted from the indexed docs

### Structural Code Search Upgrades (v0.5.1)

The code-graph answers (`who calls X`, `what does X call`, `impact of X`) got four upgrades:

- **Truncation is now visible.** Caller sets in real repos are unbounded (5,000+ call sites
  exist), so every structural listing that had to cut ends with a marker line (
  *"… and N more callers of X (re-query with higher n or offset)"*) and sets beyond 500
  collapse into a per-file rollup instead of one-liners. Nothing is silently chopped anymore.
- **Offset pagination.** `memory_search` accepts an `offset` alongside `n` for structural
  listings, so the full set can be walked page by page. Ranked text/vector results stay
  rank-truncated: the window applies to listings only.
- **Depth-2 impact chains.** `impact of X` now walks two hops over the call-edge graph,
  surfacing the callers-of-callers that a direct query misses.
- **Cleaner hits.** External code hits render **repo-relative paths** instead of absolute
  machine paths, and every `scope=all` hit carries a **corpus provenance tag** (which index
  the fact came from), so a memory answer and a code-graph answer can be told apart (#89).

### Embedding Hardening (v0.3.81)

Embedding API calls now carry **timeouts**, a **vector gate** (malformed or empty vectors never reach the store), and **non-blocking writes** (embedding failures never stall the turn) (#1062). `/doctor` reports **embedding health**, and a sweep finds **unembedded documents** and backfills them (#1069, #1067).

### Embedding Modes

OpenCrabs supports three embedding configurations:

1. **Local GGUF** (default): downloads a 300MB embedding model and runs it locally via llama.cpp
2. **OpenAI-compatible API**: configure external embedding providers (OpenAI `text-embedding-3-small`, Ollama `nomic-embed-text`, Jina, LM Studio, or any `/v1/embeddings` endpoint) via `[memory.embedding]` config with `url`, `model`, `api_key`, `dimensions`
3. **FTS5-only**: pure keyword search with zero RAM overhead. Set `[memory] vector_enabled = false`. Auto-detects VPS environments and configures automatically

## Context Compaction

When context reaches ~80% capacity, OpenCrabs automatically compacts:

1. Summarizes the conversation so far into a comprehensive continuation document
2. Clears old messages from context
3. Continues with the summary as context

Manual compaction: type `/compact` in chat.

## Auto-Save Triggers

The agent saves to memory when:
- New integrations are connected
- Server/infrastructure changes occur
- Bugs are found and fixed
- New tools are configured
- Credentials are rotated
- Architecture decisions are made
- You say "remember this"
- Errors take >5 minutes to debug

## Proactive Memory Surfacing (v0.3.76)

The agent now **surfaces relevant memory without being asked**. When a conversation topic matches something stored in MEMORY.md or daily notes, the relevant context is loaded and injected automatically. Previously, memory was only recalled when the agent explicitly called `memory_search` or `load_brain_file`. Now the system proactively checks for relevant context on each turn, so the agent brings up past decisions, server details, or preferences that apply to the current conversation without you having to say "check your memory."

## Confidence, Decay and Pruning (v0.5.3)

MEMORY.md facts carry **confidence** and **decay** over time (#1643): a belief's trust
drops as it goes unused and unverified, so stale claims stop outranking fresh ones when
memory is recalled. Decay is config-driven through the `[epistemic]` section.

Cold content is managed in two passes (#1657):

- **Archive pass**: cold MEMORY.md sections move to `memory/archive/` with a backup and
  reindex, instead of silently accumulating forever.
- **`opencrabs memory prune`**: a dry-run preview (the default) of cold beliefs (key,
  confidence, hits, age) and cold sections (heading, bytes). `--apply` archives the
  sections first, then sweeps the remaining cold beliefs. `--max-age-days <N>` overrides
  the cold threshold (default: 90).

## Brain Files

See [Brain Files](./brain-files.md) for the full list of files the agent reads on startup.
