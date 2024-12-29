mod data;
mod sentry;
mod server;

use axum::{
    routing::{get, post},
    Router,
};
use data::SentryState;
use std::net::SocketAddr;
use std::path::PathBuf;

#[tokio::main(flavor = "multi_thread", worker_threads = 3)]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Could not initialize the tracing system!");

    match dotenvy::dotenv() {
        Ok(path) => tracing::info!("Loaded environment from {path:?}"),
        Err(e) => {
            tracing::error!("Failed to load environment: {e}");
            return;
        }
    }

    let state = SentryState {
        disk_name: std::env::var("DISK_NAME").expect("DISK_NAME was not loaded from .env file!"),
        data_path: PathBuf::from(
            std::env::var("DATA_PATH").expect("DATA_PATH was not loaded from .env file!"),
        ),
        config_path: PathBuf::from(
            std::env::var("CONFIG_PATH").expect("CONFIG_PATH was not loaded from .env file!"),
        ),
        config_bck_path: PathBuf::from(
            std::env::var("CONFIG_BACKUP_PATH")
                .expect("CONFIG_BACKUP_PATH was not loaded from .env file!"),
        ),
        process_name: std::env::var("PROCESS_NAME")
            .expect("PROCESS_NAME was not loaded from .env file!"),
    };

    tracing::info!("Starting the sentry server...");
    let router = Router::new()
        .route("/status", get(server::status))
        .route("/catalog", post(server::catalog))
        .route("/backup", post(server::backup))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("Listening on 0.0.0.0:8080");
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(lstnr) => lstnr,
        Err(e) => {
            tracing::error!("Failed to make listener: {e}");
            return;
        }
    };

    match axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        Ok(()) => (),
        Err(e) => {
            tracing::error!("An error occured: {e}");
            return;
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c_sig = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to handle ctrl-c")
    };

    let terminate_sig = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Could not hook to terminate signal.")
            .recv()
            .await
    };

    tokio::select! {
        _ = ctrl_c_sig => {
            tracing::info!("Shuting down due to ctrl-c...");
        }
        _ = terminate_sig => {
            tracing::info!("Shuting down due to terminate...");
        }
    }
}
