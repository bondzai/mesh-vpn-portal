use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use crate::domain::logger::EventLogger;
use chrono::Local;

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

    fn shorten_device(device: &str) -> String {
        // Simple heuristic: try to grab content inside the first parentheses (OS info usually),
        // effectively shortening the long standard Mozilla/5.0 prefix and engine suffix.
        if let Some(start) = device.find('(') {
            if let Some(end) = device[start..].find(')') {
                return device[start+1..start+end].to_string();
            }
        }
        // Fallback: truncate to 30 chars
        device.chars().take(30).collect()
    }
}

impl EventLogger for FileLogger {
    fn log(&self, ip: &str, device: &str, action: &str, count: u32) {
        if let Ok(mut file) = self.file.lock() {
            let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S %z");
            let short_device = Self::shorten_device(device);
            // Sanitize commas in device string
            let sanitized_device = short_device.replace(",", " ");
            
            if let Err(e) = writeln!(file, "{},{},{},{},{}", timestamp, ip, sanitized_device, action, count) {
                eprintln!("Failed to write to log: {}", e);
            }
        }
    }
}
