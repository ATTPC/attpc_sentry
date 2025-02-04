//! Contains all of the functions to be run on the server
//! This is the actual "sentry" part
use super::data::{SentryParameters, SentryResponse, SentryState};
use std::path::PathBuf;
use std::process::Command;
use sysinfo::{Disks, Pid, ProcessRefreshKind, RefreshKind, System};
use tokio::fs::read_dir;

/// All the errors the sentry can run into
#[derive(Debug)]
pub enum SentryError {
    NotDirectory(PathBuf),
    NoProcess(String),
    CatAlreadyExists(PathBuf, i32),
    BadIO(std::io::Error),
}

impl From<std::io::Error> for SentryError {
    fn from(value: std::io::Error) -> Self {
        Self::BadIO(value)
    }
}

impl std::fmt::Display for SentryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotDirectory(path) => write!(
                f,
                "Sentry was given a non-directory path ({path:?}) to check the status of"
            ),
            Self::BadIO(e) => write!(f, "Sentry was not able to communicate with the disk: {e}"),
            Self::CatAlreadyExists(path, run) => write!(
                f,
                "Sentry tried to catalog run {run} but directory {path:?} already exists"
            ),
            Self::NoProcess(name) => write!(f, "Sentry could not find a process with name {name}"),
        }
    }
}

impl std::error::Error for SentryError {}

/// Check the status of the AT-TPC DAQ process and the workstation
/// Uses the [sysinfo](https://docs.rs/sysinfo/latest/sysinfo/) crate to examine the
/// status of the workstation disk and a DAQ process (typically dataRouter or friends)
/// and return a SentryResponse
pub async fn check_status(state: &SentryState) -> Result<SentryResponse, SentryError> {
    let disks = Disks::new_with_refreshed_list();
    let mut disk_total = 0;
    let mut avail_total = 0;
    for disk in disks.iter() {
        if *disk.name() == *state.disk_name {
            disk_total += disk.total_space();
            avail_total += disk.available_space();
            break;
        }
    }

    let pid = get_pid_old_macos(&state.process_name);
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    match sys.process(Pid::from(pid)) {
        Some(_) => (),
        None => return Err(SentryError::NoProcess(state.process_name.clone())),
    };

    if !state.data_path.is_dir() {
        return Err(SentryError::NotDirectory(state.data_path.clone()));
    }

    let mut n_files = 0;
    let mut reader = read_dir(&state.data_path).await?;
    let mut bytes_count = 0;
    while let Some(entry) = reader.next_entry().await? {
        let path = entry.path();
        if path.is_dir() || path.extension().is_none_or(|ex| ex != "graw") {
            continue;
        }
        bytes_count += path.metadata()?.len();
        n_files += 1;
    }

    Ok(SentryResponse {
        disk: state.disk_name.clone(),
        process: state.process_name.clone(),
        data_path: String::from(state.data_path.to_string_lossy()),
        data_written_gb: (bytes_count as f64) * 1.0e-9,
        disk_avail_gb: (avail_total as f64) * 1.0e-9,
        disk_total_gb: (disk_total as f64) * 1.0e-9,
        data_path_files: n_files,
    })
}

/// Move the DAQ runfiles (the acutal data) to a safe place
/// By default the AT-TPC DAQ does not create data runs. It just stores all of its data
/// in timestamped files at a single directory. This is not ideal as it can be difficult
/// to identify which file corresponds to which run. This function creates a run
/// directory and moves the .graw files to the run directory. It then returns a response
/// after checking the status.
pub async fn catalog_run(
    state: &SentryState,
    params: SentryParameters,
) -> Result<SentryResponse, SentryError> {
    let cat_dir = state.data_path.join(format!(
        "{}/run_{:04}",
        params.experiment, params.run_number
    ));

    if cat_dir.exists() {
        return Err(SentryError::CatAlreadyExists(cat_dir, params.run_number));
    }
    tokio::fs::create_dir_all(&cat_dir).await?;

    let mut reader = read_dir(&state.data_path).await?;
    while let Some(entry) = reader.next_entry().await? {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if path.is_file() && ext == "graw" {
                let new_path =
                    cat_dir.join(path.file_name().expect("File doesn't have file name?!"));
                tokio::fs::rename(path, new_path).await?;
            }
        }
    }

    check_status(state).await
}

/// This is a janky hack. For some reason, when starting processes using the
/// "AT-TPC method" (launchctl + shell script) the processes lose a lot of information
/// to sysinfo. This hack uses ps and essentially grep to find the process id.
fn get_pid_old_macos(process_name: &str) -> usize {
    let procs = Command::new("ps")
        .arg("-e")
        .output()
        .expect("We don't have the ps command?");

    let output = String::from_utf8(procs.stdout).expect("Output isn't utf8?");
    for line in output.lines() {
        let entries: Vec<&str> = line.split_whitespace().collect();
        if entries[3] == process_name {
            return entries[0].parse().expect("PID was not a number?");
        }
    }
    0
}
