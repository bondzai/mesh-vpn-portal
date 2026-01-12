use axum::{
    routing::get,
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

use state::AppState;
use infrastructure::file_logger::FileLogger;

pub mod admin {
  pub use crate::api::admin::*;
}

#[tokio::main]
async fn main() {
    let logger = Arc::new(FileLogger::new("server.log"));
    let app_state = Arc::new(AppState::new(logger));

    let app = Router::new()
        .route("/health", get(api::health::health_check))
        .route("/logged_out", get(api::htmx::logged_out_handler))
        .route("/logout", get(api::admin::logout_handler))
        .route("/ws", get(api::websocket::ws_handler))
        .merge(
            Router::new()
                .route("/admin", get(api::htmx::dashboard_handler))
                .route("/htmx/stats", get(api::htmx::stats_handler))
                .route("/htmx/logs", get(api::htmx::logs_handler))
                .route("/api/logs", get(api::admin::get_logs).delete(api::admin::clear_logs))
                .route("/api/export", get(api::admin::download_logs))
                .route("/api/status", get(api::admin::get_system_status))
                .route_layer(axum::middleware::from_fn(api::middleware::auth))
        )
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    
    // Axum 0.8 uses axum::serve
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
