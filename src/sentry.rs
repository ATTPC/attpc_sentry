use super::models::DirectoryStatus;
use deadpool_diesel::sqlite::Pool;
use std::fs::read_dir;
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time;
use sysinfo::Disks;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Receiver;

const WAIT_TIME: u64 = 10;

#[derive(Debug)]
pub enum WatchError {
    ClosedChannel,
    SendError(SendError<DirectoryStatus>),
    NotDirectory(PathBuf),
    BadIO(std::io::Error),
    DatabasePoolFailed(deadpool_diesel::PoolError),
    DatabaseConnFailed(deadpool_diesel::InteractError),
}

impl From<SendError<DirectoryStatus>> for WatchError {
    fn from(val: SendError<DirectoryStatus>) -> Self {
        Self::SendError(val)
    }
}

impl From<std::io::Error> for WatchError {
    fn from(value: std::io::Error) -> Self {
        Self::BadIO(value)
    }
}

impl From<deadpool_diesel::PoolError> for WatchError {
    fn from(value: deadpool_diesel::PoolError) -> Self {
        Self::DatabasePoolFailed(value)
    }
}

impl From<deadpool_diesel::InteractError> for WatchError {
    fn from(value: deadpool_diesel::InteractError) -> Self {
        Self::DatabaseConnFailed(value)
    }
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClosedChannel => write!(f, "Watcher's communication channel was closed!"),
            Self::SendError(e) => write!(f, "Watcher failed to send a message with error: {e}"),
            Self::NotDirectory(path) => write!(
                f,
                "Watcher was given a non-directory path ({path:?}) to watch"
            ),
            Self::BadIO(e) => write!(
                f,
                "Watcher was not able to iterate through the directory with error: {e}"
            ),
            Self::DatabasePoolFailed(e) => {
                write!(f, "Watcher failed at the database connection pool: {e}")
            }
            Self::DatabaseConnFailed(e) => {
                write!(f, "Watcher failed at the database connection: {e}")
            }
        }
    }
}

impl std::error::Error for WatchError {}

pub enum Message {
    Cancel,
    WatchNew(PathBuf),
}

pub async fn watch_directory(
    directory: &Path,
    mut incoming: Receiver<Message>,
    conn_pool: Pool,
) -> Result<(), WatchError> {
    let mut current_dir = directory.to_path_buf();
    loop {
        tokio::select! {
            res = incoming.recv() => {
                if let Some(msg) = res {
                    match msg {
                        Message::Cancel => return Ok(()),
                        Message::WatchNew(path) => current_dir = path
                    }
                } else {
                    return Err(WatchError::ClosedChannel)
                }
            }

            _ = tokio::time::sleep(time::Duration::from_secs(WAIT_TIME)) => {
                    let status = check_directory(&current_dir)?;
                    write_to_database(&conn_pool, status).await?;
            }
        }
    }
}

fn check_directory(directory: &Path) -> Result<DirectoryStatus, WatchError> {
    let disks = Disks::new_with_refreshed_list();
    let mut disk_total = 0;
    let mut avail_total = 0;
    for disk in disks.iter() {
        disk_total += disk.total_space();
        avail_total += disk.available_space();
    }

    if !directory.is_dir() {
        return Err(WatchError::NotDirectory(directory.to_path_buf()));
    }

    let mut n_files = 0;
    let mut used_size = 0;
    for maybe_entry in read_dir(directory)? {
        if let Ok(entry) = maybe_entry {
            let path = entry.path();

            if path.is_dir() {
                continue;
            }

            n_files += 1;
            used_size += path.metadata()?.size();
        }
    }

    Ok(DirectoryStatus {
        id: 1,
        path: String::from(directory.to_string_lossy()),
        dir_bytes: (used_size as f64) * 1.0e-9,
        bytes_avail: (avail_total as f64) * 1.0e-9,
        total_bytes: (disk_total as f64) * 1.0e-9,
        n_files,
    })
}

async fn write_to_database(conn_pool: &Pool, stat: DirectoryStatus) -> Result<(), WatchError> {
    use super::schema::status::dsl::*;
    use diesel::prelude::*;
    let connection = conn_pool.get().await?;
    let _ = connection
        .interact(|conn| diesel::update(status).set(stat).execute(conn))
        .await?;
    Ok(())
}
