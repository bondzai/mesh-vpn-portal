use axum::{
    response::{Html, IntoResponse, Json},
    http::StatusCode,
};
use std::fs;
use serde_json::json;

// Helper to get log path
const LOG_PATH: &str = "server.log";

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

pub async fn clear_logs() -> impl IntoResponse {
    match fs::write(LOG_PATH, "") {
        Ok(_) => (StatusCode::OK, Json(json!({"status": "cleared"}))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}
