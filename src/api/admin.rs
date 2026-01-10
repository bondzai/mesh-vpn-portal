use axum::{
    response::{Html, IntoResponse, Json},
    http::{StatusCode, HeaderMap, header},
};
use std::fs;
use serde_json::json;

// Helper to get log path
const LOG_PATH: &str = "server.log";
const ADMIN_PASSWORD: &str = "admin";

pub async fn dashboard() -> impl IntoResponse {
    match fs::read_to_string("static/dashboard.html") {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Dashboard not found").into_response(),
    }
}

pub async fn get_logs() -> impl IntoResponse {
    match fs::read_to_string(LOG_PATH) {
        Ok(content) => content.into_response(),
        Err(_) => "".into_response(), // Return empty if no file
    }
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


