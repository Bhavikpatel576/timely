use crate::error::Result;
use std::process::Command;

const AFK_THRESHOLD_NS: u64 = 180_000_000_000; // 3 minutes in nanoseconds

pub fn is_afk() -> Result<bool> {
    let output = Command::new("ioreg")
        .args(["-c", "IOHIDSystem", "-d", "4"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        if line.contains("HIDIdleTime") {
            // Line looks like: "HIDIdleTime" = 1234567890
            if let Some(val_str) = line.split('=').nth(1) {
                let val_str = val_str.trim();
                if let Ok(idle_ns) = val_str.parse::<u64>() {
                    return Ok(idle_ns > AFK_THRESHOLD_NS);
                }
            }
        }
    }

    Ok(false)
}
