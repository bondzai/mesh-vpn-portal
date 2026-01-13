use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;
use std::sync::Arc;

mod domain;
mod api;
mod state;
mod infrastructure;
mod services;
mod repositories;

use state::AppState;
use infrastructure::file_logger::FileLogger;
use repositories::log_repository::FileLogRepository;

pub mod admin {
  pub use crate::api::admin::*;
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let logger = Arc::new(FileLogger::new("server.log"));
    let log_repo = Arc::new(FileLogRepository::new("server.log"));
    let app_state = AppState::new(logger, log_repo);
    
    // Spawn background task to broadcast system stats
    let app_state_for_task = app_state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        loop {
            interval.tick().await;
            let stats = app_state_for_task.get_dashboard_stats();
            let _ = app_state_for_task.tx.send(stats);
        }
    });

    let app = Router::new()
        .route("/health", get(api::health::health_check))
        .route("/login", get(api::auth::login_page).post(api::auth::login_submit))
        .route("/logout", get(api::auth::logout))
        .route("/ws", get(api::websocket::ws_handler))
        .merge(
            Router::new()
                .route("/admin", get(api::htmx::dashboard_handler))
                .route("/htmx/overview", get(api::htmx::overview_tab_handler))
                .route("/htmx/logs-tab", get(api::htmx::logs_tab_handler))
                .route("/htmx/logs", get(api::htmx::logs_handler)) // Keep for table pagination
                .route("/api/logs", get(api::admin::get_logs).delete(api::admin::clear_logs))
                .route("/api/export", get(api::admin::download_logs))
                .route("/api/status", get(api::admin::get_system_status))
                .route_layer(axum::middleware::from_fn_with_state(app_state.clone(), api::middleware::auth))
        )
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    
    // Axum 0.8 uses axum::serve
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
