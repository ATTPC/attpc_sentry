mod data;
mod sentry;
mod server;

use axum::{routing::post, Router};
use std::net::SocketAddr;

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

    tracing::info!("Starting the sentry server...");
    let router = Router::new()
        .route("/status", post(server::status))
        .route("/catalog", post(server::catalog))
        .route("/backup", post(server::backup));

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
