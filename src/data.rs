use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SentryState {
    pub data_path: PathBuf,
    pub config_path: PathBuf,
    pub config_bck_path: PathBuf,
    pub process_name: String,
    pub disk_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryParameters {
    pub experiment: String,
    pub run_number: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryResponse {
    pub disk: String,
    pub process: String,
    pub data_path: String,
    pub data_path_files: i32,
    pub data_written_gb: f64,
    pub disk_avail_gb: f64,
    pub disk_total_gb: f64,
}
