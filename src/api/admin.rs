use crate::state::AppState;
use std::sync::Arc;
use axum::{
    response::{Html, IntoResponse, Json},
    http::{StatusCode, HeaderMap, header},
    extract::State,
};
use std::fs;
use serde_json::json;

pub async fn get_system_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let uptime = state.start_time.elapsed().as_secs();
    let total_mem = sys.total_memory() / 1024 / 1024; // MB
    let used_mem = sys.used_memory() / 1024 / 1024; // MB
    let cpu_usage = sys.global_cpu_usage();

    Json(json!({
        "uptime_seconds": uptime,
        "memory_used_mb": used_mem,
        "memory_total_mb": total_mem,
        "cpu_usage_percent": cpu_usage
    })).into_response()
}

// Helper to get log path
const LOG_PATH: &str = "server.log";
const ADMIN_PASSWORD: &str = "admin";

pub async fn dashboard() -> impl IntoResponse {
    match fs::read_to_string("static/dashboard.html") {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Dashboard not found").into_response(),
    }
}

use axum::extract::Query;
use serde::{Deserialize, Serialize};

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

fn default_page() -> usize { 1 }
fn default_page_size() -> usize { 50 }
fn default_sort_by() -> String { "timestamp".to_string() }
fn default_order() -> String { "desc".to_string() }

#[derive(Debug, Serialize, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub ip: String,
    pub device: String,
    pub action: String,
    pub count: u32,
    pub raw: String,
}

use std::collections::HashSet;

#[derive(Debug, Serialize)]
pub struct LogsResponse {
    pub data: Vec<LogEntry>,
    pub meta: LogMetadata,
    pub stats: LogStats,
}

#[derive(Debug, Serialize)]
pub struct LogMetadata {
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

#[derive(Debug, Serialize)]
pub struct LogStats {
    pub unique_ips: usize,
    pub active_users: u32,
    pub last_activity: String,
}

pub async fn get_logs(Query(params): Query<LogQuery>) -> impl IntoResponse {
    let content = match fs::read_to_string(LOG_PATH) {
        Ok(c) => c,
        Err(_) => return Json(LogsResponse {
            data: vec![],
            meta: LogMetadata { total: 0, page: 1, page_size: params.page_size, total_pages: 0 },
            stats: LogStats { unique_ips: 0, active_users: 0, last_activity: "-".to_string() }
        }).into_response(),
    };

    let mut all_logs: Vec<LogEntry> = content.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() < 5 { return None; }
            Some(LogEntry {
                timestamp: parts[0].to_string(),
                ip: parts[1].to_string(),
                device: parts[2].to_string(),
                action: parts[3].to_string(),
                count: parts[4].parse().unwrap_or(0),
                raw: line.to_string(),
            })
        })
        .collect();

    // Calculate Global Stats (before filtering?)
    // Usually stats are on "All Logs" or "Filtered Logs"? 
    // Dashboard had "Total Events" which matches Table. 
    // "Unique IPs" matched Table? Or All?
    // Let's assume metrics reflect the CURRENT VIEW (Filtered).
    // EXCEPT "Active Users" usually means system state.
    // But previously: `document.getElementById('metric-total').textContent = allLogs.length;`
    // So it was based on *fetched* logs. 
    // If we filter in backend, `total` will be filtered count.
    // Let's calculate stats on the FILTERED set.

    // Filter
    if let Some(q) = &params.q {
        let q = q.to_lowercase();
        all_logs.retain(|log| {
            log.timestamp.to_lowercase().contains(&q) ||
            log.ip.to_lowercase().contains(&q) ||
            log.device.to_lowercase().contains(&q) ||
            log.action.to_lowercase().contains(&q)
        });
    }
    
    // Stats calculation on filtered set
    let unique_ips = all_logs.iter().map(|l| &l.ip).collect::<HashSet<_>>().len();
    let (active_users, last_activity) = if let Some(last) = all_logs.last() {
        (last.count, last.timestamp.clone())
    } else {
        (0, "-".to_string())
    };

    // Sort
    all_logs.sort_by(|a, b| {
        let cmp = match params.sort_by.as_str() {
            "count" => a.count.cmp(&b.count),
            "ip" => a.ip.cmp(&b.ip),
            "device" => a.device.cmp(&b.device),
            "action" => a.action.cmp(&b.action),
            _ => a.timestamp.cmp(&b.timestamp), // default timestamp
        };
        if params.order == "asc" { cmp } else { cmp.reverse() }
    });

    // Pagination
    let total = all_logs.len();
    let total_pages = (total as f64 / params.page_size as f64).ceil() as usize;
    let start = (params.page.saturating_sub(1)) * params.page_size;
    
    let page_data = if start < total {
        all_logs.into_iter().skip(start).take(params.page_size).collect()
    } else {
        vec![]
    };

    Json(LogsResponse {
        data: page_data,
        meta: LogMetadata {
            total,
            page: params.page,
            page_size: params.page_size,
            total_pages,
        },
        stats: LogStats {
            unique_ips,
            active_users,
            last_activity,
        }
    }).into_response()
}

fn check_auth(headers: &HeaderMap) -> bool {
    headers
        .get("x-admin-password")
        .and_then(|h| h.to_str().ok())
        .map(|pwd| pwd == ADMIN_PASSWORD)
        .unwrap_or(false)
}

pub async fn clear_logs(headers: HeaderMap) -> impl IntoResponse {
    if !check_auth(&headers) {
         return (StatusCode::UNAUTHORIZED, Json(json!({"error": "Unauthorized"}))).into_response();
    }

    match fs::write(LOG_PATH, "") {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "cleared"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

pub async fn download_logs(headers: HeaderMap) -> impl IntoResponse {
    if !check_auth(&headers) {
         return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    match fs::read_to_string(LOG_PATH) {
        Ok(content) => {
             // Create csv filename with today date?
             // Simple "server_logs.csv" is fine or maybe "logs_TIMESTAMP.csv".
             // Let's stick to simple "server_logs.csv".
             ([(header::CONTENT_TYPE, "text/csv"), (header::CONTENT_DISPOSITION, "attachment; filename=\"server_logs.csv\"")], content).into_response()
        },
        Err(_) => (StatusCode::NOT_FOUND, "No logs found").into_response(), 
    }
}


