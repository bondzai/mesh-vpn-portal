use crate::domain::{LogEntry, LogMetadata, LogStats, LogQuery};
use std::error::Error;

pub trait LogRepository: Send + Sync {
    fn append(&self, entry: &LogEntry) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn find_all(&self, query: &LogQuery) -> (Vec<LogEntry>, LogMetadata, LogStats);
    fn clear(&self) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn get_raw_content(&self) -> Result<String, Box<dyn Error + Send + Sync>>;
}
