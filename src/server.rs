use super::data::{SentryConfig, SentryResponse};
use super::sentry::{backup_configs, catalog_run, check_status};
use axum::{http::StatusCode, Json};

pub async fn status(
    Json(config): Json<SentryConfig>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = check_status(config).await.map_err(internal_error)?;
    Ok(Json(response))
}

pub async fn catalog(
    Json(config): Json<SentryConfig>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = catalog_run(config).await.map_err(internal_error)?;
    Ok(Json(response))
}

pub async fn backup(
    Json(config): Json<SentryConfig>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = backup_configs(config).await.map_err(internal_error)?;
    Ok(Json(response))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
