# Installation

Five ways to get OpenCrabs running.

## Quick install (Linux and macOS)

The fastest path. One command resolves the latest release, checks your platform, verifies the download against the published `SHA256SUMS`, installs `opencrabs` and the `rtk` helper, and installs the system libraries the Linux build needs.

```bash
curl -fsSL https://opencrabs.com/install.sh | bash
```

On Windows, use PowerShell:

```powershell
iwr -useb https://opencrabs.com/install.ps1 | iex
```

It installs to `/usr/local/bin` when that directory is writable, otherwise to `~/.local/bin`, and prints the `export PATH` line when the target is not on your PATH yet. It refuses to install when glibc is older than 2.38 (Ubuntu 24.04+, Debian 13+, Fedora 39+) or when the system uses musl, and it names the package to install instead of failing silently.

Flags go after `bash -s --` when you pipe the script:

| Flag | Effect |
|---|---|
| `--dry-run` | Print the plan, download nothing |
| `--version <tag>` | Install a specific release instead of the latest |
| `--dir <path>` | Install into a different directory |

Both scripts are plain shell and PowerShell, readable before you run them: [install.sh](https://opencrabs.com/install.sh) and [install.ps1](https://opencrabs.com/install.ps1).

## Option 1: Homebrew (recommended on macOS and Linux)

```bash
brew install opencrabs
```

OpenCrabs is in [homebrew-core](https://github.com/Homebrew/homebrew-core/blob/HEAD/Formula/o/opencrabs.rb). Homebrew ships prebuilt bottles for Apple Silicon Macs (macOS 15 and newer) and for Linux amd64 and arm64, so no Rust toolchain is needed there. On an Intel Mac, Homebrew builds it from source instead; use the `macos-amd64` release binary if you'd rather skip the build.

Run `opencrabs` after install. It goes straight to the chat with its built-in provider already live, so the crab greets you and you can start talking. Run `opencrabs --onboard` when you want the setup wizard.

## Option 2: Download Binary (all platforms)

Grab a pre-built binary from [GitHub Releases](https://github.com/opencrabs/opencrabs/releases).

### Linux (amd64)

```bash
sudo apt update && sudo apt install -y curl jq libgomp1
TAG=$(curl -sL https://api.github.com/repos/opencrabs/opencrabs/releases/latest | jq -r .tag_name)
curl -fsSL "https://github.com/opencrabs/opencrabs/releases/download/${TAG}/opencrabs-${TAG}-linux-amd64.tar.gz" | tar xz
./opencrabs
```

### Linux (arm64)

```bash
sudo apt update && sudo apt install -y curl jq libgomp1
TAG=$(curl -sL https://api.github.com/repos/opencrabs/opencrabs/releases/latest | jq -r .tag_name)
curl -fsSL "https://github.com/opencrabs/opencrabs/releases/download/${TAG}/opencrabs-${TAG}-linux-arm64.tar.gz" | tar xz
./opencrabs
```

### macOS (arm64 / Apple Silicon)

```bash
TAG=$(curl -sL https://api.github.com/repos/opencrabs/opencrabs/releases/latest | jq -r .tag_name)
curl -fsSL "https://github.com/opencrabs/opencrabs/releases/download/${TAG}/opencrabs-${TAG}-macos-arm64.tar.gz" | tar xz
./opencrabs
```

### Windows

```powershell
$tag = (Invoke-RestMethod https://api.github.com/repos/opencrabs/opencrabs/releases/latest).tag_name
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest "https://github.com/opencrabs/opencrabs/releases/download/$tag/opencrabs-$tag-windows-amd64.zip" -OutFile opencrabs.zip -UseBasicParsing
Expand-Archive opencrabs.zip -DestinationPath . -Force
.\opencrabs.exe
```

It goes straight to the chat with its built-in provider already live. Run `opencrabs --onboard` when you want the setup wizard.

> **Windows is a first-class platform (v0.5.3):** platform-aware shell handling, BOM/UTF-16
> file reads, and tilde/cwd fallbacks are built in, and CI builds Windows on every PR (#627).

> **Verify your download (optional):** every release publishes a `SHA256SUMS` file alongside the tarballs.

> **Terminal permissions required.** OpenCrabs reads/writes brain files, config, and project files. Your terminal app needs filesystem access or the OS will block operations.
>
> | OS | What to do |
> |---|---|
> | **macOS** | **System Settings → Privacy & Security → Full Disk Access** → toggle your terminal app ON (Alacritty, iTerm2, Terminal, etc.). If not listed, click "+" and add it from `/Applications/`. Without this, macOS repeatedly prompts "would like to access data from other apps". |
> | **Windows** | Run your terminal (Windows Terminal, PowerShell, cmd) **as Administrator** on first run, or grant the terminal **write access** to `%USERPROFILE%\.opencrabs\` and your project directories. Windows Defender may also prompt — click "Allow". |
> | **Linux** | Ensure your user owns `~/.opencrabs/` and project directories. On SELinux/AppArmor systems, the terminal process needs read/write access to those paths. Flatpak/Snap terminals may need `--filesystem=home` or equivalent permission. |

> `/rebuild` works even with pre-built binaries — it auto-clones the source to `~/.opencrabs/source/` on first use, then builds and hot-restarts.

## Option 3: Build from Source

Required for `/rebuild`, adding custom tools, or modifying the agent.

### Quick setup (recommended)

The setup script auto-detects your platform (macOS, Debian/Ubuntu, Fedora/RHEL, Arch) and installs all build dependencies + Rust:

```bash
# Install all dependencies
curl -fsSL https://raw.githubusercontent.com/opencrabs/opencrabs/main/src/scripts/setup.sh | bash

# Clone and build
git clone https://github.com/opencrabs/opencrabs.git
cd opencrabs
cargo build --release
./target/release/opencrabs
```

### Manual setup

If you prefer to install dependencies yourself:

- **Rust stable** — [Install Rust](https://rustup.rs/). Stable toolchain works since v0.2.85
- **An API key** from at least one supported provider
- **SQLite** (bundled via rusqlite)
- **macOS:** `brew install cmake pkg-config`
- **Debian/Ubuntu:** `sudo apt install build-essential pkg-config libssl-dev cmake`
- **Fedora/RHEL:** `sudo dnf install gcc gcc-c++ make pkg-config openssl-devel cmake`
- **Arch:** `sudo pacman -S base-devel pkg-config openssl cmake`

```bash
git clone https://github.com/opencrabs/opencrabs.git
cd opencrabs
cargo build --release
./target/release/opencrabs
```

> OpenCrabs uses `keys.toml` instead of `.env` for API keys. The onboarding wizard will help you set it up, or edit `~/.opencrabs/keys.toml` directly.

## Option 4: Docker

Run OpenCrabs in an isolated container. Build takes ~15min (Rust release + LTO).

```bash
git clone https://github.com/opencrabs/opencrabs.git
cd opencrabs
docker compose -f src/docker/compose.yml up --build
```

Config, workspace, and memory DB persist in a Docker volume across restarts. API keys in `keys.toml` are mounted into the container at runtime — never baked into the image.

## Autostart on Boot

Keep OpenCrabs running as a background daemon that starts with your system.

### Linux (systemd)

The binary writes the unit itself, so the file cannot drift from the version you are running:

```bash
opencrabs service install
```

That writes `~/.config/systemd/user/opencrabs.service` with `Type=simple` and `OOMPolicy=continue`, then reloads systemd. Start it with `opencrabs service start`: a user unit does not come back after a reboot unless lingering is enabled (see below), while a system-wide unit is enabled so it does. `OOMPolicy=continue` matters because the daemon is the parent of every tool call and cron child: without it, one child killed by the OOM killer stops the whole unit and takes every agent lane with it. The unit name carries the profile when you are not on `default`, as `opencrabs-<profile>.service`.

On a headless SSH session the per-user systemd bus is often missing, and the command fails with `Failed to connect to bus`. Enable lingering so the user service runs without a login, or install a system-wide unit instead:

```bash
sudo loginctl enable-linger $(whoami)
# or
sudo opencrabs service install
```

Manage the daemon with `opencrabs service start|stop|restart|status`, or with systemctl directly.

Check status: `systemctl --user status opencrabs` | Logs: `journalctl --user -u opencrabs -f`

### macOS (launchd)

```bash
cat > ~/Library/LaunchAgents/com.opencrabs.agent.plist << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.opencrabs.agent</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/opencrabs</string>
        <string>daemon</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/tmp/opencrabs.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/opencrabs.err</string>
</dict>
</plist>
EOF

launchctl load ~/Library/LaunchAgents/com.opencrabs.agent.plist
```

Update the path in `ProgramArguments` to match your install location.

### Windows (Task Scheduler)

1. `Win + R` → `taskschd.msc`
2. **Create Basic Task** → Name: `OpenCrabs`
3. Trigger: **When I log on**
4. Action: **Start a program** → `C:\Users\<you>\.cargo\bin\opencrabs.exe`, Arguments: `daemon`
5. In Properties > Settings, check **If the task fails, restart every 1 minute**

Or via PowerShell:

```powershell
$action = New-ScheduledTaskAction -Execute "$env:USERPROFILE\.cargo\bin\opencrabs.exe" -Argument "daemon"
$trigger = New-ScheduledTaskTrigger -AtLogon
$settings = New-ScheduledTaskSettingsSet -RestartCount 3 -RestartInterval (New-TimeSpan -Minutes 1)
Register-ScheduledTask -TaskName "OpenCrabs" -Action $action -Trigger $trigger -Settings $settings
```

## Updating

- **Homebrew users:** `brew upgrade opencrabs` (if you type `/evolve` on a Homebrew install, it delegates to Homebrew — the binary stays managed by brew, and the old tap is retired in favour of homebrew-core autobumps)
- **Binary users:** Type `/evolve` in the TUI to download the latest release
- **Source users:** `git pull && cargo build --release`, or type `/rebuild` in the TUI
- **Docker users:** `docker compose pull && docker compose up -d`
