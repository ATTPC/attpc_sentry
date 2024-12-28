use super::data::{SentryConfig, SentryResponse};
use super::sentry::catalogue_run;
use super::sentry::check_status;
use axum::{http::StatusCode, Json};

pub async fn status(
    Json(config): Json<SentryConfig>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = check_status(config).await.map_err(internal_error)?;
    Ok(Json(response))
}

pub async fn catalogue(
    Json(config): Json<SentryConfig>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = catalogue_run(config).await.map_err(internal_error)?;
    Ok(Json(response))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
