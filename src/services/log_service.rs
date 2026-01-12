use std::fs;
use std::collections::HashSet;
use crate::domain::{LogEntry, LogMetadata, LogStats, LogQuery};

const LOG_PATH: &str = "server.log";

pub fn fetch_logs(params: &LogQuery) -> (Vec<LogEntry>, LogMetadata, LogStats) {
    let content = match fs::read_to_string(LOG_PATH) {
        Ok(c) => c,
        Err(_) => return (
            vec![], 
            LogMetadata { total: 0, page: 1, page_size: params.page_size, total_pages: 0 }, 
            LogStats { unique_ips: 0, active_users: 0, last_activity: "-".to_string() }
        ),
    };

    let mut all_logs: Vec<LogEntry> = content.lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split(',').collect();
            // Handle legacy (5 entries) and new (6 entries) formats
            if parts.len() == 6 {
                Some(LogEntry {
                    timestamp: parts[0].to_string(),
                    ip: parts[1].to_string(),
                    device: parts[2].to_string(),
                    device_id: parts[3].to_string(),
                    action: parts[4].to_string(),
                    count: parts[5].parse().unwrap_or(0),
                    raw: line.to_string(),
                })
            } else if parts.len() == 5 { // Legacy fallback
                Some(LogEntry {
                    timestamp: parts[0].to_string(),
                    ip: parts[1].to_string(),
                    device: parts[2].to_string(),
                    device_id: "N/A".to_string(),
                    action: parts[3].to_string(),
                    count: parts[4].parse().unwrap_or(0),
                    raw: line.to_string(),
                })
            } else {
                None
            }
        })
        .collect();

    // Filter
    if let Some(q) = &params.q {
        let q = q.to_lowercase();
        all_logs.retain(|log| {
            log.timestamp.to_lowercase().contains(&q) ||
            log.ip.to_lowercase().contains(&q) ||
            log.device.to_lowercase().contains(&q) ||
            log.device_id.to_lowercase().contains(&q) ||
            log.action.to_lowercase().contains(&q)
        });
    }
    
    // Stats calculation on filtered set
    let unique_ips = all_logs.iter().map(|l| &l.ip).collect::<HashSet<_>>().len();
    let (active_users, last_activity) = if let Some(last) = all_logs.last() {
        (last.count, last.timestamp.clone())
    } else {
        (0, "-".to_string())
    };

    // Sort
    all_logs.sort_by(|a, b| {
        let cmp = match params.sort_by.as_str() {
            "count" => a.count.cmp(&b.count),
            "ip" => a.ip.cmp(&b.ip),
            "device" => a.device.cmp(&b.device),
            "device_id" => a.device_id.cmp(&b.device_id),
            "action" => a.action.cmp(&b.action),
            _ => a.timestamp.cmp(&b.timestamp), // default timestamp
        };
        if params.order == "asc" { cmp } else { cmp.reverse() }
    });

    // Pagination
    let total = all_logs.len();
    let total_pages = (total as f64 / params.page_size as f64).ceil() as usize;
    let start = (params.page.saturating_sub(1)) * params.page_size;
    
    let page_data = if start < total {
        all_logs.into_iter().skip(start).take(params.page_size).collect()
    } else {
        vec![]
    };

    (
        page_data,
        LogMetadata {
            total,
            page: params.page,
            page_size: params.page_size,
            total_pages,
        },
        LogStats {
            unique_ips,
            active_users,
            last_activity,
        }
    )
}
