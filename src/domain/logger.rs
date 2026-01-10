pub trait EventLogger: Send + Sync {
    fn log(&self, ip: &str, device: &str, action: &str, count: u32);
}
