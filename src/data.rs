//! Module of all of the data structures used in the application

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// This is the state of the sentry app, containing all the paths and names that the
/// server needs to run
#[derive(Debug, Clone)]
pub struct SentryState {
    pub data_path: PathBuf,
    pub process_name: String,
    pub disk_name: String,
}

/// These are external parameters that the sentry needs for some specific operations
/// like moving or backing up files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryParameters {
    pub experiment: String,
    pub run_number: i32,
}

/// This is the data returned by the sentry after running an operation
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
