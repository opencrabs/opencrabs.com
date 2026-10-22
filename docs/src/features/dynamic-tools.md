# Dynamic Tools

Define custom tools at runtime without recompiling. Tools are defined in `~/.opencrabs/tools.toml` and can be created, removed, and reloaded on the fly.

## Defining Tools

Create `~/.opencrabs/tools.toml`:

```toml
[[tools]]
name = "deploy"
description = "Deploy the application to production"
executor = "shell"
command = "cd {{project_dir}} && ./deploy.sh {{environment}}"

[[tools]]
name = "check-status"
description = "Check service health"
executor = "http"
method = "GET"
url = "https://api.example.com/health"
```

## Executors

| Executor | Description |
|----------|-------------|
| `shell` | Runs a shell command |
| `http` | Makes an HTTP request |

## Template Parameters

Use `{{param}}` syntax for dynamic values. The agent fills these in when calling the tool:

```toml
[[tools]]
name = "search-logs"
description = "Search application logs for a pattern"
executor = "shell"
command = "grep -r '{{pattern}}' /var/log/myapp/ --include='*.log' -l"
```

## Runtime Management

The `tool_manage` meta-tool lets the agent manage dynamic tools during a session:

- **Create**: Add a new tool definition
- **Remove**: Delete an existing dynamic tool
- **Reload**: Re-read `tools.toml` without restarting

Dynamic tools appear alongside built-in tools in the agent's tool list. Enable or disable individual tools without restarting the process.

## Per-Parameter Value Coercion (v0.3.24)

Dynamic tools defined in `tools.toml` can now handle empty-string or null parameters gracefully. When a parameter arrives as `""` or `null`, the engine substitutes a configured value before rendering the command template.

| Field | Purpose |
|-------|---------|
| `coerce_empty_to` | Substitute when parameter is `""` |
| `coerce_null_to` | Substitute when parameter is `null` |

```toml
[[tools]]
name = "deploy"
description = "Deploy to environment with optional verbose flag"
executor = "shell"
command = "cd {{project_dir}} && ./deploy.sh --env {{environment}} {{verbose}}"

[[tools.deploy.params]]
name = "verbose"
type = "string"
required = false
coerce_empty_to = "--quiet"
```

A shell tool with an optional `--verbose` flag no longer breaks when the parameter is omitted. The engine substitutes `--quiet` (or any configured default) instead of passing an empty string.

([#95](https://github.com/opencrabs/opencrabs/issues/95))

External contributions now enable `tools.toml` to be loaded in **run mode** and **agent mode** (not just the TUI). Previously, dynamic tools only worked in the interactive TUI session. Now they're available across all modes, allowing headless automation and scripted workflows to use custom tools.

([#79](https://github.com/opencrabs/opencrabs/issues/79): thanks @leshchenko)

## Shell Parameter Escaping (v0.3.35)

Dynamic shell tool parameters now properly escape single quotes, preventing command injection edge cases when user-supplied values contain quotes. Contributed by @leshchenko1979.
