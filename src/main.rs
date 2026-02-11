use clap::Parser;
use std::process;

use timely::cli::{self, Cli, Commands, DaemonAction, CategorizeAction, ConfigAction, DevicesAction, SyncAction};
use timely::output;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Daemon { action } => match action {
            DaemonAction::Start => cli::daemon::cmd_start(),
            DaemonAction::Stop => cli::daemon::cmd_stop(),
            DaemonAction::Status { json } => cli::daemon::cmd_status(json),
            DaemonAction::Run => cli::daemon::cmd_run(),
        },
        Commands::Now { json, all_devices, device } => {
            cli::now::cmd_now(json, all_devices, device.as_deref())
        }
        Commands::Summary { from, to, by, json, all_devices, device } => {
            cli::summary::cmd_summary(&from, &to, &by, json, all_devices, device.as_deref())
        }
        Commands::Timeline { from, to, limit, json, all_devices, device } => {
            cli::timeline::cmd_timeline(&from, &to, limit, json, all_devices, device.as_deref())
        }
        Commands::Categorize { action } => match action {
            CategorizeAction::Set { pattern, category, field, retroactive } => {
                cli::categorize::cmd_set(&pattern, &category, &field, retroactive)
            }
            CategorizeAction::List { json } => cli::categorize::cmd_list(json),
            CategorizeAction::Delete { id } => cli::categorize::cmd_delete(id),
        },
        Commands::Config { action } => match action {
            ConfigAction::Set { key, value } => cli::config_cmd::cmd_set(&key, &value),
            ConfigAction::Get { key } => cli::config_cmd::cmd_get(&key),
            ConfigAction::List { json } => cli::config_cmd::cmd_list(json),
        },
        Commands::Devices { action } => match action {
            DevicesAction::List { json } => cli::devices::cmd_list(json),
        },
        Commands::Export { format, from, to } => {
            cli::export::cmd_export(&format, &from, &to)
        }
        Commands::Import { file } => cli::import_cmd::cmd_import(&file),
        Commands::Dashboard { port } => cli::dashboard::cmd_dashboard(port),
        Commands::Sync { action } => match action {
            SyncAction::Setup { hub, key } => cli::sync_cmd::cmd_setup(&hub, key.as_deref()),
            SyncAction::Push => cli::sync_cmd::cmd_push(),
            SyncAction::Status { json } => cli::sync_cmd::cmd_status(json),
        },
    };

    if let Err(e) = result {
        output::print_error_json(&e);
        process::exit(1);
    }
}
