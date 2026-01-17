use serde::{Deserialize, Serialize};

pub mod logger;
pub mod repositories;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub uptime: String,
    pub cpu: String,
    pub ram: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMetrics {
    #[serde(rename = "activeUsers")]
    pub active_users: u32,
    #[serde(rename = "totalUsers")]
    pub total_users: u32,
}

// Keep DashboardStats for backward compatibility / initial render if needed,
// or reconstruct it from the two new structs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    #[serde(rename = "activeUsers")]
    pub active_users: u32,
    #[serde(rename = "totalUsers")]
    pub total_users: u32,
    pub uptime: String,
    pub cpu: String,
    pub ram: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct NavItem {
    pub label: String,
    pub href: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub ip: String,
    pub device: String,
    pub device_id: String,
    pub action: String,
    pub count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    pub raw: String,
}

#[derive(Debug, Serialize)]
pub struct LogMetadata {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub data: Vec<LogEntry>,
    pub meta: LogMetadata,
    pub stats: LogStats,
}

#[derive(Debug, Serialize)]
pub struct LogStats {
    pub unique_ips: usize,
    pub unique_device_ids: usize,
    pub active_users: u32,
    pub last_activity: String,
}

#[derive(Debug, Deserialize)]
pub struct LogQuery {
    #[serde(default = "default_page")]
    pub page: usize,
    #[serde(default = "default_page_size")]
    pub page_size: usize,
    pub q: Option<String>,
    #[serde(default = "default_sort_by")]
    pub sort_by: String,
    #[serde(default = "default_order")]
    pub order: String,
}

fn default_page() -> usize {
    1
}
fn default_page_size() -> usize {
    50
}
fn default_sort_by() -> String {
    "timestamp".to_string()
}
fn default_order() -> String {
    "desc".to_string()
}
