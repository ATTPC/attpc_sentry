use super::data::{SentryParameters, SentryResponse, SentryState};
use super::sentry::{backup_configs, catalog_run, check_status};
use axum::{extract::State, http::StatusCode, Json};

pub async fn status(
    State(state): State<SentryState>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = check_status(&state).await.map_err(internal_error)?;
    Ok(Json(response))
}

pub async fn catalog(
    State(state): State<SentryState>,
    Json(config): Json<SentryParameters>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = catalog_run(&state, config).await.map_err(internal_error)?;
    Ok(Json(response))
}

pub async fn backup(
    State(state): State<SentryState>,
    Json(config): Json<SentryParameters>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = backup_configs(&state, config)
        .await
        .map_err(internal_error)?;
    Ok(Json(response))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
