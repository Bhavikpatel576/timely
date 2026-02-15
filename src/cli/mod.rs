pub mod daemon;
pub mod now;
pub mod summary;
pub mod timeline;
pub mod categorize;
pub mod config_cmd;
pub mod devices;
pub mod export;
pub mod import_cmd;
pub mod dashboard;
pub mod sync_cmd;
pub mod focus;
pub mod trends;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "timely",
    about = "Agent-friendly activity tracker",
    long_about = "Agent-friendly activity tracker for macOS.\n\n\
        All commands support --json for structured output using a standard envelope:\n  \
        Success: {\"ok\": true, \"data\": ...}\n  \
        Error:   {\"ok\": false, \"error\": \"...\", \"error_code\": \"...\"}\n\n\
        Exit codes: 0 = success, 1 = error (with JSON on stderr).",
    version,
    after_help = "EXAMPLES:\n  \
        timely daemon start            Start the background daemon\n  \
        timely now --json              Current activity as JSON\n  \
        timely summary --json          Today's summary as JSON\n  \
        timely summary --from 2d --json  Last 2 days summary\n  \
        timely timeline --from 1h --json  Last hour timeline\n  \
        timely categorize set Code work/coding --field app\n  \
        timely config set sync.enabled true\n\n\
        TIME RANGES:\n  \
        now, today, yesterday, Nd (days), Nh (hours), Nm (minutes), YYYY-MM-DD"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage the background daemon
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Show current activity
    Now {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
        /// Query all devices via hub
        #[arg(long)]
        all_devices: bool,
        /// Query a specific device by name
        #[arg(long)]
        device: Option<String>,
    },
    /// Show activity summary
    Summary {
        /// Start time (default: today)
        #[arg(long, default_value = "today")]
        from: String,
        /// End time (default: now)
        #[arg(long, default_value = "now")]
        to: String,
        /// Group by: category, app, or url
        #[arg(long, default_value = "category")]
        by: String,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
        /// Query all devices via hub
        #[arg(long)]
        all_devices: bool,
        /// Query a specific device by name
        #[arg(long)]
        device: Option<String>,
    },
    /// Show activity timeline
    Timeline {
        /// Start time (default: today)
        #[arg(long, default_value = "today")]
        from: String,
        /// End time (default: now)
        #[arg(long, default_value = "now")]
        to: String,
        /// Limit number of entries
        #[arg(long)]
        limit: Option<i64>,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
        /// Query all devices via hub
        #[arg(long)]
        all_devices: bool,
        /// Query a specific device by name
        #[arg(long)]
        device: Option<String>,
    },
    /// Manage category rules
    Categorize {
        #[command(subcommand)]
        action: CategorizeAction,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// List tracked devices
    Devices {
        #[command(subcommand)]
        action: DevicesAction,
    },
    /// Export activity data
    Export {
        /// Output format: json or csv
        #[arg(long, default_value = "json")]
        format: String,
        /// Start time
        #[arg(long, default_value = "today")]
        from: String,
        /// End time
        #[arg(long, default_value = "now")]
        to: String,
    },
    /// Import activity data from file
    Import {
        /// Path to import file (JSON)
        file: String,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Launch the web dashboard
    Dashboard {
        /// Port to serve on
        #[arg(long, default_value = "8080")]
        port: u16,
    },
    /// Multi-device sync management
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
    /// Analyze focus and context switching
    Focus {
        /// Start time (default: today)
        #[arg(long, default_value = "today")]
        from: String,
        /// End time (default: now)
        #[arg(long, default_value = "now")]
        to: String,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Show activity trends over time
    Trends {
        /// Start time (default: 7d)
        #[arg(long, default_value = "7d")]
        from: String,
        /// End time (default: now)
        #[arg(long, default_value = "now")]
        to: String,
        /// Bucket interval: hour, day, week, or month
        #[arg(long, default_value = "day")]
        interval: String,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the daemon (via launchd on macOS)
    Start {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Stop the daemon
    Stop {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Check daemon status
    Status {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Run daemon in foreground
    Run,
}

#[derive(Subcommand)]
pub enum CategorizeAction {
    /// Set a category rule
    Set {
        /// Pattern to match (e.g. "Code", "*.rs", "github.com")
        pattern: String,
        /// Category name (e.g. "work/coding")
        category: String,
        /// Field to match: app, title, or url_domain
        #[arg(long, default_value = "app")]
        field: String,
        /// Apply retroactively to existing events
        #[arg(long)]
        retroactive: bool,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// List all category rules
    List {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Delete a category rule by ID
    Delete {
        /// Rule ID to delete
        id: i64,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a config value
    Set {
        key: String,
        value: String,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Get a config value
    Get {
        key: String,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// List all config values
    List {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum DevicesAction {
    /// List registered devices
    List {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum SyncAction {
    /// Configure sync with a hub
    Setup {
        /// Hub URL (e.g. http://192.168.1.10:8080)
        #[arg(long)]
        hub: String,
        /// Shared API key for authentication (optional — omit for open-mode hubs)
        #[arg(long)]
        key: Option<String>,
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Push unsynced events to hub
    Push {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
    /// Show sync status
    Status {
        /// Output as JSON envelope: {"ok": true, "data": ...}
        #[arg(long)]
        json: bool,
    },
}
