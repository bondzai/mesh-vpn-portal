use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use crate::domain::logger::EventLogger;

pub struct FileLogger {
    file: Mutex<std::fs::File>,
}

impl FileLogger {
    pub fn new(path: &str) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("Unable to open log file");
            
        Self {
            file: Mutex::new(file),
        }
    }
}

impl EventLogger for FileLogger {
    fn log(&self, ip: &str, device: &str, action: &str, count: u32) {
        if let Ok(mut file) = self.file.lock() {
            // CSV format: ip,device,action,total_active_users
            // Sanitize device string a bit to avoid CSV breaking if it has commas?
            // For simplicity/DRY/clean requirement, trusting input or simple replace usually works.
            // Let's replace commas in device string with spaces just in case.
            let sanitized_device = device.replace(",", " ");
            if let Err(e) = writeln!(file, "{},{},{},{}", ip, sanitized_device, action, count) {
                eprintln!("Failed to write to log: {}", e);
            }
        }
    }
}
