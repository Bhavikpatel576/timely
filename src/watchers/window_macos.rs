use crate::error::Result;
use std::process::Command;

pub struct WindowInfo {
    pub app: String,
    pub title: String,
}

pub fn get_active_window() -> Result<WindowInfo> {
    let script = r#"
tell application "System Events"
    set frontApp to name of first application process whose frontmost is true
end tell

try
    tell application frontApp
        set windowTitle to name of front window
    end tell
on error
    set windowTitle to ""
end try

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
