use crate::domain::logger::EventLogger; // Import trait
use crate::domain::repositories::LogRepository;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use sysinfo::System; // Ensure trait is imported for refresh methods
use std::time::Instant;

use crate::domain::UserStats;
use std::collections::HashMap;
use axum_extra::extract::cookie::Key;
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    pub active_connections: Arc<Mutex<HashMap<String, u32>>>,
    pub tx: broadcast::Sender<UserStats>, // Changed from String
    pub logger: Arc<dyn EventLogger + Send + Sync>,
    pub log_repository: Arc<dyn LogRepository>,
    pub system: Arc<Mutex<System>>,
    pub start_time: Instant,
    pub connection_start_times: Arc<Mutex<HashMap<String, Instant>>>,
    pub key: Key,
}

impl AppState {
    pub fn new(logger: Arc<dyn EventLogger + Send + Sync>, log_repository: Arc<dyn LogRepository>) -> Self {
        let (tx, _) = broadcast::channel(100);
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        Self {
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            tx,
            logger,
            log_repository,
            system: Arc::new(Mutex::new(sys)),
            start_time: Instant::now(),
            connection_start_times: Arc::new(Mutex::new(HashMap::new())),
            key: Key::generate(),
        }
    }
    pub fn join(&self, ip: &str, device: &str, device_id: &str) -> u32 {
        let mut conn_map = self.active_connections.lock().unwrap();
        *conn_map.entry(device_id.to_string()).or_insert(0) += 1;
        let count = conn_map.len() as u32;
        drop(conn_map); // Unlock before logging/sending to avoid blocking

        if count == 1 {
            let mut start_times = self.connection_start_times.lock().unwrap();
            start_times.insert(device_id.to_string(), Instant::now());
        }

        self.logger.log(ip, device, device_id, "CONNECTED", count, None);
        let _ = self.tx.send(UserStats { active_users: count, total_users: count });
        count
    }

    pub fn leave(&self, ip: &str, device: &str, device_id: &str) -> u32 {
        let mut conn_map = self.active_connections.lock().unwrap();
        if let Some(c) = conn_map.get_mut(device_id) {
            *c -= 1;
            if *c == 0 {
                conn_map.remove(device_id);
            }
        }
        let count = conn_map.len() as u32;
        drop(conn_map);

        let mut duration_str = None;
        if count == 0 {
             let mut start_times = self.connection_start_times.lock().unwrap();
             if let Some(start) = start_times.remove(device_id) {
                 let duration = start.elapsed();
                 let secs = duration.as_secs();
                 let formatted = if secs < 60 {
                     format!("{}s", secs)
                 } else if secs < 3600 {
                     format!("{}m {}s", secs / 60, secs % 60)
                 } else {
                     format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                 };
                 duration_str = Some(formatted);
             }
        }

        self.logger.log(ip, device, device_id, "DISCONNECTED", count, duration_str);
        let _ = self.tx.send(UserStats { active_users: count, total_users: count });
        count
    }
    
    // Helper to get current count without modifying state
    pub fn get_active_count(&self) -> u32 {
        self.active_connections.lock().unwrap().len() as u32
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}
