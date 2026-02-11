#[cfg(target_os = "macos")]
pub mod afk_macos;
#[cfg(target_os = "macos")]
pub mod window_macos;
#[cfg(target_os = "macos")]
pub mod browser_macos;

use crate::error::Result;
use crate::types::WatcherSnapshot;

pub fn collect_snapshot() -> Result<WatcherSnapshot> {
    #[cfg(target_os = "macos")]
    {
        let window = window_macos::get_active_window()?;
        let is_afk = afk_macos::is_afk().unwrap_or(false);

        let (url, url_domain) = match browser_macos::get_browser_tab(&window.app) {
            Ok(Some(tab)) => (Some(tab.url), Some(tab.domain)),
            _ => (None, None),
        };

        Ok(WatcherSnapshot {
            app: window.app,
            title: window.title,
            url,
            url_domain,
            is_afk,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(crate::error::TimelyError::PlatformNotSupported(
            std::env::consts::OS.to_string(),
        ))
    }
}
