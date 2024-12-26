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

    match dotenv() {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to load .env file: {e}");
            return;
        }
    }

    let db_url =
        env::var("DATABASE_URL").expect(".env file does not contain a DATABASE_URL variable!");
    let default_path =
        env::var("DEFAULT_PATH").expect(".env file does not contain a DEFAULT_PATH variable!");
    let default_disk =
        env::var("DEFAULT_DISK").expect(".env file does not contain a DEFAULT_DISK variable!");

    tracing::info!("Initializing the database connection...");
    let manager = Manager::new(db_url, deadpool_diesel::Runtime::Tokio1);
    let conn_pool = match Pool::builder(manager).build() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to create database connection pool: {e}");
            return;
        }
    };

    // Setting up some junk with the database
    tracing::info!("Initializing the database value...");
    match sentry::initialize_database_value(&conn_pool).await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to initialize database value: {e}");
            return;
        }
    };

    tracing::info!("Starting the sentry...");
    let (sentry_tx, sentry_rx) = channel(10);
    let pool_copy = conn_pool.clone();
    let handle = tokio::task::spawn(async move {
        sentry::watch_directory(
            &PathBuf::from(default_path),
            &default_disk,
            sentry_rx,
            pool_copy,
        )
        .await
    });

    tracing::info!("Starting the sentry server...");
    let app_state = server::AppState {
        conn_pool,
        sentry_tx: sentry_tx.clone(),
    };

    let router = Router::new()
        .route("/status", get(server::get_status))
        .route("/switch", post(server::set_directory))
        .with_state(app_state);

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
        .with_graceful_shutdown(shutdown_signal(sentry_tx))
        .await
    {
        Ok(()) => (),
        Err(e) => {
            tracing::error!("An error occured: {e}");
            return;
        }
    }

    tracing::info!("Joining watcher task...");
    match handle.await {
        Ok(res) => match res {
            Ok(_) => tracing::info!("Watcher gracefully shutdown."),
            Err(e) => tracing::error!("Watcher failed with error: {e}"),
        },
        Err(e) => tracing::error!("Watcher task failed to join: {e}"),
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
            tracing::info!("Shuting down due to ctrl-c...");
            sentry_tx.send(sentry::Message::Cancel).await.expect("Couldn't send cancel to sentry");
        }
        _ = terminate_sig => {
            tracing::info!("Shuting down due to terminate...");
            sentry_tx.send(sentry::Message::Cancel).await.expect("Couldn't send cancel to sentry");
        }
    }
}
