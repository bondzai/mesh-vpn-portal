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

use state::AppState;
use infrastructure::file_logger::FileLogger;

#[tokio::main]
async fn main() {
    let logger = Arc::new(FileLogger::new("server.log"));
    let app_state = AppState::new(logger);

    // build our application with a route
    let app = Router::new()
        .route("/health", get(api::health::health_check))
        .route("/ws", get(api::websocket::ws_handler))
        .with_state(app_state)
        .layer(CorsLayer::permissive());

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    
    // Axum 0.8 uses axum::serve
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap();
}
