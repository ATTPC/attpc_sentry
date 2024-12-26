use super::models::MachineStatus;
use super::schema::status::dsl::*;

use super::sentry::Message;
use axum::{extract::State, http::StatusCode, Json};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppState {
    pub conn_pool: deadpool_diesel::sqlite::Pool,
    pub sentry_tx: tokio::sync::mpsc::Sender<Message>,
}

#[derive(Serialize, Deserialize)]
pub struct DirectoryChange {
    disk: String,
    directory: String,
}

pub async fn get_status(
    State(app): State<AppState>,
) -> Result<Json<MachineStatus>, (StatusCode, String)> {
    let conn = match app.conn_pool.get().await {
        Ok(c) => c,
        Err(e) => return Err(internal_error(e)),
    };
    let stat = conn
        .interact(|conn| {
            status
                .filter(id.eq(1))
                .select(MachineStatus::as_select())
                .load(conn)
        })
        .await
        .map_err(internal_error)?
        .map_err(internal_error)?
        .pop();

    if let Some(s) = stat {
        Ok(Json(s))
    } else {
        Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            String::from("No status exists!"),
        ))
    }
}

pub async fn set_directory(
    State(app): State<AppState>,
    Json(directory): Json<DirectoryChange>,
) -> Result<(), (StatusCode, String)> {
    app.sentry_tx
        .send(Message::WatchNew(
            PathBuf::from(directory.directory),
            directory.disk,
        ))
        .await
        .map_err(internal_error)?;
    Ok(())
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
