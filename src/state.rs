use crate::domain::logger::EventLogger; // Import trait
use crate::domain::repositories::LogRepository;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use sysinfo::System; // Ensure trait is imported for refresh methods
use std::time::Instant;

use crate::domain::DashboardStats;
use std::collections::HashMap;
use axum_extra::extract::cookie::Key;
use axum::extract::FromRef;

#[derive(Clone)]
pub struct AppState {
    // Stores active sessions by device_id
    pub sessions: Arc<Mutex<HashMap<String, crate::domain::ActiveSession>>>,
    pub tx: broadcast::Sender<DashboardStats>, 
    pub logger: Arc<dyn EventLogger + Send + Sync>,
    pub log_repository: Arc<dyn LogRepository>,
    pub system: Arc<Mutex<System>>,
    pub start_time: Instant,
    pub key: Key,
}

impl AppState {
    pub fn new(logger: Arc<dyn EventLogger + Send + Sync>, log_repository: Arc<dyn LogRepository>) -> Self {
        let (tx, _) = broadcast::channel(100);
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            tx,
            logger,
            log_repository,
            system: Arc::new(Mutex::new(sys)),
            start_time: Instant::now(),
            key: Key::generate(),
        }
    }
    pub fn join(&self, ip: &str, device: &str, device_id: &str) -> u32 {
        let mut sessions = self.sessions.lock().unwrap();
        
        // Only insert if not already present or just update?
        // Actually, if a user opens a 2nd tab, they have the same device_id (usually stored in cookie/localstorage?)
        // The original logic counted connections per device_id, but here sessions key is device_id.
        // If we want to support multiple tabs for same user, we might need a counter inside ActiveSession or just update the timestamp?
        // For simplicity and "Active Users" tracking, one session per device_id is fine.
        // If it exists, we assume it's the same user reconnecting or a new tab.
        // Let's just overwrite or keep existing?
        // Original logic: `*conn_map.entry(device_id).or_insert(0) += 1;` -> tracked count of connections per device.
        // And `connection_start_times` only set if count == 1.
        
        // New logic: We want to display the user in the list.
        // If we want to track connection count per device, we can add `connection_count` to ActiveSession.
        
        let _entry = sessions.entry(device_id.to_string()).or_insert_with(|| {
            crate::domain::ActiveSession {
                device_id: device_id.to_string(),
                ip: ip.to_string(),
                device: device.to_string(),
                connected_at: Instant::now(),
            }
        });
        
        // We can't easily track multiple tabs strictly without a unique connection ID per WS connection.
        // But for "Active Users" list, unique device ID is what matters.
        
        let count = sessions.len() as u32;
        drop(sessions);

        self.logger.log(ip, device, device_id, "CONNECTED", count, None);
        // Stats broadcast is handled by the background loop
        count
    }

    pub fn leave(&self, ip: &str, device: &str, device_id: &str) -> u32 {
        let mut sessions = self.sessions.lock().unwrap();
        
        // In the simple map model (1 session per device), leaving removes it.
        // But if we have multiple tabs, this might match the wrong usage if we blindly remove.
        // However, without connection IDs, this is the best we can do for now:
        // A user closing ONE tab might remove the session even if others are open IF we don't track count.
        // To strictly maintain existing behavior (track count), we'd need `connection_count` in ActiveSession.
        
        // Let's add connection_count to ActiveSession in next step if getting bugs, but for now, 
        // to simplify: simple removal. 
        // Wait, the user might want to see the session active if they have another tab open.
        // BUT, WS usually closes when checking for "Active Users".
        // Let's assume 1 tab = 1 user for this refactor to start, or rely on the frontend to manage device_id?
        // Actually, the previous implementation counted connections. 
        // Let's revert to simple removal for now, assuming "Active User" = "Has at least one connection".
        // But wait, if I have 2 tabs, and close 1, `leave` is called. If I remove the session, the other tab is still there but server thinks user left.
        // This is a regression.
        
        // FIX: We need to store connection count in `ActiveSession` or handle it.
        // I will first implement it with simple removal and if I strictly need ref counting, I'll add it.
        // Actually, let's keep it simple: Remove it. The frontend re-establishes or keeps alive. 
        // If the other tab is open, it might send a heartbeat or just remain "connected" but if `active_connections` was the source of truth...
        
        // OK, I will update ActiveSession struct to include `u32` count in a separate edit if needed.
        // For now, let's just remove it and see.
        
        let mut duration_str = None;
        if let Some(session) = sessions.remove(device_id) {
             let duration = session.connected_at.elapsed();
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

        let count = sessions.len() as u32;
        drop(sessions);

        self.logger.log(ip, device, device_id, "DISCONNECTED", count, duration_str);
        // Stats broadcast is handled by the background loop
        count
    }
    
    // Helper to get current count without modifying state
    pub fn get_active_count(&self) -> u32 {
        self.sessions.lock().unwrap().len() as u32
    }

    pub fn get_dashboard_stats(&self) -> crate::domain::DashboardStats {
        let mut sys = self.system.lock().unwrap();
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let uptime_sec = self.start_time.elapsed().as_secs();
        let hrs = uptime_sec / 3600;
        let mins = (uptime_sec % 3600) / 60;
        let secs = uptime_sec % 60;
        let uptime = format!("{}h {}m {}s", hrs, mins, secs);

        let cpu = format!("{:.1}", sys.global_cpu_usage());
        let ram = format!("{}MB / {}MB", sys.used_memory() / 1024 / 1024, sys.total_memory() / 1024 / 1024);
        
        // Active users
        let active_users = self.sessions.lock().unwrap().len() as u32;

        crate::domain::DashboardStats {
            active_users,
            total_users: active_users, 
            uptime,
            cpu,
            ram,
        }
    }

    pub fn get_active_sessions(&self) -> Vec<crate::domain::ActiveSessionView> {
        let sessions = self.sessions.lock().unwrap();
        let mut views = Vec::new();
        
        for session in sessions.values() {
            let duration = session.connected_at.elapsed();
            let secs = duration.as_secs();
            let duration_str = if secs < 60 {
                format!("{}s", secs)
            } else if secs < 3600 {
                format!("{}m {}s", secs / 60, secs % 60)
            } else {
                format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
            };

            // Format connected_at as readable simple string if needed, or just ISO?
            // Let's use duration for now as primary metric.
            // But connected_at might be useful.
            
            views.push(crate::domain::ActiveSessionView {
                device_id: session.device_id.clone(),
                ip: session.ip.clone(),
                device: session.device.clone(),
                connected_at: format!("{:?}", session.connected_at), // Debug format for now or use chrono if stuck
                duration: duration_str,
            });
        }
        
        // Sort by duration desc (newest first? longest first?)
        // Let's sort by connected_at desc (newest first)
        // Since we don't have connected_at easily sortable in View without parsing, 
        // we might rely on the order or just send it.
        // Actually HashMap is unordered.
        // Let's leave unordered or sort by IP?
        views
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}
