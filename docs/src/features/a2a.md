# Agent-to-Agent (A2A) Protocol

OpenCrabs includes a built-in A2A gateway implementing the [A2A Protocol RC v1.0](https://google.github.io/A2A/) for peer-to-peer agent communication.

## Enabling

> **Legacy `[gateway]` name (v0.3.82):** this section was once spelled `[gateway]`. Both spellings now fold into one setting, and the legacy name is **rewritten to `[a2a]` on disk** so it survives a reload round trip instead of reappearing (#1116). Nothing to do by hand; an existing `[gateway]` block migrates itself on the next load.

```toml
# config.toml
[a2a]
enabled = true
bind = "127.0.0.1"   # Loopback only (default): use "0.0.0.0" to expose externally
port = 18790
# api_key = "your-secret"  # Optional Bearer token auth for incoming requests
# allowed_origins = ["http://localhost:3000"]  # CORS
```

### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `enabled` | `false` | Enable the A2A gateway |
| `bind` | `127.0.0.1` | Bind address: use `0.0.0.0` to accept external connections |
| `port` | `18790` | Gateway port |
| `api_key` | _(none)_ | Bearer token for authenticating incoming requests. If set, all JSON-RPC requests must include `Authorization: Bearer <key>` |
| `allowed_origins` | `[]` | CORS allowed origins: no cross-origin requests unless explicitly set |

## Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/.well-known/agent.json` | GET | Agent Card: discover capabilities (auto-populated from tool registry) |
| `/a2a/v1` | POST | JSON-RPC 2.0: `message/send`, `message/stream`, `tasks/get`, `tasks/cancel` |
| `/a2a/health` | GET | Health check |

### Methods

- **`message/send`**: Send a message to the agent, creates a task. Returns the task with result.
- **`message/stream`**: Same as `message/send` but returns an SSE stream with real-time status updates and artifact chunks as the agent works.
- **`tasks/get`**: Poll a task by ID to check status and retrieve results.
- **`tasks/cancel`**: Cancel a running task.

Active tasks are persisted to the database and restored on restart.

## The `a2a_send` Tool

The agent has a built-in `a2a_send` tool that lets it proactively communicate with remote A2A agents. This enables true bidirectional agent-to-agent communication.

**Actions:**

| Action | Description |
|--------|-------------|
| `discover` | Fetch a remote agent's Agent Card to see its capabilities and skills |
| `send` | Send a task to a remote agent and wait for the result |
| `get` | Poll a task by ID on a remote agent |
| `cancel` | Cancel a running task on a remote agent |

The agent can use this tool autonomously: for example, delegating subtasks to a specialized remote agent.

## Connecting Two Agents

### Example: VPS + Local Machine

**On VPS** (`~/.opencrabs/config.toml`):
```toml
[a2a]
enabled = true
bind = "0.0.0.0"
port = 18790
api_key = "your-shared-secret"
```

**On local machine** (`~/.opencrabs/config.toml`):
```toml
[a2a]
enabled = true
bind = "127.0.0.1"
port = 18790
```

### Connectivity Options

1. **SSH tunnel** (recommended): No ports to open, encrypted:
   ```bash
   # From local machine, tunnel VPS A2A to localhost:18791
   ssh -L 18791:127.0.0.1:18790 user@your-vps
   ```
   Local agent talks to `http://127.0.0.1:18791/a2a/v1`

2. **Direct**: Open port 18790 on VPS firewall. Simple but exposes the port. Always use `api_key` with this approach.

3. **Reverse proxy**: Nginx/Caddy on VPS with TLS + Bearer auth via `api_key`.

## Examples

```bash
# Discover the agent
curl http://127.0.0.1:18790/.well-known/agent.json | jq .

# Send a message (with Bearer auth)
curl -X POST http://127.0.0.1:18790/a2a/v1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-shared-secret" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"text": "What tools do you have?"}]
      }
    }
  }'

# Stream a task (SSE)
curl -N -X POST http://127.0.0.1:18790/a2a/v1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-shared-secret" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "message/stream",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"text": "Analyze the system status"}]
      }
    }
  }'

# Poll a task
curl -X POST http://127.0.0.1:18790/a2a/v1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-shared-secret" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tasks/get","params":{"id":"TASK_ID"}}'

# Cancel a task
curl -X POST http://127.0.0.1:18790/a2a/v1 \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-shared-secret" \
  -d '{"jsonrpc":"2.0","id":4,"method":"tasks/cancel","params":{"id":"TASK_ID"}}'

# Health check
curl http://127.0.0.1:18790/a2a/health | jq .
```

## Session Resume and Profile Addressing (v0.3.83)

A2A sessions **resume by context id** (#1159): a reconnecting client restores the session bound to its context instead of starting cold. The gateway is **profile-addressed** (#1161), so a multi-profile host serves the right profile's sessions, and the `profile_list` tool is registered for remote peers (#1160, #1161).

## Bee Colony Debate

Multi-agent structured debate via confidence-weighted voting (based on [ReConcile, ACL 2024](https://arxiv.org/abs/2309.13007)). Multiple "bee" agents argue across configurable rounds, enriched with knowledge context, then converge on a consensus answer.

## Security

- **Loopback only** by default: binds to `127.0.0.1`
- **Bearer auth**: set `api_key` to require `Authorization: Bearer <key>` on all JSON-RPC requests
- **CORS locked down**: no cross-origin requests unless `allowed_origins` is set
- For public exposure, use a reverse proxy with TLS + the `api_key` Bearer auth
