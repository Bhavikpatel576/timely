use crate::config;
use crate::error::{Result, TimelyError};
use crate::output;
use serde::Serialize;
use std::path::PathBuf;

const GITHUB_API_LATEST: &str =
    "https://api.github.com/repos/Bhavikpatel576/timely/releases/latest";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const SYMLINK_PATH: &str = "/usr/local/bin/timely";

#[derive(Debug, Clone, Serialize)]
pub struct VersionCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_notes: Option<String>,
}

/// Check GitHub releases for the latest version.
pub fn check_for_update() -> Result<VersionCheckResult> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("timely/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| TimelyError::Generic(format!("HTTP client error: {}", e)))?;

    let resp = client
        .get(GITHUB_API_LATEST)
        .send()
        .map_err(|e| TimelyError::Generic(format!("Failed to check for updates: {}", e)))?;

    if resp.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(TimelyError::Generic(
            "GitHub API rate limit exceeded. Try again later.".into(),
        ));
    }

    if !resp.status().is_success() {
        return Err(TimelyError::Generic(format!(
            "GitHub API returned {}",
            resp.status()
        )));
    }

    let release: serde_json::Value = resp
        .json()
        .map_err(|e| TimelyError::Generic(format!("Failed to parse release info: {}", e)))?;

    let tag = release["tag_name"]
        .as_str()
        .ok_or_else(|| TimelyError::Generic("No tag_name in release".into()))?;
    let latest_version = tag.strip_prefix('v').unwrap_or(tag).to_string();

    let release_notes = release["body"].as_str().map(|s| s.to_string());

    let download_url = find_asset_url(&release, &latest_version);

    let update_available = is_newer(&latest_version, CURRENT_VERSION);

    Ok(VersionCheckResult {
        current_version: CURRENT_VERSION.to_string(),
        latest_version,
        update_available,
        download_url,
        release_notes,
    })
}

pub fn cmd_update(check_only: bool, no_restart: bool, json: bool) -> Result<()> {
    if !json {
        eprintln!("Checking for updates...");
    }

    let check = check_for_update()?;

    if check_only || !check.update_available {
        if json {
            output::print_json(&check);
        } else if check.update_available {
            println!(
                "Update available: {} -> {}",
                check.current_version, check.latest_version
            );
            println!("Run `timely update` to install it.");
        } else {
            println!(
                "You are on the latest version ({})",
                check.current_version
            );
        }
        return Ok(());
    }

    let download_url = check.download_url.as_deref().ok_or_else(|| {
        TimelyError::Generic(format!(
            "No release asset found for architecture {}",
            std::env::consts::ARCH
        ))
    })?;

    if !json {
        eprintln!(
            "Downloading timely v{}...",
            check.latest_version
        );
    }

    let result = download_and_apply(download_url, &check.latest_version, no_restart)?;

    if json {
        output::print_json(&result);
    } else {
        println!(
            "Updated timely {} -> {}",
            result.previous_version, result.new_version
        );
        if result.daemon_restarted {
            println!("Daemon restarted with new version.");
        }
    }

    Ok(())
}

#[derive(Debug, Serialize)]
struct UpdateApplyResult {
    previous_version: String,
    new_version: String,
    daemon_restarted: bool,
}

fn download_and_apply(
    download_url: &str,
    new_version: &str,
    no_restart: bool,
) -> Result<UpdateApplyResult> {
    // Detect install type
    let exe = std::env::current_exe()?.canonicalize()?;
    let exe_str = exe.display().to_string();

    if exe_str.contains("/Cellar/") || exe_str.contains("/homebrew/") {
        return Err(TimelyError::Generic(
            "Timely appears to be installed via Homebrew. Use `brew upgrade timely` instead."
                .into(),
        ));
    }

    let is_app_bundle = exe_str.contains(".app/Contents/MacOS/");
    let app_dest = PathBuf::from("/Applications/Timely.app");

    // Create temp directory
    let tmp_dir = std::env::temp_dir().join(format!("timely-update-{}", new_version));
    if tmp_dir.exists() {
        std::fs::remove_dir_all(&tmp_dir)?;
    }
    std::fs::create_dir_all(&tmp_dir)?;

    // Download tarball
    let tarball_path = tmp_dir.join("timely.tar.gz");
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("timely/{}", CURRENT_VERSION))
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| TimelyError::Generic(format!("HTTP client error: {}", e)))?;

    let resp = client
        .get(download_url)
        .send()
        .map_err(|e| TimelyError::Generic(format!("Download failed: {}", e)))?;

    if !resp.status().is_success() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(TimelyError::Generic(format!(
            "Download failed with status {}",
            resp.status()
        )));
    }

    let bytes = resp
        .bytes()
        .map_err(|e| TimelyError::Generic(format!("Failed to read download: {}", e)))?;
    std::fs::write(&tarball_path, &bytes)?;

    // Extract
    let tar_status = std::process::Command::new("tar")
        .args(["-xzf", &tarball_path.display().to_string(), "-C", &tmp_dir.display().to_string()])
        .status()?;

    if !tar_status.success() {
        let _ = std::fs::remove_dir_all(&tmp_dir);
        return Err(TimelyError::Generic("Failed to extract update archive".into()));
    }

    // Check if daemon is running
    let daemon_was_running = is_daemon_running()?;

    // Stop daemon if running
    if daemon_was_running && !no_restart {
        eprintln!("Stopping daemon for update...");
        let _ = std::process::Command::new("launchctl")
            .args(["remove", config::LAUNCHD_LABEL])
            .status();
    }

    if is_app_bundle {
        // Replace app bundle
        let new_app = tmp_dir.join("Timely.app");
        if !new_app.exists() {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err(TimelyError::Generic(
                "Extracted archive does not contain Timely.app".into(),
            ));
        }

        // Remove old bundle
        if app_dest.exists() {
            if let Err(e) = std::fs::remove_dir_all(&app_dest) {
                let _ = std::fs::remove_dir_all(&tmp_dir);
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    return Err(TimelyError::Generic(format!(
                        "Permission denied writing to {}. Try: sudo timely update",
                        app_dest.display()
                    )));
                }
                return Err(e.into());
            }
        }

        // Move new bundle into place
        let mv_status = std::process::Command::new("mv")
            .args([
                &new_app.display().to_string(),
                &app_dest.display().to_string(),
            ])
            .status()?;

        if !mv_status.success() {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err(TimelyError::Generic(
                "Failed to install new version. You may need to re-run install.sh".into(),
            ));
        }

        // Re-create symlink
        let binary_path = app_dest.join("Contents/MacOS/timely");
        let symlink = PathBuf::from(SYMLINK_PATH);
        let _ = std::fs::remove_file(&symlink);
        if let Err(e) = std::os::unix::fs::symlink(&binary_path, &symlink) {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                eprintln!(
                    "Warning: Could not update symlink at {}. You may need: sudo ln -sf {} {}",
                    SYMLINK_PATH,
                    binary_path.display(),
                    SYMLINK_PATH
                );
            }
        }
    } else {
        // Non-app-bundle: replace binary in place
        let new_binary = tmp_dir.join("Timely.app/Contents/MacOS/timely");
        let fallback_binary = tmp_dir.join("timely");
        let source = if new_binary.exists() {
            new_binary
        } else if fallback_binary.exists() {
            fallback_binary
        } else {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            return Err(TimelyError::Generic(
                "Could not find timely binary in extracted archive".into(),
            ));
        };

        let target = &exe;
        if let Err(e) = std::fs::copy(&source, target) {
            let _ = std::fs::remove_dir_all(&tmp_dir);
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(TimelyError::Generic(format!(
                    "Permission denied writing to {}. Try: sudo timely update",
                    target.display()
                )));
            }
            return Err(e.into());
        }
    }

    // Restart daemon if it was running
    let daemon_restarted = if daemon_was_running && !no_restart {
        eprintln!("Restarting daemon...");
        let plist_path = config::launchd_plist_path()?;
        let status = std::process::Command::new("launchctl")
            .args(["load", "-w"])
            .arg(&plist_path)
            .status()?;
        status.success()
    } else {
        false
    };

    // Cleanup
    let _ = std::fs::remove_dir_all(&tmp_dir);

    Ok(UpdateApplyResult {
        previous_version: CURRENT_VERSION.to_string(),
        new_version: new_version.to_string(),
        daemon_restarted,
    })
}

fn find_asset_url(release: &serde_json::Value, version: &str) -> Option<String> {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "arm64",
        other => other,
    };

    let expected_name = format!("timely-v{}-{}-apple-darwin.tar.gz", version, arch);

    let assets = release["assets"].as_array()?;
    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            if name == expected_name {
                return asset["browser_download_url"].as_str().map(|s| s.to_string());
            }
        }
    }
    None
}

fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = v.split('.').filter_map(|s| s.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(latest) > parse(current)
}

fn is_daemon_running() -> Result<bool> {
    let pid_path = config::pid_path()?;
    if pid_path.exists() {
        let content = std::fs::read_to_string(&pid_path)?;
        if let Ok(pid) = content.trim().parse::<u32>() {
            return Ok(unsafe { libc::kill(pid as i32, 0) == 0 });
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer() {
        assert!(!is_newer("0.3.0", "0.3.0"));
        assert!(is_newer("0.4.0", "0.3.0"));
        assert!(is_newer("1.0.0", "0.9.9"));
        assert!(is_newer("0.3.1", "0.3.0"));
        assert!(!is_newer("0.2.0", "0.3.0"));
        assert!(is_newer("0.10.0", "0.9.0"));
    }

    #[test]
    fn test_arch_mapping() {
        // Just verify the mapping logic works for known architectures
        let arch = match "aarch64" {
            "aarch64" => "arm64",
            other => other,
        };
        assert_eq!(arch, "arm64");

        let arch = match "x86_64" {
            "aarch64" => "arm64",
            other => other,
        };
        assert_eq!(arch, "x86_64");
    }
}
