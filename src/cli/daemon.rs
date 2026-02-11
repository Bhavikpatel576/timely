use crate::config;
use crate::error::{Result, TimelyError};
use crate::output;
use crate::types::DaemonStatus;

pub fn cmd_start() -> Result<()> {
    // Check if already running
    if let Some(pid) = read_pid()? {
        if is_process_alive(pid) {
            return Err(TimelyError::DaemonAlreadyRunning(pid));
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Resolve symlinks to get the real binary path (e.g. inside .app bundle)
        let exe = std::env::current_exe()?.canonicalize()?;

        // Warn if binary is not inside a .app bundle
        let exe_path = exe.display().to_string();
        if !exe_path.contains(".app/Contents/MacOS/") {
            eprintln!("Warning: timely is not running from a .app bundle.");
            eprintln!("  macOS requires an app bundle to grant Accessibility permissions.");
            eprintln!("  Install with: make install (or see README for details)");
        }

        // Generate launchd plist
        let plist_path = config::launchd_plist_path()?;
        let plist_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>daemon</string>
        <string>run</string>
    </array>
    <key>KeepAlive</key>
    <true/>
    <key>RunAtLoad</key>
    <true/>
    <key>AssociatedBundleIdentifiers</key>
    <string>{bundle_id}</string>
    <key>StandardErrorPath</key>
    <string>{log_dir}/timely.log</string>
    <key>StandardOutPath</key>
    <string>{log_dir}/timely.log</string>
</dict>
</plist>"#,
            label = config::LAUNCHD_LABEL,
            exe = exe.display(),
            bundle_id = config::BUNDLE_IDENTIFIER,
            log_dir = config::data_dir()?.display(),
        );

        std::fs::write(&plist_path, plist_content)?;

        let status = std::process::Command::new("launchctl")
            .args(["load", "-w"])
            .arg(&plist_path)
            .status()?;

        if status.success() {
            println!("Daemon started via launchd");
        } else {
            return Err(TimelyError::Generic("Failed to load launchd plist".into()));
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        return Err(TimelyError::PlatformNotSupported(
            "daemon start only supported on macOS (launchd)".into(),
        ));
    }

    Ok(())
}

pub fn cmd_stop() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // Try launchctl remove first
        let status = std::process::Command::new("launchctl")
            .args(["remove", config::LAUNCHD_LABEL])
            .status();

        if let Ok(s) = status {
            if s.success() {
                // Clean up PID file
                let _ = std::fs::remove_file(config::pid_path()?);
                println!("Daemon stopped");
                return Ok(());
            }
        }

        // Fallback: kill via PID file
        if let Some(pid) = read_pid()? {
            if is_process_alive(pid) {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
                let _ = std::fs::remove_file(config::pid_path()?);
                println!("Daemon stopped (pid {})", pid);
                return Ok(());
            }
        }

        Err(TimelyError::DaemonNotRunning)
    }

    #[cfg(not(target_os = "macos"))]
    {
        if let Some(pid) = read_pid()? {
            if is_process_alive(pid) {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
                let _ = std::fs::remove_file(config::pid_path()?);
                println!("Daemon stopped (pid {})", pid);
                return Ok(());
            }
        }
        Err(TimelyError::DaemonNotRunning)
    }
}

pub fn cmd_status(json: bool) -> Result<()> {
    let (running, pid) = if let Some(pid) = read_pid()? {
        (is_process_alive(pid), Some(pid))
    } else {
        (false, None)
    };

    let status = DaemonStatus {
        running,
        pid: if running { pid } else { None },
        uptime_seconds: None, // Would need start time tracking
        uptime_time: None,
    };

    if json {
        output::print_json(&status);
    } else if running {
        println!("Daemon is running (pid {})", pid.unwrap());
    } else {
        println!("Daemon is not running");
    }

    Ok(())
}

pub fn cmd_run() -> Result<()> {
    crate::daemon::run_daemon()
}

fn read_pid() -> Result<Option<u32>> {
    let path = config::pid_path()?;
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;
        if let Ok(pid) = content.trim().parse::<u32>() {
            return Ok(Some(pid));
        }
    }
    Ok(None)
}

fn is_process_alive(pid: u32) -> bool {
    unsafe { libc::kill(pid as i32, 0) == 0 }
}
