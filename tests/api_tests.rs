use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::test]
async fn test_health_check() {
    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:3000/health")
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());
    let text = res.text().await.expect("Failed to get text");
    assert_eq!(text, "OK");
}

#[tokio::test]
async fn test_client_ws_connection() {
    let url = "ws://localhost:3000/client/ws?device_id=test-client";
    let (mut socket, response) = connect_async(url).await.expect("Failed to connect");

    assert_eq!(response.status(), 101);

    // Should receive initial message immediately
    if let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            assert!(text.contains("activeUsers"));
            assert!(text.contains("totalUsers"));
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("Socket closed unexpectedy");
    }
}

#[tokio::test]
async fn test_admin_ws_connection() {
    let url = "ws://localhost:3000/admin/ws?device_id=test-admin";
    let (mut socket, response) = connect_async(url).await.expect("Failed to connect");

    assert_eq!(response.status(), 101);

    // Should receive initial message immediately
    if let Some(Ok(msg)) = socket.next().await {
        if let Message::Text(text) = msg {
            assert!(text.contains("uptime"));
            assert!(text.contains("cpu"));
            assert!(text.contains("ram"));
        } else {
            panic!("Expected text message");
        }
    } else {
        panic!("Socket closed unexpectedy");
    }
}

// Helper to allow stream iteration
use futures_util::StreamExt;
