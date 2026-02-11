use crate::error::Result;
use std::process::Command;

pub struct BrowserTab {
    pub url: String,
    pub title: String,
    pub domain: String,
}

pub fn get_browser_tab(app: &str) -> Result<Option<BrowserTab>> {
    let script = match app {
        "Google Chrome" | "Chromium" | "Brave Browser" | "Microsoft Edge" | "Vivaldi" | "Arc" => {
            format!(
                r#"tell application "{}"
    try
        set tabUrl to URL of active tab of front window
        set tabTitle to title of active tab of front window
        return tabUrl & "|" & tabTitle
    on error
        return ""
    end try
end tell"#,
                app
            )
        }
        "Safari" => {
            r#"tell application "Safari"
    try
        set tabUrl to URL of front document
        set tabTitle to name of front document
        return tabUrl & "|" & tabTitle
    on error
        return ""
    end try
end tell"#
                .to_string()
        }
        _ => return Ok(None),
    };

    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() {
        return Ok(None);
    }

    if let Some((url, title)) = stdout.split_once('|') {
        let domain = extract_domain(url);
        Ok(Some(BrowserTab {
            url: url.to_string(),
            title: title.to_string(),
            domain,
        }))
    } else {
        Ok(None)
    }
}

fn extract_domain(url: &str) -> String {
    // Simple domain extraction without pulling in the url crate
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme
        .split('/')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_string()
}
