pub trait EventLogger: Send + Sync {
    fn log(&self, ip: &str, device: &str, device_id: &str, action: &str, count: u32, duration: Option<String>);
}
