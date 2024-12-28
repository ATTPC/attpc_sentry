use super::data::{SentryConfig, SentryResponse};
use std::path::PathBuf;
use sysinfo::Disks;
use tokio::fs::read_dir;

const CONFIG_PATH: &str = "/Users/attpc/configs";
const BACKUP_CONFIG_PATH: &str = "/Users/attpc/configs_backup";
const COBO_DESC: &str = "describe-cobo";

#[derive(Debug)]
pub enum SentryError {
    NotDirectory(PathBuf),
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
                "Sentry tried to catalogue run {run} but directory {path:?} already exists"
            ),
            Self::BckAlreadyExists(path, run) => write!(
                f,
                "Sentry tried to backup config files for {run} but directory {path:?} already exists"
            ),
        }
    }
}

impl std::error::Error for SentryError {}

pub async fn check_status(config: SentryConfig) -> Result<SentryResponse, SentryError> {
    let disks = Disks::new_with_refreshed_list();
    let directory = PathBuf::from(&config.path);
    let mut disk_total = 0;
    let mut avail_total = 0;
    for disk in disks.iter() {
        if *disk.name() == *config.disk {
            disk_total += disk.total_space();
            avail_total += disk.available_space();
            break;
        }
    }

    if !directory.is_dir() {
        return Err(SentryError::NotDirectory(directory.to_path_buf()));
    }

    let mut n_files = 0;
    let mut used_size = 0;
    let mut reader = read_dir(directory).await?;
    loop {
        match reader.next_entry().await? {
            Some(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                n_files += 1;
                used_size += tokio::fs::metadata(path).await?.len();
            }
            None => break,
        }
    }

    Ok(SentryResponse {
        disk: config.disk.clone(),
        path: config.path.clone(),
        path_gb: (used_size as f64) * 1.0e-9,
        disk_avail_gb: (avail_total as f64) * 1.0e-9,
        disk_total_gb: (disk_total as f64) * 1.0e-9,
        path_n_files: n_files,
    })
}

pub async fn catalogue_run(config: SentryConfig) -> Result<SentryResponse, SentryError> {
    let daq_dir = PathBuf::from(&config.path);
    let cat_dir = daq_dir.join(format!(
        "{}/run_{:04}",
        config.experiment, config.run_number
    ));

    if cat_dir.exists() {
        return Err(SentryError::CatAlreadyExists(cat_dir, config.run_number));
    }
    tokio::fs::create_dir_all(&cat_dir).await?;

    let mut reader = read_dir(daq_dir).await?;
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

    check_status(config).await
}

pub async fn backup_configs(config: SentryConfig) -> Result<SentryResponse, SentryError> {
    let config_dir = PathBuf::from(CONFIG_PATH);
    let config_cobo_dir = config_dir.join(COBO_DESC);
    let bck_config_dir = PathBuf::from(BACKUP_CONFIG_PATH).join(format!(
        "{}/run_{:04}",
        config.experiment, config.run_number
    ));
    let bck_cobo_dir = bck_config_dir.join(COBO_DESC);

    if bck_config_dir.exists() {
        return Err(SentryError::BckAlreadyExists(
            bck_config_dir,
            config.run_number,
        ));
    }
    tokio::fs::create_dir_all(&bck_cobo_dir).await?;

    let prep_name = format!("prepare-{}.xcfg", config.experiment);
    let desc_name = format!("describe-{}.xcfg", config.experiment);
    let conf_name = format!("configure-{}.xcfg", config.experiment);

    tokio::fs::copy(config_dir.join(&prep_name), bck_config_dir.join(&prep_name)).await?;
    tokio::fs::copy(config_dir.join(&desc_name), bck_config_dir.join(&desc_name)).await?;
    tokio::fs::copy(config_dir.join(&conf_name), bck_config_dir.join(&conf_name)).await?;

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
        disk: String::from("Macintosh HD"),
        path: String::from(bck_config_dir.to_string_lossy()),
        path_gb: 0.0,
        path_n_files: 3,
        disk_avail_gb: 0.0,
        disk_total_gb: 0.0,
    })
}
