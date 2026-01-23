use crate::domain::repositories::LogRepository;
use crate::domain::{LogEntry, LogMetadata, LogQuery, LogStats};
use std::collections::HashSet;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

pub struct FileLogRepository {
    path: String,
    // We keep a mutex for writing only, assuming reads can be concurrent/loose or also protected.
    // However, LogService was stateless with reads.
    // For now, let's keep the write mutex if we want to ensure thread safety on writes inside this repo.
    write_lock: Mutex<()>,
}

impl FileLogRepository {
    pub fn new(path: &str) -> Self {
        // Ensure file exists
        if !std::path::Path::new(path).exists() {
            fs::File::create(path).unwrap();
        }

        Self {
            path: path.to_string(),
            write_lock: Mutex::new(()),
        }
    }

    fn shorten_device(device: &str) -> String {
        if let Some(start) = device.find('(') {
            if let Some(end) = device[start..].find(')') {
                return device[start + 1..start + end].to_string();
            }
        }
        device.chars().take(30).collect()
    }
}

impl LogRepository for FileLogRepository {
    fn append(&self, entry: &LogEntry) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _lock = self.write_lock.lock().unwrap();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;

        let short_device = Self::shorten_device(&entry.device);
        let sanitized_device = short_device.replace(",", " ");

        // entry.timestamp is expected to be formatted already or we format it here?
        // In existing code, FileLogger formatted it.
        // Here, let's assume the caller passes a populated LogEntry, BUT for raw CSV writing we might need to re-construct it or use the struct fields.
        // Existing FileLogger::log used: Local::now(), ip, device, device_id, action, count.
        // Let's implement append to take arguments or assume entry has them.
        // LogEntry has strings.

        // Wait, existing FileLogger generated the timestamp.
        // If we move logic here, this repo should probably handle the "append raw line" or "append structured".
        // Let's stick to the trait: append(&LogEntry).

        let duration_str = entry.duration.clone().unwrap_or_default();

        writeln!(
            file,
            "{},{},{},{},{},{},{}",
            entry.timestamp,
            entry.ip,
            sanitized_device,
            entry.device_id,
            entry.action,
            entry.count,
            duration_str
        )?;

        Ok(())
    }

    fn find_all(&self, params: &LogQuery) -> (Vec<LogEntry>, LogMetadata, LogStats) {
        let file = match fs::File::open(&self.path) {
            Ok(f) => f,
            Err(_) => {
                return (
                    vec![],
                    LogMetadata {
                        total: 0,
                        page: 1,
                        page_size: params.page_size,
                        total_pages: 0,
                    },
                    LogStats {
                        unique_ips: 0,
                        unique_device_ids: 0,
                        active_users: 0,
                        last_activity: "-".to_string(),
                        top_ips: vec![],
                        requests_over_time: vec![],
                    },
                );
            }
        };

        let reader = std::io::BufReader::new(file);
        use std::collections::HashMap;
        use std::io::BufRead;

        let mut all_logs = Vec::new();
        let mut ip_counts: HashMap<String, u32> = HashMap::new();
        let mut unique_device_ids = HashSet::new();
        let mut hourly_counts: HashMap<String, u32> = HashMap::new(); // Key: YYYY-MM-DD HH:00

        let q_lower = params.q.as_ref().map(|s| s.to_lowercase());
        let exclude_ips: Option<Vec<String>> = params.exclude_ip.as_ref().map(|s| {
            s.split(',')
                .map(|part| part.trim().to_lowercase())
                .filter(|part| !part.is_empty())
                .collect()
        });

        for line_res in reader.lines() {
            if let Ok(line) = line_res {
                let parts: Vec<&str> = line.split(',').collect();
                let len = parts.len();

                let entry = if len >= 6 {
                    let duration = if len >= 7 && !parts[6].is_empty() {
                        Some(parts[6].to_string())
                    } else {
                        None
                    };
                    Some(LogEntry {
                        timestamp: parts[0].to_string(),
                        ip: parts[1].to_string(),
                        device: parts[2].to_string(),
                        device_id: parts[3].to_string(),
                        action: parts[4].to_string(),
                        count: parts[5].parse().unwrap_or(0),
                        duration,
                        raw: line.clone(),
                    })
                } else if len == 5 {
                    Some(LogEntry {
                        timestamp: parts[0].to_string(),
                        ip: parts[1].to_string(),
                        device: parts[2].to_string(),
                        device_id: "N/A".to_string(),
                        action: parts[3].to_string(),
                        count: parts[4].parse().unwrap_or(0),
                        duration: None,
                        raw: line.clone(),
                    })
                } else {
                    None
                };

                if let Some(log) = entry {
                    // Filter
                    if let Some(ex_list) = &exclude_ips {
                       let log_ip_lower = log.ip.to_lowercase();
                       if ex_list.iter().any(|ex| log_ip_lower.contains(ex)) {
                           continue;
                       }
                    }

                    if let Some(q) = &q_lower {
                        if !log.timestamp.to_lowercase().contains(q)
                            && !log.ip.to_lowercase().contains(q)
                            && !log.device.to_lowercase().contains(q)
                            && !log.device_id.to_lowercase().contains(q)
                            && !log.action.to_lowercase().contains(q)
                        {
                            continue;
                        }
                    }

                    // Collect Stats
                    *ip_counts.entry(log.ip.clone()).or_insert(0) += 1;
                    unique_device_ids.insert(log.device_id.clone());

                    // Hourly stats for chart (simple approximation)
                    // Assuming timestamp format YYYY-MM-DD HH:MM:SS ...
                    if log.timestamp.len() >= 13 {
                        let hour_key = format!("{}:00", &log.timestamp[..13]);
                        *hourly_counts.entry(hour_key).or_insert(0) += 1;
                    }

                    all_logs.push(log);
                }
            }
        }

        // Stats calculation
        let unique_ips = ip_counts.len();
        let unique_device_ids_count = unique_device_ids.len();
        let (active_users, last_activity) = if let Some(last) = all_logs.last() {
            (last.count, last.timestamp.clone())
        } else {
            (0, "-".to_string())
        };

        // Top IPs
        let mut top_ips_vec: Vec<(String, u32)> = ip_counts.into_iter().collect();
        top_ips_vec.sort_by(|a, b| b.1.cmp(&a.1));
        top_ips_vec.truncate(10); // Top 10

        // Chart Data (sorted by time)
        let mut requests_over_time: Vec<(String, u32)> = hourly_counts.into_iter().collect();
        requests_over_time.sort_by(|a, b| a.0.cmp(&b.0));
        // Limit chart points if too many? For now keep all.

        // Sort Logs
        all_logs.sort_by(|a, b| {
            let cmp = match params.sort_by.as_str() {
                "count" => a.count.cmp(&b.count),
                "ip" => a.ip.cmp(&b.ip),
                "device" => a.device.cmp(&b.device),
                "device_id" => a.device_id.cmp(&b.device_id),
                "action" => a.action.cmp(&b.action),
                _ => a.timestamp.cmp(&b.timestamp),
            };
            if params.order == "asc" {
                cmp
            } else {
                cmp.reverse()
            }
        });

        // Pagination
        let total = all_logs.len();
        let total_pages = if params.page_size > 0 {
            (total as f64 / params.page_size as f64).ceil() as usize
        } else {
            0
        };
        let start = (params.page.saturating_sub(1)) * params.page_size;

        let page_data = if start < total {
            all_logs
                .into_iter()
                .skip(start)
                .take(params.page_size)
                .collect()
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
                unique_device_ids: unique_device_ids_count,
                active_users,
                last_activity,
                top_ips: top_ips_vec,
                requests_over_time,
            },
        )
    }

    fn clear(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let _lock = self.write_lock.lock().unwrap();
        fs::write(&self.path, "")?;
        Ok(())
    }

    fn get_raw_content(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        let content = fs::read_to_string(&self.path)?;
        Ok(content)
    }
}
