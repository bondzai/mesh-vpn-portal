use crate::domain::logger::EventLogger; // Import trait
use crate::domain::repositories::LogRepository;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Instant;
use sysinfo::System; // Ensure trait is imported for refresh methods
use tokio::sync::broadcast;
use crate::services::wakatime::WakatimeData;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ActiveConnection {
    pub ip: String,
    pub device: String,
    pub device_id: String,
    pub connected_at: Instant,
}

#[derive(Clone)]
pub struct AppState {
    pub active_connections: Arc<Mutex<HashMap<String, ActiveConnection>>>,
    pub system_tx: broadcast::Sender<crate::domain::SystemMetrics>,
    pub users_tx: broadcast::Sender<crate::domain::UserMetrics>,
    pub logger: Arc<dyn EventLogger + Send + Sync>,
    pub log_repository: Arc<dyn LogRepository>,
    pub system: Arc<Mutex<System>>,
    pub start_time: Instant,
    pub key: Key,
    pub wakatime_data: Arc<RwLock<Option<WakatimeData>>>,
}

impl AppState {
    pub fn new(
        logger: Arc<dyn EventLogger + Send + Sync>,
        log_repository: Arc<dyn LogRepository>,
    ) -> Self {
        let (system_tx, _) = broadcast::channel(100);
        let (users_tx, _) = broadcast::channel(100);

        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            active_connections: Arc::new(Mutex::new(HashMap::new())),
            system_tx,
            users_tx,
            logger,
            log_repository,
            system: Arc::new(Mutex::new(sys)),
            start_time: Instant::now(),
            key: Key::generate(),
            wakatime_data: Arc::new(RwLock::new(None)),
        }
    }

    pub fn join(&self, ip: &str, device: &str, device_id: &str) -> u32 {
        let mut conn_map = self.active_connections.lock().unwrap();
        conn_map.insert(
            device_id.to_string(),
            ActiveConnection {
                ip: ip.to_string(),
                device: device.to_string(),
                device_id: device_id.to_string(),
                connected_at: Instant::now(),
            },
        );
        let count = conn_map.len() as u32;
        drop(conn_map);

        self.logger
            .log(ip, device, device_id, "CONNECTED", count, None);

        // Notify user stream
        let _ = self.users_tx.send(self.get_user_metrics());

        count
    }

    pub fn leave(&self, ip: &str, device: &str, device_id: &str) -> u32 {
        let mut conn_map = self.active_connections.lock().unwrap();
        let mut duration_str = None;

        if let Some(conn) = conn_map.remove(device_id) {
            let duration = conn.connected_at.elapsed();
            let secs = duration.as_secs();
            let formatted = crate::utils::format_duration(secs);
            duration_str = Some(formatted);
        }

        let count = conn_map.len() as u32;
        drop(conn_map);

        self.logger
            .log(ip, device, device_id, "DISCONNECTED", count, duration_str);

        // Notify user stream
        let _ = self.users_tx.send(self.get_user_metrics());

        count
    }

    // Helper to get current count without modifying state
    pub fn get_active_count(&self) -> u32 {
        self.active_connections.lock().unwrap().len() as u32
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
        let ram = format!(
            "{}MB / {}MB",
            sys.used_memory() / 1024 / 1024,
            sys.total_memory() / 1024 / 1024
        );

        // Active users (exclude admin dashboard)
        let active_users = self
            .active_connections
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.device_id != "admin-dashboard")
            .count() as u32;

        crate::domain::DashboardStats {
            active_users,
            total_users: active_users,
            uptime,
            cpu,
            ram,
        }
    }

    pub fn get_system_metrics(&self) -> crate::domain::SystemMetrics {
        let mut sys = self.system.lock().unwrap();
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let uptime_sec = self.start_time.elapsed().as_secs();
        let hrs = uptime_sec / 3600;
        let mins = (uptime_sec % 3600) / 60;
        let secs = uptime_sec % 60;
        let uptime = format!("{}h {}m {}s", hrs, mins, secs);

        let cpu = format!("{:.1}", sys.global_cpu_usage());
        let ram = format!(
            "{}MB / {}MB",
            sys.used_memory() / 1024 / 1024,
            sys.total_memory() / 1024 / 1024
        );

        crate::domain::SystemMetrics { uptime, cpu, ram }
    }

    pub fn get_user_metrics(&self) -> crate::domain::UserMetrics {
        let active_users = self
            .active_connections
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.device_id != "admin-dashboard")
            .count() as u32;

        crate::domain::UserMetrics {
            active_users,
            total_users: active_users,
        }
    }

    pub fn get_active_users(&self) -> Vec<ActiveConnection> {
        self.active_connections
            .lock()
            .unwrap()
            .values()
            .filter(|c| c.device_id != "admin-dashboard")
            .cloned()
            .collect()
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}
