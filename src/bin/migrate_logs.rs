use std::collections::HashMap;
use std::fs;
use std::io::Write;
use chrono::{DateTime, Duration};

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: String,
    ip: String,
    device: String,
    device_id: String,
    action: String,
    count: u32,
    duration: Option<String>,
    raw: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = "server.log";
    if !std::path::Path::new(path).exists() {
        println!("No log file found at {}", path);
        return Ok(());
    }

    let content = fs::read_to_string(path)?;
    let mut logs: Vec<LogEntry> = content.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            let len = parts.len();
            if len >= 6 {
                let duration = if len >= 7 && !parts[6].is_empty() { Some(parts[6].to_string()) } else { None };
                Some(LogEntry {
                    timestamp: parts[0].to_string(),
                    ip: parts[1].to_string(),
                    device: parts[2].to_string(),
                    device_id: parts[3].to_string(),
                    action: parts[4].to_string(),
                    count: parts[5].parse().unwrap_or(0),
                    duration,
                    raw: line.to_string(),
                })
            } else if len == 5 { // Handle old format without device_id
                 Some(LogEntry {
                    timestamp: parts[0].to_string(),
                    ip: parts[1].to_string(),
                    device: parts[2].to_string(),
                    device_id: "N/A".to_string(),
                    action: parts[3].to_string(),
                    count: parts[4].parse().unwrap_or(0),
                    duration: None,
                    raw: line.to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    // Sort by timestamp properly to ensure chronological processing
    logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    let mut start_times: HashMap<String, String> = HashMap::new();
    let mut updated_logs: Vec<LogEntry> = Vec::new();

    // Process logs to calculate duration
    for mut log in logs {
        if log.action == "CONNECTED" {
             // Store start timestamp
             start_times.insert(log.device_id.clone(), log.timestamp.clone());
        } else if log.action == "DISCONNECTED" {
            if let Some(start_ts) = start_times.remove(&log.device_id) {
                // Determine format
                 // Assuming format: %Y-%m-%d %H:%M:%S %z
                 // e.g. 2026-01-13 03:10:05 +0700
                 if let Ok(start) = DateTime::parse_from_str(&start_ts, "%Y-%m-%d %H:%M:%S %z") {
                     if let Ok(end) = DateTime::parse_from_str(&log.timestamp, "%Y-%m-%d %H:%M:%S %z") {
                         let duration = end.signed_duration_since(start);
                         let secs = duration.num_seconds();
                         if secs >= 0 {
                             let formatted = if secs < 60 {
                                 format!("{}s", secs)
                             } else if secs < 3600 {
                                 format!("{}m {}s", secs / 60, secs % 60)
                             } else {
                                 format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
                             };
                             log.duration = Some(formatted);
                         }
                     }
                 }
            }
        }
        updated_logs.push(log);
    }

    // Write back to file
    let count = updated_logs.len();
    let mut file = fs::File::create(path)?;
    for log in updated_logs {
        let duration_str = log.duration.unwrap_or_default();
        
        writeln!(file, "{},{},{},{},{},{},{}", 
            log.timestamp, 
            log.ip, 
            log.device, 
            log.device_id, 
            log.action, 
            log.count,
            duration_str
        )?;
    }

    println!("Migration complete. Processed {} logs.", count);
    Ok(())
}
