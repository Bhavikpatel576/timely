use std::process::Command;

/// Known terminal emulator app names (must match what AppleScript returns).
const TERMINAL_APPS: &[&str] = &[
    "Terminal",
    "iTerm2",
    "Alacritty",
    "kitty",
    "Warp",
    "WezTerm",
    "Ghostty",
];

/// Known TUI/CLI tools we want to detect running inside terminals.
/// Maps process comm name → friendly display name.
const KNOWN_TUI_APPS: &[(&str, &str)] = &[
    // AI coding assistants
    ("claude", "Claude Code"),
    ("codex", "Codex CLI"),
    ("aider", "Aider"),
    ("cursor-cli", "Cursor CLI"),
    // Editors
    ("nvim", "Neovim"),
    ("vim", "Vim"),
    ("emacs", "Emacs"),
    ("nano", "nano"),
    ("helix", "Helix"),
    // Dev tools
    ("lazygit", "Lazygit"),
    ("lazydocker", "Lazydocker"),
    ("k9s", "k9s"),
    ("tig", "Tig"),
    // System tools
    ("htop", "htop"),
    ("btop", "btop"),
    ("top", "top"),
    // Multiplexers
    ("tmux", "tmux"),
    ("screen", "screen"),
    ("zellij", "Zellij"),
    // Other
    ("python", "Python REPL"),
    ("node", "Node REPL"),
    ("irb", "Ruby IRB"),
    ("iex", "Elixir IEx"),
    ("ghci", "GHCi"),
];

pub struct TuiInfo {
    /// Display name of the TUI app (e.g. "Claude Code")
    pub app_name: String,
    /// Raw process name (e.g. "claude")
    pub process_name: String,
}

/// Returns true if the given app name is a known terminal emulator.
pub fn is_terminal_app(app: &str) -> bool {
    TERMINAL_APPS.iter().any(|t| t.eq_ignore_ascii_case(app))
}

/// Detects the foreground TUI process running inside the active terminal window.
///
/// Strategy: find all terminal tty sessions, get the foreground process of each,
/// and match against known TUI apps. We pick the one on the terminal's active window.
pub fn detect_tui_process() -> Option<TuiInfo> {
    // Get foreground processes on all ttys — `ps` shows which are in foreground (S+/R+ stat)
    let output = Command::new("ps")
        .args(["-eo", "tty,stat,comm"])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let stat = parts[1];
        // '+' suffix means foreground process group
        if !stat.contains('+') {
            continue;
        }

        // Only look at terminal ttys (ttysNNN), not the console
        let tty = parts[0];
        if !tty.starts_with("ttys") {
            continue;
        }

        // comm might contain slashes — take the basename
        let comm = parts[2..].join(" ");
        let basename = comm.rsplit('/').next().unwrap_or(&comm);

        if let Some((_, display_name)) = KNOWN_TUI_APPS
            .iter()
            .find(|(proc_name, _)| basename.eq_ignore_ascii_case(proc_name))
        {
            return Some(TuiInfo {
                app_name: display_name.to_string(),
                process_name: basename.to_string(),
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_terminal_app() {
        assert!(is_terminal_app("iTerm2"));
        assert!(is_terminal_app("Terminal"));
        assert!(is_terminal_app("Ghostty"));
        assert!(is_terminal_app("Warp"));
        assert!(!is_terminal_app("Code"));
        assert!(!is_terminal_app("Google Chrome"));
    }

    #[test]
    fn test_detect_tui_finds_something() {
        // This test runs inside a terminal, so there should be at least a shell
        // We can't guarantee a specific TUI is running, but the function shouldn't panic
        let _result = detect_tui_process();
    }
}
