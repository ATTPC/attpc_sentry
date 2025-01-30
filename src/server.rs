/// These are thin wrappers directing the axum routes to the appropriate sentry
/// actions
use super::data::{SentryParameters, SentryResponse, SentryState};
use super::sentry::{catalog_run, check_status};
use axum::{extract::State, http::StatusCode, Json};

/// Direct the /status route to the check_status function
pub async fn status(
    State(state): State<SentryState>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = check_status(&state).await.map_err(internal_error)?;
    Ok(Json(response))
}

/// Direct the /catalog route to the catalog_run function
pub async fn catalog(
    State(state): State<SentryState>,
    Json(config): Json<SentryParameters>,
) -> Result<Json<SentryResponse>, (StatusCode, String)> {
    let response = catalog_run(&state, config).await.map_err(internal_error)?;
    Ok(Json(response))
}

/// Wrap a sentry error into something that axum can report
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
