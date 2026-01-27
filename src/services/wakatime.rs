use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use base64::prelude::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Summary {
    pub grand_total: GrandTotal,
    pub range: Range,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrandTotal {
    pub digital: String,
    pub hours: i32,
    pub minutes: i32,
    pub text: String,
    pub total_seconds: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Range {
    pub date: String,
    pub end: String,
    pub start: String,
    pub text: String,
    pub timezone: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SummariesResponse {
    pub data: Vec<Summary>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct WakatimeData {
    pub all_time: Option<AllTimeStats>,
    pub summaries: Option<Vec<Summary>>,
}

// ... existing AllTimeStats and related structs ...

impl WakatimeService {
    // ... existing new() ...

    // ... existing fetch_all_time_stats() ...

    pub async fn fetch_summaries(&self) -> Result<Vec<Summary>, Box<dyn Error + Send + Sync>> {
        let cache_file = "wakatime_summaries_cache.json";
        let cache_duration = Duration::from_secs(3600); // 1 hour

        if let Some(resp) = Self::load_from_cache::<SummariesResponse>(cache_file, cache_duration) {
             println!("Loaded WakaTime summaries from cache");
             return Ok(resp.data);
        }

        if self.api_key.is_empty() {
            return Err("WAKATIME_API_KEY not set".into());
        }

        // Fetch last 365 days
        let start = chrono::Utc::now() - chrono::Duration::days(365);
        let end = chrono::Utc::now();
        let start_str = start.format("%Y-%m-%d").to_string();
        let end_str = end.format("%Y-%m-%d").to_string();

        let url = format!("{}/users/current/summaries?start={}&end={}", self.base_url, start_str, end_str);
        
        let api_key_auth = BASE64_STANDARD.encode(format!("{}:", self.api_key));
        
        println!("Fetching WakaTime summaries from API...");
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Basic {}", api_key_auth))
            .send()
            .await?;

        if !resp.status().is_success() {
             return Err(format!("Failed to fetch wakatime summaries: {}", resp.status()).into());
        }

        let text = resp.text().await?;
        
        // Save to cache
        if let Err(e) = fs::write(cache_file, &text) {
             println!("Failed to write summaries cache: {}", e);
        }

        let response = serde_json::from_str::<SummariesResponse>(&text)?;
        Ok(response.data)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllTimeStats {
    pub data: AllTimeData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AllTimeData {
    pub total_seconds: f64,
    pub text: Option<String>,
    pub decimal: Option<String>,
    pub digital: Option<String>,
    pub is_up_to_date: bool,
    pub timeout: i32,
    pub total_seconds_including_other_language: f64,
    pub human_readable_total: String,
    pub human_readable_total_including_other_language: String,
    pub daily_average: f64,
    pub daily_average_including_other_language: f64,
    pub human_readable_daily_average: String,
    pub human_readable_daily_average_including_other_language: String,
    pub languages: Vec<StatItem>,
    pub operating_systems: Vec<StatItem>,
    pub editors: Vec<StatItem>,
    pub categories: Vec<StatItem>,
    // dependencies: Vec<StatItem>, 
    // machines: Vec<StatItem>,
    // projects: Vec<StatItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatItem {
    pub name: String,
    pub total_seconds: f64,
    pub percent: f64,
    pub digital: String,
    pub text: String,
    pub hours: i32,
    pub minutes: i32,
}

pub struct WakatimeService {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}

use std::fs;

use std::time::{SystemTime, Duration};

use serde::de::DeserializeOwned;

impl WakatimeService {
    pub fn new() -> Self {
        let api_key = env::var("WAKATIME_API_KEY").unwrap_or_default();
        let base_url = "https://wakatime.com/api/v1".to_string();
        
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
        }
    }

    fn load_from_cache<T: DeserializeOwned>(cache_file: &str, cache_duration: Duration) -> Option<T> {
        let metadata = fs::metadata(cache_file).ok()?;
        let modified = metadata.modified().ok()?;
        let elapsed = SystemTime::now().duration_since(modified).ok()?;

        if elapsed >= cache_duration {
            return None;
        }

        let contents = fs::read_to_string(cache_file).ok()?;
        serde_json::from_str(&contents).ok()
    }

    pub async fn fetch_all_time_stats(&self) -> Result<AllTimeStats, Box<dyn Error + Send + Sync>> {
        let cache_file = "wakatime_cache.json";
        let cache_duration = Duration::from_secs(3600); // 1 hour

        if let Some(stats) = Self::load_from_cache(cache_file, cache_duration) {
            println!("Loaded WakaTime stats from cache");
            return Ok(stats);
        }

        if self.api_key.is_empty() {
            return Err("WAKATIME_API_KEY not set".into());
        }

        let url = format!("{}/users/current/stats/all_time", self.base_url);
        
        let api_key_auth = BASE64_STANDARD.encode(format!("{}:", self.api_key));
        
        println!("Fetching WakaTime stats from API...");
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Basic {}", api_key_auth))
            .send()
            .await?;

        if !resp.status().is_success() {
             return Err(format!("Failed to fetch wakatime stats: {}", resp.status()).into());
        }

        let text = resp.text().await?;
        
        // Save to cache
        if let Err(e) = fs::write(cache_file, &text) {
             println!("Failed to write cache: {}", e);
        }

        let stats = serde_json::from_str(&text)?;
        Ok(stats)
    }
}
