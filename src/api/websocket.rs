use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, ConnectInfo, Query},
    response::IntoResponse,
    http::HeaderMap,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Extract User-Agent
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown Device")
        .to_string();

    // Extract Device ID
    let device_id = params.get("device_id").cloned().unwrap_or_else(|| "unknown-id".to_string());

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
        
    ws.on_upgrade(move |socket| handle_socket(socket, state, ip, user_agent, device_id))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, ip: String, device: String, device_id: String) {
    // 1. Client connected: increment count and notify everyone
    state.join(&ip, &device, &device_id);

    // 2. Subscribe to broadcast updates
    let mut rx = state.tx.subscribe();

    // 3. Send initial state immediately
    let initial_count = state.get_active_count();
    let initial_msg = serde_json::to_string(&crate::domain::UserStats {
        active_users: initial_count,
        total_users: initial_count,
    }).unwrap();
    
    if socket.send(Message::Text(initial_msg.into())).await.is_err() {
        state.leave(&ip, &device, &device_id);
        return;
    }

    // 4. Listen for updates OR client disconnect
    loop {
        tokio::select! {
            // Receive update from channel
            Ok(msg) = rx.recv() => {
                let json = serde_json::to_string(&msg).unwrap(); // msg is UserStats, so this produces {"activeUsers":...}
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
    state.leave(&ip, &device, &device_id);
}
