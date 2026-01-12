use crate::state::AppState;
use std::sync::Arc;
use axum::{
    response::{Html, IntoResponse, Json, Response},
    http::{StatusCode, HeaderMap, header},
    extract::{State, Query},
    body::Body,
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

use crate::domain::{LogQuery, LogsResponse};
use crate::services::log_service;

pub async fn get_logs(Query(params): Query<LogQuery>) -> impl IntoResponse {
    let (data, meta, stats) = log_service::fetch_logs(&params);

    Json(LogsResponse {
        data,
        meta,
        stats,
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

pub async fn logout_handler() -> impl IntoResponse {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::OK;
    response
}


