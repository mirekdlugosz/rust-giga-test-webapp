use crate::errors::Error;
use crate::giga_test::get_giga_test;
use axum::Router;
use std::process::ExitCode;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tower_serve_static::ServeDir;
use include_dir::{Dir, include_dir};
use regex::Regex;

mod env;
mod errors;
mod giga_test;
mod models;
mod pages;
mod routes;

static STATIC_ASSETS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/resources");

#[derive(Clone)]
pub struct AppState {
    giga_test: models::Test,
    questions_db: models::QuestionsDB,
}

pub(crate) fn make_app(timeout: Duration) -> Router<AppState> {
    // FIXME: headery dla cache zasobów statycznych, może obsługa nagłówka if-modified
    // FIXME: we might want persistent file-based session storage with memory storage in front, as part of
    //        cached storage
    // FIXME: check how it works when multiple people try to access a page
    Router::new()
        .nest_service("/static", ServeDir::new(&STATIC_ASSETS_DIR))
        .merge(routes::routes())
        .layer(
            ServiceBuilder::new()
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

// FIXME: review all unwrap() calls
fn html_preprocessor(input: &str) -> String {
    let re = Regex::new(r"\[img\](\S+\.png)\[/img\]").unwrap();
    let new = re.replace_all(input, "<img src='/static/img/$1'>");
    new.to_string()
}

async fn start() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // FIXME: port env to dotenv or similar
    // FIXME: protocol, domain, port, path - everything
    let addr = env::addr()?;
    let timeout = env::http_timeout()?;
    // FIXME: should preprocessor know about path?
    let giga_test = get_giga_test(&html_preprocessor);
    let questions_db = &giga_test.get_questions().to_owned();

    let state = AppState {
        giga_test: giga_test.to_owned(),
        questions_db: questions_db.clone(),
    };

    tracing::info!("serving on {addr}");
    tracing::info!("timeout set to {:?}", timeout);

    let service = make_app(timeout).with_state(state);
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
