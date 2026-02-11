# Multi-Device Sync for Timely

**Status**: Implemented

## Overview

Timely supports multi-device sync where one Mac acts as a hub and others push events to it. CLI queries with `--all-devices` hit the hub's API for an aggregated view.

## Architecture

- **Hub Mac**: Runs daemon (local events) + web server (receives remote events, serves queries)
- **Client Macs**: Run daemon (local events) + periodic sync push to hub
- **Auth**: Optional shared API key in `X-API-Key` header (open mode when no key is set)

## Setup

### Quick setup (no auth, trusted network)

```bash
# Hub Mac
timely daemon start
timely dashboard --port 8080

# Each client Mac
timely sync setup --hub http://192.168.1.10:8080
timely daemon start
```

### With authentication

```bash
# Hub Mac
timely config set sync.api_key "$(openssl rand -hex 16)"
timely daemon start
timely dashboard --port 8080

# Each client Mac
timely sync setup --hub http://192.168.1.10:8080 --key "<same-key>"
timely daemon start
```

## Querying

```bash
timely summary --from 7d --json --all-devices    # aggregated from hub
timely summary --from 7d --json                   # local only (unchanged)
timely sync status --json                          # check sync health
```

## Sync API Endpoints

All endpoints require `X-API-Key` header.

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/sync/push` | POST | Push batch of events from client to hub |
| `/api/sync/register` | POST | Register a device with the hub |
| `/api/sync/status` | GET | List all devices + event counts |

## Config Keys

| Key | Description | Default |
|-----|-------------|---------|
| `sync.hub_url` | Hub server URL | (none) |
| `sync.api_key` | Shared authentication key | (none) |
| `sync.enabled` | Enable auto-push in daemon | `false` |
| `sync.interval_secs` | Push interval in seconds | `300` |

## Files

### New files
- `src/sync/mod.rs` — Module exports
- `src/sync/server.rs` — Axum handlers for push/register/status
- `src/sync/client.rs` — HTTP client for pushing events + remote queries
- `src/sync/auth.rs` — API key validation middleware
- `src/cli/sync_cmd.rs` — CLI sync subcommands
- `tests/sync_test.rs` — Integration tests (10 tests)

### Modified files
- `Cargo.toml` — Added `reqwest` dependency
- `src/lib.rs` — Added `pub mod sync`
- `src/config.rs` — Added `SYNC_DEFAULT_INTERVAL_SECS`
- `src/error.rs` — Added `Sync` error variant
- `src/types.rs` — Added sync response types
- `src/db/schema.rs` — Migration v2: `sync_log` table
- `src/db/mod.rs` — Added `pub mod sync`
- `src/db/sync.rs` — Sync DB operations (upsert, dedup, sync log)
- `src/db/events.rs` — Added `query_events_after_id()`
- `src/web/router.rs` — Added sync API routes with auth middleware
- `src/web/handlers.rs` — Added `device` query param to summary/timeline
- `src/daemon/mod.rs` — Added sync tick to daemon loop
- `src/cli/mod.rs` — Added `Sync` command, `--all-devices`/`--device` flags
- `src/cli/sync_cmd.rs` — Sync subcommand implementations
- `src/cli/summary.rs` — Remote query mode
- `src/cli/timeline.rs` — Remote query mode
- `src/cli/now.rs` — Remote query mode
- `src/main.rs` — Wired up sync command dispatch
