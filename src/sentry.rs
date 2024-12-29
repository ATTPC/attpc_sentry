use super::data::{SentryParameters, SentryResponse, SentryState};
use std::path::PathBuf;
use sysinfo::{Disks, ProcessRefreshKind, RefreshKind, System};
use tokio::fs::read_dir;

const COBO_DESC: &str = "describe-cobo";

#[derive(Debug)]
pub enum SentryError {
    NotDirectory(PathBuf),
    NoProcess(String),
    CatAlreadyExists(PathBuf, i32),
    BckAlreadyExists(PathBuf, i32),
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
            Self::BckAlreadyExists(path, run) => write!(
                f,
                "Sentry tried to backup config files for {run} but directory {path:?} already exists"
            ),
            Self::NoProcess(name) => write!(f, "Sentry could not find a process with name {name}")
        }
    }
}

impl std::error::Error for SentryError {}

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

    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    let proc = match sys.processes_by_name(state.process_name.as_ref()).next() {
        Some(p) => p,
        None => return Err(SentryError::NoProcess(state.process_name.clone())),
    };

    if !state.data_path.is_dir() {
        return Err(SentryError::NotDirectory(state.data_path.clone()));
    }

    let mut n_files = 0;
    let mut reader = read_dir(&state.data_path).await?;
    loop {
        match reader.next_entry().await? {
            Some(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                n_files += 1;
            }
            None => break,
        }
    }

    Ok(SentryResponse {
        disk: state.disk_name.clone(),
        process: state.process_name.clone(),
        data_path: String::from(state.data_path.to_string_lossy()),
        data_written_gb: (proc.disk_usage().written_bytes as f64) * 1.0e-9,
        disk_avail_gb: (avail_total as f64) * 1.0e-9,
        disk_total_gb: (disk_total as f64) * 1.0e-9,
        data_path_files: n_files,
    })
}

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
    loop {
        match reader.next_entry().await? {
            Some(entry) => {
                let path = entry.path();
                match path.extension() {
                    Some(ext) => {
                        if path.is_file() && ext == "graw" {
                            let new_path = cat_dir
                                .join(path.file_name().expect("File doesn't have file name?!"));
                            tokio::fs::rename(path, new_path).await?;
                        }
                    }
                    None => continue,
                }
            }
            None => break,
        }
    }

    check_status(state).await
}

pub async fn backup_configs(
    state: &SentryState,
    params: SentryParameters,
) -> Result<SentryResponse, SentryError> {
    let config_cobo_dir = state.config_path.join(COBO_DESC);
    let bck_config_dir = state.config_bck_path.join(format!(
        "{}/run_{:04}",
        params.experiment, params.run_number
    ));
    let bck_cobo_dir = bck_config_dir.join(COBO_DESC);

    if bck_config_dir.exists() {
        return Err(SentryError::BckAlreadyExists(
            bck_config_dir,
            params.run_number,
        ));
    }
    tokio::fs::create_dir_all(&bck_cobo_dir).await?;

    let prep_name = format!("prepare-{}.xcfg", params.experiment);
    let desc_name = format!("describe-{}.xcfg", params.experiment);
    let conf_name = format!("configure-{}.xcfg", params.experiment);

    tokio::fs::copy(
        state.config_path.join(&prep_name),
        bck_config_dir.join(&prep_name),
    )
    .await?;
    tokio::fs::copy(
        state.config_path.join(&desc_name),
        bck_config_dir.join(&desc_name),
    )
    .await?;
    tokio::fs::copy(
        state.config_path.join(&conf_name),
        bck_config_dir.join(&conf_name),
    )
    .await?;

    let mut reader = read_dir(config_cobo_dir).await?;
    loop {
        match reader.next_entry().await? {
            Some(entry) => {
                let path = entry.path();
                if path.is_file() {
                    tokio::fs::copy(
                        &path,
                        bck_cobo_dir.join(path.file_name().expect("File doesn't have file name?")),
                    )
                    .await?;
                }
            }
            None => break,
        }
    }

    Ok(SentryResponse {
        disk: state.disk_name.clone(),
        process: state.process_name.clone(),
        data_path: String::from(state.data_path.to_string_lossy()),
        data_written_gb: 0.0,
        disk_avail_gb: 0.0,
        disk_total_gb: 0.0,
        data_path_files: 0,
    })
}
