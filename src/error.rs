use thiserror::Error;

#[derive(Debug, Error)]
pub enum TimelyError {
    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Config(String),

    #[error("Daemon not running")]
    DaemonNotRunning,

    #[error("Daemon already running (pid {0})")]
    DaemonAlreadyRunning(u32),

    #[error("No data for the requested time range. Is the daemon running? Check: timely daemon status")]
    NoData,

    #[error("Invalid time range: {0}")]
    InvalidTimeRange(String),

    #[error("Category not found: {0}")]
    CategoryNotFound(String),

    #[error("Rule not found: {0}")]
    RuleNotFound(i64),

    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),

    #[error("Sync error: {0}")]
    Sync(String),

    #[error("{0}")]
    Generic(String),
}

impl TimelyError {
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Db(_) => "db_error",
            Self::Io(_) => "io_error",
            Self::Json(_) => "json_error",
            Self::Config(_) => "config_error",
            Self::DaemonNotRunning => "daemon_not_running",
            Self::DaemonAlreadyRunning(_) => "daemon_already_running",
            Self::NoData => "no_data",
            Self::InvalidTimeRange(_) => "invalid_time_range",
            Self::CategoryNotFound(_) => "category_not_found",
            Self::RuleNotFound(_) => "rule_not_found",
            Self::PlatformNotSupported(_) => "platform_not_supported",
            Self::Sync(_) => "sync_error",
            Self::Generic(_) => "error",
        }
    }
}

pub type Result<T> = std::result::Result<T, TimelyError>;
