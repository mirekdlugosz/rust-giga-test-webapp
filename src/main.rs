use crate::errors::Error;
use axum::extract::DefaultBodyLimit;
use axum::Router;
use std::process::ExitCode;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

//mod assets;
mod env;
mod errors;
mod models;
mod pages;
mod routes;
#[cfg(test)]
mod test_helpers;

#[derive(Clone)]
pub struct AppState {
    giga_test: models::Test,
}

pub(crate) fn make_app(max_body_size: usize, timeout: Duration) -> Router<AppState> {
    Router::new()
        .nest("/", routes::routes())
        .layer(
            ServiceBuilder::new()
                .layer(DefaultBodyLimit::disable())
                .layer(DefaultBodyLimit::max(max_body_size))
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(timeout))
                .layer(SessionManagerLayer::new(MemoryStore::default())
                       .with_name("giga_test_session")),
        )
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("received signal, exiting ...");
}

async fn start() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = env::addr()?;
    let max_body_size = env::max_body_size()?;
    let timeout = env::http_timeout()?;
    let giga_test = env::giga_test().to_owned();
    let state = AppState {
        giga_test,
    };

    tracing::info!("serving on {addr}");
    tracing::info!("restricting maximum body size to {max_body_size} bytes");
    tracing::info!("timeout set to {:?}", timeout);

    let service = make_app(max_body_size, timeout).with_state(state);
    let listener = TcpListener::bind(&addr).await?;

    axum::serve(listener, service)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> ExitCode {
    match start().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
            ExitCode::FAILURE
        }
    }
}
