use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use std::sync::Arc;
use crate::state::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    // 1. Client connected: increment count and notify everyone
    state.join();

    // 2. Subscribe to broadcast updates
    let mut rx = state.tx.subscribe();

    // 3. Send initial state immediately
    let initial_count = state.active_users.load(std::sync::atomic::Ordering::Relaxed);
    let initial_msg = serde_json::to_string(&crate::domain::UserStats {
        active_users: initial_count,
        total_users: initial_count,
    }).unwrap();
    
    if socket.send(Message::Text(initial_msg.into())).await.is_err() {
        state.leave();
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
    state.leave();
}
