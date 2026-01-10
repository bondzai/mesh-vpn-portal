use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, ConnectInfo},
    response::IntoResponse,
    http::HeaderMap,
};
use std::net::SocketAddr;
use std::sync::Arc;
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Extract User-Agent
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown Device")
        .to_string();

    // Extract Real IP (X-Forwarded-For > X-Real-IP > ConnectInfo)
    let ip = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.split(',').next().unwrap_or(s).trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| addr.ip().to_string());
        
    ws.on_upgrade(move |socket| handle_socket(socket, state, ip, user_agent))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>, ip: String, device: String) {
    // 1. Client connected: increment count and notify everyone
    state.join(&ip, &device);

    // 2. Subscribe to broadcast updates
    let mut rx = state.tx.subscribe();

    // 3. Send initial state immediately
    let initial_count = state.active_users.load(std::sync::atomic::Ordering::Relaxed);
    let initial_msg = serde_json::to_string(&crate::domain::UserStats {
        active_users: initial_count,
        total_users: initial_count,
    }).unwrap();
    
    if socket.send(Message::Text(initial_msg.into())).await.is_err() {
        state.leave(&ip, &device);
        return;
    }

    // 4. Listen for updates OR client disconnect
    loop {
        tokio::select! {
            // Receive update from channel
            Ok(msg) = rx.recv() => {
                let json = serde_json::to_string(&msg).unwrap();
                if socket.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
            // Receive message from client (ignore or handle close)
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(_)) => {}, // ignore incoming messages for now
                    _ => break, // disconnect on error or close
                }
            }
        }
    }

    // 5. Client disconnected: decrement count and notify everyone
    state.leave(&ip, &device);
}
