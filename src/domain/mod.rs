use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    #[serde(rename = "activeUsers")]
    pub active_users: u32,
    #[serde(rename = "totalUsers")]
    pub total_users: u32,
}
