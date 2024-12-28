use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryConfig {
    pub disk: String,
    pub path: String,
    pub experiment: String,
    pub run_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryResponse {
    pub disk: String,
    pub path: String,
    pub path_gb: f64,
    pub path_n_files: i32,
    pub disk_avail_gb: f64,
    pub disk_total_gb: f64,
}
