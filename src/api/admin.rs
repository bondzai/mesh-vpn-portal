use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::env;

pub async fn get_system_status(State(state): State<AppState>) -> impl IntoResponse {
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
    }))
    .into_response()
}

// Helper to get log path
use crate::domain::{LogQuery, LogsResponse};

pub async fn get_logs(
    State(state): State<AppState>,
    Query(params): Query<LogQuery>,
) -> impl IntoResponse {
    let (data, meta, stats) = state.log_repository.find_all(&params);

    Json(LogsResponse { data, meta, stats }).into_response()
}

fn check_auth(headers: &HeaderMap) -> bool {
    let expected = match env::var("ADMIN_PASSWORD") {
        Ok(v) => v,
        Err(_) => return false,
    };

    headers
        .get("x-admin-password")
        .and_then(|h| h.to_str().ok())
        .map(|pwd| pwd == expected)
        .unwrap_or(false)
}

pub async fn clear_logs(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !check_auth(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response();
    }

    match state.log_repository.clear() {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "cleared"}))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn download_logs(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !check_auth(&headers) {
        return (StatusCode::UNAUTHORIZED, "Unauthorized").into_response();
    }

    match state.log_repository.get_raw_content() {
        Ok(content) => {
            // Create csv filename with today date?
            // Simple "server_logs.csv" is fine or maybe "logs_TIMESTAMP.csv".
            // Let's stick to simple "server_logs.csv".
            (
                [
                    (header::CONTENT_TYPE, "text/csv"),
                    (
                        header::CONTENT_DISPOSITION,
                        "attachment; filename=\"server_logs.csv\"",
                    ),
                ],
                content,
            )
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "No logs found").into_response(),
    }
}

pub async fn logout_handler() -> impl IntoResponse {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = StatusCode::OK;
    response
}
