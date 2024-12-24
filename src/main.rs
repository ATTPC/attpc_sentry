mod models;
mod schema;
mod sentry;
mod server;

use axum::{
    routing::{get, post},
    Router,
};
use deadpool_diesel::sqlite::{Manager, Pool};
use dotenvy::dotenv;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::sync::mpsc::{channel, Sender};

#[tokio::main]
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

    let db_url = match dotenv() {
        Ok(path) => {
            tracing::info!("Loaded env file from {path:?}");
            match env::var("DATABASE_URL") {
                Ok(value) => value,
                Err(e) => {
                    tracing::error!("env file did not contain variable DATABASE_URL: {e}");
                    return;
                }
            }
        }
        Err(e) => {
            tracing::error!("Failed to load .env file: {e}");
            return;
        }
    };

    let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let conn_pool = match Pool::builder(manager).build() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to create database connection pool: {e}");
            return;
        }
    };

    let (sentry_tx, sentry_rx) = channel(10);
    let pool_copy = conn_pool.clone();
    tokio::task::spawn(async move {
        sentry::watch_directory(&PathBuf::from("/Volumes"), sentry_rx, pool_copy).await
    });

    let app_state = server::AppState {
        conn_pool,
        sentry_tx: sentry_tx.clone(),
    };

    let router = Router::new()
        .route("/status", get(server::get_status))
        .route("/switch", post(server::set_directory))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on 0.0.0.0:8080");
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(lstnr) => lstnr,
        Err(e) => {
            tracing::error!("Failed to make listener: {e}");
            return;
        }
    };

    match axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(sentry_tx))
        .await
    {
        Ok(()) => (),
        Err(e) => {
            tracing::error!("An error occured: {e}");
            return;
        }
    }
}

async fn shutdown_signal(sentry_tx: Sender<sentry::Message>) {
    let ctrl_c_sig = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to handle ctrl-c")
    };

    #[cfg(unix)]
    let terminate_sig = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Could not hook to terminate signal.")
            .recv()
            .await
    };

    // No equivalent in windows?
    #[cfg(not(unix))]
    let terminate_sig = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c_sig => {
            sentry_tx.send(sentry::Message::Cancel).await.expect("Couldn't send cancel to sentry");
        }
        _ = terminate_sig => {
            sentry_tx.send(sentry::Message::Cancel).await.expect("Couldn't send cancel to sentry");
        }
    }
}
