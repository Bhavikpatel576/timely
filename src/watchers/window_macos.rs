use crate::error::Result;
use std::process::Command;

pub struct WindowInfo {
    pub app: String,
    pub title: String,
}

pub fn get_active_window() -> Result<WindowInfo> {
    // Use System Events for both app name AND window title.
    // This only requires Accessibility permission — avoids per-app
    // Automation permission popups that `tell application X` triggers.
    let script = r#"
tell application "System Events"
    set frontProc to first application process whose frontmost is true
    set frontApp to name of frontProc
    try
        set windowTitle to name of front window of frontProc
    on error
        set windowTitle to ""
    end try
end tell

return frontApp & "|" & windowTitle
"#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if let Some((app, title)) = stdout.split_once('|') {
        Ok(WindowInfo {
            app: app.to_string(),
            title: title.to_string(),
        })
    } else {
        Ok(WindowInfo {
            app: stdout,
            title: String::new(),
        })
    }
}
