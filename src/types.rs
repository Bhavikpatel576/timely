use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: i64,
    pub device_id: String,
    pub timestamp: DateTime<Utc>,
    pub duration: f64,
    pub app: String,
    pub title: String,
    pub url: Option<String>,
    pub url_domain: Option<String>,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub is_afk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub productivity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryRule {
    pub id: i64,
    pub category_id: i64,
    pub category_name: Option<String>,
    pub field: String,
    pub pattern: String,
    pub is_builtin: bool,
    pub priority: i32,
}

#[derive(Debug, Clone)]
pub struct WatcherSnapshot {
    pub app: String,
    pub title: String,
    pub url: Option<String>,
    pub url_domain: Option<String>,
    pub is_afk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub last_sync: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryGroup {
    pub label: String,
    pub seconds: f64,
    pub time: String,
    pub percentage: f64,
    pub productivity_score: Option<f64>,
    pub event_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryResponse {
    pub from: String,
    pub to: String,
    pub total_seconds: f64,
    pub total_time: String,
    pub productivity_score: f64,
    pub groups: Vec<SummaryGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: String,
    pub duration_seconds: f64,
    pub duration_time: String,
    pub app: String,
    pub title: String,
    pub url: Option<String>,
    pub category: Option<String>,
    pub productivity_score: Option<f64>,
    pub is_afk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineResponse {
    pub from: String,
    pub to: String,
    pub count: usize,
    pub entries: Vec<TimelineEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NowResponse {
    pub app: String,
    pub title: String,
    pub url: Option<String>,
    pub category: Option<String>,
    pub productivity_score: Option<f64>,
    pub since: String,
    pub duration_seconds: f64,
    pub duration_time: String,
    pub is_afk: bool,
    pub stale: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub running: bool,
    pub pid: Option<u32>,
    pub uptime_seconds: Option<f64>,
    pub uptime_time: Option<String>,
}

// --- Web API response types (match Express dashboard shapes exactly) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBreakdown {
    pub app: String,
    pub category: String,
    pub seconds: i64,
    pub time: String,
    pub pct: f64,
    pub events: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductivityResponse {
    pub score: i64,
    pub productive: i64,
    pub neutral: i64,
    pub distracting: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendBucket {
    pub bucket: String,
    pub total_seconds: i64,
    pub total_hours: f64,
    pub productivity: i64,
    pub categories: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentActivity {
    pub app: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub category: String,
    pub duration_seconds: f64,
    pub is_afk: bool,
    pub since: String,
}

// --- Web-specific summary/timeline response types (different field names from CLI) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSummaryGroup {
    pub name: String,
    pub seconds: i64,
    pub time: String,
    pub pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSummaryResponse {
    pub period_from: String,
    pub period_to: String,
    pub total_active: String,
    pub total_active_seconds: i64,
    pub groups: Vec<WebSummaryGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebTimelineEntry {
    pub timestamp: String,
    pub duration: f64,
    pub app: Option<String>,
    pub title: Option<String>,
    pub url: Option<String>,
    pub category: String,
    pub is_afk: bool,
}

// --- Sync response types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatusResponse {
    pub sync_enabled: bool,
    pub hub_url: Option<String>,
    pub hub_reachable: bool,
    pub last_sync_at: Option<String>,
    pub pending_events: i64,
    pub devices: Vec<SyncDeviceStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDeviceStatus {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub last_sync: Option<String>,
    pub event_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPushResponse {
    pub accepted: usize,
    pub duplicates: usize,
    pub batches: usize,
}

// --- Focus analytics types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepWorkBlock {
    pub start: String,
    pub end: String,
    pub duration_seconds: f64,
    pub duration_time: String,
    pub category: String,
    pub apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistractionEntry {
    pub app: String,
    pub switches_to: u32,
    pub total_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusResponse {
    pub from: String,
    pub to: String,
    pub total_active_seconds: f64,
    pub total_active_time: String,
    pub focus_score: u32,
    pub context_switches: u32,
    pub switches_per_hour: f64,
    pub deep_work_blocks: Vec<DeepWorkBlock>,
    pub longest_focus_minutes: f64,
    pub top_distractions: Vec<DistractionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsResponse {
    pub from: String,
    pub to: String,
    pub interval: String,
    pub buckets: Vec<TrendBucket>,
}

pub fn format_duration(seconds: f64) -> String {
    let total = seconds as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
