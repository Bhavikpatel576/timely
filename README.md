# Timely

A lightweight, agent-friendly activity tracker for macOS. Timely runs as a background daemon, watches which apps and websites you use, categorizes your time, and serves it all through a CLI and web dashboard — from a single binary.

## Features

- **Background daemon** — polls every 5 seconds, merges heartbeats into continuous activity sessions
- **Automatic categorization** — 80+ builtin rules for common apps (IDEs, browsers, chat, etc.)
- **Productivity scoring** — weighted scores from -2 (distracting) to +2 (productive), mapped to 0-100
- **Web dashboard** — React frontend embedded in the binary, served via `timely dashboard`
- **JSON output** — every CLI command supports `--json` for agent/script consumption
- **AFK detection** — uses macOS idle time (via `ioreg`) to track away-from-keyboard periods
- **Browser tracking** — captures active URL and domain from Chrome, Safari, Arc, Firefox via AppleScript
- **TUI app detection** — detects Claude Code, Codex, Vim, Neovim, and 20+ other tools running inside terminals

## Installation

### Homebrew (recommended)

```sh
brew tap Bhavikpatel576/tap
brew install timely
```

After installing, grant Accessibility permissions and start the daemon:

```sh
# Open System Settings > Privacy & Security > Accessibility
# Click '+' and add the terminal you run timely from
timely daemon start
```

### From source

```sh
# Clone the repo
git clone https://github.com/Bhavikpatel576/timely.git
cd timely

# Build everything (dashboard + Rust binary)
make build

# The binary is at target/release/timely
cp target/release/timely /usr/local/bin/
```

### Install script

```sh
curl -fsSL https://raw.githubusercontent.com/Bhavikpatel576/timely/main/install.sh | sh
```

### Rust binary only (no dashboard)

```sh
cargo build --release
# Dashboard will show a placeholder page, but CLI works fully
```

## Quick Start

```sh
# Start the background daemon
timely daemon start

# Check what you're doing right now
timely now

# See today's activity summary
timely summary

# Launch the web dashboard
timely dashboard
```

## CLI Reference

### `timely daemon`

Manage the background activity tracker.

```sh
timely daemon start              # Start via launchd (persists across reboots)
timely daemon start --json       # JSON confirmation
timely daemon stop               # Stop the daemon
timely daemon stop --json        # JSON confirmation
timely daemon status             # Check if running
timely daemon status --json      # Structured status output
timely daemon run                # Run in foreground (useful for debugging)
```

### `timely now`

Show current activity.

```sh
timely now                    # Human-readable
timely now --json             # JSON envelope: { "ok": true, "data": { ... } }
timely now --all-devices      # Current activity from hub (all devices)
timely now --device laptop    # Current activity for a specific device
```

### `timely summary`

Show activity summary grouped by category, app, or URL domain.

```sh
timely summary                          # Today, grouped by category
timely summary --from yesterday --to now
timely summary --from 7d --by app       # Last 7 days, grouped by app
timely summary --from 2026-01-01 --to 2026-01-31 --by url --json
```

| Flag | Default | Description |
|------|---------|-------------|
| `--from` | `today` | Start time: `now`, `today`, `yesterday`, `Nd`, `Nh`, `Nm`, or `YYYY-MM-DD` |
| `--to` | `now` | End time (same format) |
| `--by` | `category` | Group by: `category`, `app`, or `url` |
| `--json` | false | Output as JSON |
| `--all-devices` | false | Query all devices via hub |
| `--device` | — | Query a specific device by name |

### `timely timeline`

Show individual activity events in chronological order.

```sh
timely timeline                         # Today's events
timely timeline --from 2d --limit 50           # Last 2 days, max 50 entries
timely timeline --json
timely timeline --from 7d --all-devices --json # All devices via hub
```

### `timely categorize`

Manage category rules. Rules map app names, window titles, or URL domains to categories.

```sh
# Assign Figma to work/design
timely categorize set Figma work/design --field app

# Assign github.com to work/coding, and apply to existing events
timely categorize set github.com work/coding --field url_domain --retroactive --json

# List all rules
timely categorize list --json

# Delete a user rule by ID
timely categorize delete 42 --json
```

| Flag | Default | Description |
|------|---------|-------------|
| `--field` | `app` | Field to match: `app`, `title`, or `url_domain` |
| `--retroactive` | false | Recategorize existing events matching this rule |
| `--json` | false | Output as JSON envelope |

### `timely dashboard`

Launch the web dashboard. Opens your browser to a React-based activity viewer.

```sh
timely dashboard                # Serves on http://localhost:8080
timely dashboard --port 9090    # Custom port
```

### `timely config`

Manage configuration key-value pairs.

```sh
timely config set key value --json   # Returns {"ok": true, "data": {"key": ..., "value": ...}}
timely config get key --json         # Returns {"ok": true, "data": {"key": ..., "value": ...}}
timely config list --json
```

### `timely devices`

List tracked devices.

```sh
timely devices list --json
```

### `timely export` / `timely import`

Export and import activity data.

```sh
timely export --format json --from 7d --to now > backup.json
timely import backup.json --json  # Returns {"ok": true, "data": {"imported": N, "file": ...}}
```

## Web Dashboard API

When running `timely dashboard`, the following REST API is available:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/current` | Current activity (last event) |
| GET | `/api/summary?from=&to=&groupBy=` | Activity summary |
| GET | `/api/apps?from=&to=&limit=` | App breakdown |
| GET | `/api/timeline?from=&to=&limit=` | Event timeline |
| GET | `/api/productivity?from=&to=` | Productivity score + breakdown |
| GET | `/api/trends?from=&to=&interval=` | Trends by day/week/month |
| GET | `/api/categories` | All categories |
| GET | `/api/rules` | All category rules |
| POST | `/api/rules` | Create rule `{ app, category_id, field }` |
| PUT | `/api/rules/:id` | Update rule `{ category_id }` |
| DELETE | `/api/rules/:id` | Delete rule |

Date parameters use `YYYY-MM-DD` format. The `interval` parameter accepts `day`, `week`, or `month`.

## Builtin Categories

Timely ships with categories and productivity scores out of the box:

| Category | Score | Examples |
|----------|-------|---------|
| work/coding | +2.0 | VS Code, Cursor, Xcode, IntelliJ, Vim, Zed |
| work/ai-tools | +2.0 | Claude Code, Codex CLI, Aider, Cursor CLI |
| work/terminal | +1.5 | Terminal, iTerm2, Warp, Ghostty, kitty |
| work/writing | +1.5 | Notion, Obsidian, Pages, Word |
| work/design | +1.0 | Figma, Sketch, Photoshop |
| reference/docs | +1.0 | docs.rs, StackOverflow, MDN, GitHub |
| communication/email | +0.5 | Mail, Outlook, Gmail |
| communication/chat | 0.0 | Slack, Discord, Teams, Messages |
| utilities | 0.0 | Finder, System Settings, 1Password |
| entertainment/music | -0.5 | Spotify, Apple Music |
| entertainment | -1.0 | — |
| social-media | -1.5 | Twitter/X, Reddit, Facebook, Instagram |
| entertainment/video | -2.0 | YouTube, Netflix, Twitch, VLC |
| entertainment/gaming | -2.0 | Steam |

Scores range from -2 (most distracting) to +2 (most productive) and are mapped to a 0-100 scale in the productivity endpoint.

## Multi-Device Sync

Track activity across multiple Macs and query it from one place. One Mac acts as the **hub** (receives events, serves queries), the others are **clients** (push events to hub).

### Quick setup (no auth)

If all your Macs are on a trusted network:

```sh
# === Hub Mac (e.g. your desktop that's always on) ===

timely daemon start                       # track local activity
timely dashboard --port 8080              # start the server

# === Each client Mac (e.g. your laptop) ===

timely sync setup --hub http://192.168.1.10:8080
timely daemon start                       # tracks local + auto-pushes every 5 min
```

That's it. The client registers itself with the hub and starts syncing automatically.

### Setup with authentication

For untrusted networks, add a shared API key:

```sh
# === Hub Mac ===

timely config set sync.api_key "$(openssl rand -hex 16)"
timely daemon start
timely dashboard --port 8080

# === Each client Mac (use the same key) ===

timely sync setup --hub http://192.168.1.10:8080 --key "your-key-here"
timely daemon start
```

When a key is set on the hub, all sync requests must include a matching `X-API-Key` header. Without a key, the hub runs in open mode.

### Querying across devices

```sh
# Aggregated summary from all devices (fetched from hub)
timely summary --from 7d --json --all-devices

# Filter to a specific device by name
timely timeline --from today --device macbook-air --json

# Local-only query (unchanged, no flags needed)
timely summary --json
```

The `--all-devices` and `--device` flags work on `summary`, `timeline`, and `now` commands. They make an HTTP request to the hub instead of reading the local DB.

### Manual sync

```sh
# One-shot push (useful for initial sync or manual trigger)
timely sync push

# Check sync health
timely sync status
timely sync status --json
```

### `timely sync`

| Command | Description |
|---------|-------------|
| `timely sync setup --hub URL [--key KEY] [--json]` | Configure sync with a hub. Registers device, enables auto-push. |
| `timely sync push [--json]` | Push all unsynced events to hub now. |
| `timely sync status [--json]` | Show hub URL, reachability, pending events, registered devices. |

### How it works

1. The daemon polls activity every 5 seconds (unchanged)
2. When sync is enabled, the daemon pushes new events to the hub every 5 minutes (configurable via `sync.interval_secs`)
3. Events are pushed in batches of 1000 with deduplication — same `(device_id, timestamp, app, title)` won't create duplicates
4. If the hub already has an event with shorter duration, it takes the longer one (`MAX(duration)`)
5. The hub stores all events in its local SQLite DB. Queries with `--all-devices` hit the hub's API

### Sync config keys

| Key | Default | Description |
|-----|---------|-------------|
| `sync.hub_url` | — | Hub server URL |
| `sync.api_key` | — | Shared API key (optional) |
| `sync.enabled` | `false` | Auto-push in daemon loop |
| `sync.interval_secs` | `300` | Seconds between auto-pushes |

### Sync API endpoints

Available when running `timely dashboard`. Protected by API key middleware when `sync.api_key` is set.

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/sync/push` | Push batch of events from client |
| POST | `/api/sync/register` | Register a device with hub |
| GET | `/api/sync/status` | List all devices + event counts |

Existing query endpoints (`/api/summary`, `/api/timeline`) also accept a `device` query parameter to filter by device name.

## Data Storage

- Database: `~/.timely/timely.db` (SQLite, WAL mode)
- PID file: `~/.timely/timely.pid`
- Launchd plist: `~/Library/LaunchAgents/com.timely.daemon.plist`

## Development

### Project Structure

```
src/
├── cli/           # CLI commands (clap)
├── db/            # SQLite layer (rusqlite)
├── query/         # Summary, timeline, apps, productivity, trends, current
├── web/           # Axum server, handlers, rust-embed assets
├── watchers/      # macOS-specific (AppleScript, ioreg)
├── categories/    # Classification engine + builtin rules
├── daemon/        # Heartbeat merge logic
├── config.rs      # Paths and constants
├── types.rs       # Shared data types
├── error.rs       # Error types
├── output.rs      # JSON envelope helpers
└── lib.rs / main.rs

dashboard/         # React + Vite + Tailwind frontend
├── src/           # React components
├── server/        # Express dev server (for hot-reload)
└── vite.config.ts
```

### Build Targets

```sh
make build           # Build dashboard + release binary
make build-dashboard # Build React frontend only
make build-rust      # Build Rust binary only (release)
make build-debug     # Dashboard + debug binary (faster)
make dev             # Run Vite dev server with hot-reload
make clean           # Remove all build artifacts
make run             # Build everything and launch dashboard
```

### Dashboard Development

For frontend development with hot-reload:

```sh
# Terminal 1: Express API server
cd dashboard && npm run dev

# Terminal 2: Vite dev server (proxies /api to Express on port 3123)
cd dashboard && npx vite

# Or point Vite at the Rust dashboard server instead:
VITE_API_PORT=8080 npx vite
```

### Running Tests

```sh
cargo test           # 43 tests: heartbeat, categories, queries, sync, unit tests
```

## Releasing

To publish a new version:

```sh
# Tag and push — CI handles everything else
git tag v0.2.0
git push origin v0.2.0
```

The release workflow (`.github/workflows/release.yml`) automatically:

1. Builds the binary for both **arm64** and **x86_64** on macOS
2. Bundles each into a signed `Timely.app`
3. Uploads `.tar.gz` tarballs + SHA256 checksums to the GitHub release
4. Downloads both tarballs, computes checksums, and updates the Homebrew formula in [`Bhavikpatel576/homebrew-tap`](https://github.com/Bhavikpatel576/homebrew-tap)

After a release, users get the new version via:

```sh
brew upgrade timely
# or
curl -fsSL https://raw.githubusercontent.com/Bhavikpatel576/timely/main/install.sh | sh
```

### How the Homebrew tap works

Homebrew taps are GitHub repos named `homebrew-<name>`. Ours is [`Bhavikpatel576/homebrew-tap`](https://github.com/Bhavikpatel576/homebrew-tap), which contains a single formula at `Formula/timely.rb`.

When a user runs:

```sh
brew tap Bhavikpatel576/tap        # clones homebrew-tap repo locally
brew install --no-quarantine timely # reads Formula/timely.rb, downloads the tarball
```

The formula detects the user's architecture (arm64 vs x86_64), downloads the correct tarball from GitHub Releases, extracts `Timely.app`, and symlinks the CLI binary to `bin/timely`.

`--no-quarantine` is needed because the binary isn't notarized with Apple — without it, macOS Gatekeeper blocks the app.

### CI secrets required

| Secret | Purpose |
|--------|---------|
| `HOMEBREW_TAP_TOKEN` | GitHub PAT with repo scope — used to push formula updates to `homebrew-tap` |

### One-time setup

The tap repo was bootstrapped via the `setup-tap.yml` workflow. You only need to run it once (Actions → "Setup Homebrew Tap" → Run workflow).

## JSON Output Format

All CLI commands with `--json` wrap responses in a standard envelope:

```json
// Success (exit code 0, stdout)
{
  "ok": true,
  "data": { ... }
}

// Error (exit code 1, stderr)
{
  "ok": false,
  "error": "No data for the requested time range",
  "error_code": "no_data"
}
```

### Error Codes

| Code | Meaning |
|------|---------|
| `db_error` | SQLite database error |
| `io_error` | File system or I/O error |
| `json_error` | JSON parse/serialize error |
| `config_error` | Missing or invalid configuration |
| `daemon_not_running` | Daemon is not running (for `daemon stop`) |
| `daemon_already_running` | Daemon is already running (for `daemon start`) |
| `no_data` | No events found for the requested time range |
| `invalid_time_range` | Unparseable `--from` or `--to` value |
| `category_not_found` | Category name not found |
| `rule_not_found` | Category rule ID not found |
| `platform_not_supported` | Feature not available on this OS |
| `sync_error` | Sync push/register failure |
| `error` | Generic error |

### Staleness Detection

The `now --json` response includes a `"stale"` field:

```json
{
  "ok": true,
  "data": {
    "app": "Code",
    "title": "main.rs",
    "stale": true,
    ...
  }
}
```

When `"stale": true`, the daemon has stopped and the data shows the **last recorded** activity, not the current one. Agents should check this field and prompt the user to start the daemon.

### Time Range Format

Used by `--from` and `--to` flags:

| Format | Example | Meaning |
|--------|---------|---------|
| `now` | `--from now` | Current time |
| `today` | `--from today` | Start of today (00:00 local) |
| `yesterday` | `--from yesterday` | Start of yesterday |
| `Nd` | `--from 7d` | N days ago |
| `Nh` | `--from 2h` | N hours ago |
| `Nm` | `--from 30m` | N minutes ago |
| `YYYY-MM-DD` | `--from 2026-01-15` | Specific date |

## License

MIT
