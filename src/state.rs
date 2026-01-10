use crate::domain::logger::EventLogger; // Import trait
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::broadcast;
use sysinfo::System; // Ensure trait is imported for refresh methods
use std::time::Instant;

pub struct AppState {
    pub active_users: AtomicU32,
    pub tx: broadcast::Sender<String>,
    pub logger: Arc<dyn EventLogger + Send + Sync>,
    pub system: Arc<Mutex<System>>,
    pub start_time: Instant,
}

impl AppState {
    pub fn new(logger: Arc<dyn EventLogger + Send + Sync>) -> Self {
        let (tx, _) = broadcast::channel(100);
        
        let mut sys = System::new_all();
        sys.refresh_all();
        
        Self {
            active_users: AtomicU32::new(0),
            tx,
            logger,
            system: Arc::new(Mutex::new(sys)),
            start_time: Instant::now(),
        }
    }
    pub fn join(&self, ip: &str, device: &str) -> u32 {
        let count = self.active_users.fetch_add(1, Ordering::SeqCst) + 1;
        self.logger.log(ip, device, "CONNECTED", count);
        let _ = self.tx.send(count.to_string());
        count
    }

    pub fn leave(&self, ip: &str, device: &str) -> u32 {
        let count = self.active_users.fetch_sub(1, Ordering::SeqCst) - 1;
        self.logger.log(ip, device, "DISCONNECTED", count);
        let _ = self.tx.send(count.to_string());
        count
    }
}
