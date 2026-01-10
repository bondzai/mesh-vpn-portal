use crate::domain::logger::EventLogger; // Import trait
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use tokio::sync::broadcast;
use crate::domain::UserStats;

pub struct AppState {
    pub active_users: AtomicU32,
    pub tx: broadcast::Sender<UserStats>,
    pub logger: Arc<dyn EventLogger>,
}

impl AppState {
    pub fn new(logger: Arc<dyn EventLogger>) -> Arc<Self> {
        let (tx, _) = broadcast::channel(100);
        Arc::new(Self {
            active_users: AtomicU32::new(0),
            tx,
            logger,
        })
    }

    pub fn join(&self, ip: &str, device: &str) {
        let count = self.active_users.fetch_add(1, Ordering::SeqCst) + 1;
        self.logger.log(ip, device, "CONNECTED", count);
        let _ = self.tx.send(UserStats {
            active_users: count,
            total_users: count, // Simplified: total = active for now
        });
    }

    pub fn leave(&self, ip: &str, device: &str) {
        let count = self.active_users.fetch_sub(1, Ordering::SeqCst) - 1;
        self.logger.log(ip, device, "DISCONNECTED", count);
        let _ = self.tx.send(UserStats {
            active_users: count,
            total_users: count,
        });
    }
}
