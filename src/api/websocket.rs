use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, ConnectInfo, Query},
    response::IntoResponse,
    http::HeaderMap,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use crate::state::AppState;

pub async fn client_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let (ip, device, device_id) = extract_connection_info(headers, params, addr);
    ws.on_upgrade(move |socket| handle_user_socket(socket, state, ip, device, device_id))
}

pub async fn admin_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let (ip, device, device_id) = extract_connection_info(headers, params, addr);
    ws.on_upgrade(move |socket| handle_system_socket(socket, state, ip, device, device_id))
}

fn extract_connection_info(
    headers: HeaderMap, 
    params: HashMap<String, String>, 
    addr: SocketAddr
) -> (String, String, String) {
    // Extract User-Agent
    let device = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("Unknown Device")
        .to_string();

    // Extract Device ID
    let device_id = params.get("device_id").cloned().unwrap_or_else(|| {
        use rand::Rng;
        let mut rng = rand::rng(); 
        let id: u32 = rng.random();
        format!("anon-{}", id)
    });

    // Extract Real IP
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

    (ip, device, device_id)
}

async fn handle_system_socket(mut socket: WebSocket, state: AppState, ip: String, device: String, device_id: String) {
    // 1. Client connected
    state.join(&ip, &device, &device_id);

    // 2. Subscribe to SYSTEM updates
    let mut rx = state.system_tx.subscribe();

    // 3. Send initial state immediately
    let stats = state.get_system_metrics();
    let initial_msg = serde_json::to_string(&stats).unwrap();
    
    if socket.send(Message::Text(initial_msg.into())).await.is_err() {
        state.leave(&ip, &device, &device_id);
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
                    Some(Ok(_)) => {}, 
                    _ => break, 
                }
            }
        }
    }

    // 5. Client disconnected
    state.leave(&ip, &device, &device_id);
}

async fn handle_user_socket(mut socket: WebSocket, state: AppState, ip: String, device: String, device_id: String) {
    // 1. Client connected
    state.join(&ip, &device, &device_id);

    // 2. Subscribe to USER updates
    let mut rx = state.users_tx.subscribe();

    // 3. Send initial state immediately
    let stats = state.get_user_metrics();
    let initial_msg = serde_json::to_string(&stats).unwrap();
    
    if socket.send(Message::Text(initial_msg.into())).await.is_err() {
        state.leave(&ip, &device, &device_id);
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
                    Some(Ok(_)) => {},
                    _ => break,
                }
            }
        }
    }

    // 5. Client disconnected
    state.leave(&ip, &device, &device_id);
}
