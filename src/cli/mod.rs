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

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "timely", about = "Agent-friendly activity tracker", version)]
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
        /// Output as JSON
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
        /// Output as JSON
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
        /// Output as JSON
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
}

#[derive(Subcommand)]
pub enum DaemonAction {
    /// Start the daemon (via launchd on macOS)
    Start,
    /// Stop the daemon
    Stop,
    /// Check daemon status
    Status {
        /// Output as JSON
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
    },
    /// List all category rules
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Delete a category rule by ID
    Delete {
        /// Rule ID to delete
        id: i64,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a config value
    Set {
        key: String,
        value: String,
    },
    /// Get a config value
    Get {
        key: String,
    },
    /// List all config values
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub enum DevicesAction {
    /// List registered devices
    List {
        /// Output as JSON
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
    },
    /// Push unsynced events to hub
    Push,
    /// Show sync status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}
