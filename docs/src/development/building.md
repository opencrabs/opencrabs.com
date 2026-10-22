# Building from Source

## Prerequisites

- **Rust 1.94+** (stable, nightly not required)
- **SQLite3** development headers
- **OpenSSL** development headers (vendored by default)
- **pkg-config** (Linux/macOS)

### macOS

```bash
brew install sqlite3 pkg-config
```

### Ubuntu / Debian

```bash
sudo apt install build-essential pkg-config libsqlite3-dev libssl-dev
```

### Arch Linux

```bash
sudo pacman -S base-devel sqlite openssl pkg-config
```

## Clone and Build

```bash
git clone https://github.com/opencrabs/opencrabs.git
cd opencrabs
cargo build --release
```

The binary is at `target/release/opencrabs`.

## Feature Flags

OpenCrabs uses Cargo features to toggle channel support:

| Feature | Default | Description |
|---------|---------|-------------|
| `telegram` | Yes | Telegram bot via teloxide |
| `discord` | Yes | Discord bot via serenity |
| `slack` | Yes | Slack bot via slack-morphism |
| `whatsapp` | Yes | WhatsApp via whatsapp-rust |
| `trello` | Yes | Trello integration |
| `browser` | Yes | Headless Chrome automation via CDP |
| `profiling` | No | pprof flamegraphs (Unix only) |

Build with specific features:

```bash
# Minimal: TUI only, no channels
cargo build --release --no-default-features

# Only Telegram
cargo build --release --no-default-features --features telegram
```

## Release Profile

The release profile is optimized for size and speed:

```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
strip = true
panic = "abort"
```

There's also a `release-small` profile for minimal binary size:

```bash
cargo build --profile release-small
```

## Running Tests

```bash
cargo test --all-features
```

## Linting

Always use clippy with all features:

```bash
cargo clippy --all-features
```

## Self-Update

If you build from source, use `git pull && cargo build --release` instead of `/evolve`. The `/evolve` command downloads pre-built binaries from GitHub Releases.
