# Companion Tools

OpenCrabs works with companion tools that extend its capabilities.

## WhisperCrabs: Voice-to-Text

[WhisperCrabs](https://github.com/adolfousier/whispercrabs) is a floating voice-to-text tool. Click to record, click to stop, transcribes, copies to clipboard.

- **Local** (whisper.cpp, on-device) or **API** transcription
- Fully controllable via D-Bus: start/stop recording, switch providers, view history
- Works as an OpenCrabs tool: use D-Bus to control WhisperCrabs from the agent

### Installation

```bash
git clone https://github.com/adolfousier/whispercrabs.git
cd whispercrabs
cargo build --release
```

### Usage with OpenCrabs

Just ask naturally. OpenCrabs controls WhisperCrabs via D-Bus:

> "Start recording" / "Stop and transcribe" / "Switch to Groq Whisper"

---

## SocialCrabs: Social Media Automation

[SocialCrabs](https://github.com/adolfousier/socialcrabs) automates social media via **CLI + GraphQL** with human-like behavior simulation. Twitter/X, Instagram, LinkedIn. No browser needed for read operations.

### Setup

```bash
git clone https://github.com/adolfousier/socialcrabs.git
cd socialcrabs && npm install && npm run build

# Add cookies from browser DevTools to .env (auth_token + ct0 for Twitter)
# See SocialCrabs README for per-platform credential setup

node dist/cli.js session login x          # Authenticate Twitter/X
node dist/cli.js session login ig         # Authenticate Instagram
node dist/cli.js session login linkedin   # Authenticate LinkedIn
node dist/cli.js session status           # Check all sessions
```

### Usage with OpenCrabs

Just ask naturally. OpenCrabs calls SocialCrabs CLI commands via `bash` automatically:

> "Check my Twitter mentions" / "Search LinkedIn for AI founders" / "Post this to X"

Read operations run automatically. Write operations (tweet, like, follow, comment, DM) always ask for your approval first.

### Twitter/X commands

```bash
node dist/cli.js x whoami                     # Check logged-in account
node dist/cli.js x mentions -n 5              # Your mentions
node dist/cli.js x home -n 5                  # Your timeline
node dist/cli.js x search "query" -n 10       # Search tweets
node dist/cli.js x read <tweet-url>           # Read a specific tweet
node dist/cli.js x tweet "Hello world"        # Post a tweet
node dist/cli.js x reply <tweet-url> "text"   # Reply to tweet
node dist/cli.js x like <tweet-url>           # Like a tweet
node dist/cli.js x follow <username>          # Follow a user
```

### Instagram commands

```bash
node dist/cli.js ig feed -n 5                 # Your feed
node dist/cli.js ig search "query" -n 10      # Search posts
node dist/cli.js ig read <post-url>           # Read a specific post
node dist/cli.js ig like <post-url>           # Like a post
node dist/cli.js ig comment <post-url> "text" # Comment on post
node dist/cli.js ig follow <username>         # Follow a user
```

### LinkedIn commands

```bash
node dist/cli.js linkedin feed -n 5           # Your feed
node dist/cli.js linkedin search "query" -n 10 # Search posts
node dist/cli.js linkedin read <post-url>     # Read a specific post
node dist/cli.js linkedin like <post-url>     # Like a post
node dist/cli.js linkedin comment <post-url> "text" # Comment on post
```
