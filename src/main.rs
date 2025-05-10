use crate::errors::Error;
use crate::giga_test::get_giga_test;
use axum::Router;
use std::process::ExitCode;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::SessionManagerLayer;
use tower_sessions_sqlx_store::{sqlx::SqlitePool, SqliteStore};
use tower_serve_static::ServeDir;
use include_dir::{Dir, include_dir};
use regex::Regex;
use dotenvy;
use std::io::ErrorKind;
use std::fs::File;

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

fn ensure_sqlite_file_exists(pool: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filepath = pool.trim_start_matches("sqlite:");
    tracing::info!("Ensuring SQLite file exists: {filepath}");
    match File::create_new(filepath) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
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
    let _ = dotenvy::dotenv();

    // FIXME: variable to specify base path, passed to templates. that would allow to pretend app
    //      is running from subdirectory
    let bind_addr = env::bind_addr()?;
    let timeout = env::http_timeout()?;
    let sqlite_pool = env::sqlite_pool()?;
    if ! (sqlite_pool.to_lowercase() == "sqlite::memory:") {
        ensure_sqlite_file_exists(&sqlite_pool)?;
    }
    let giga_test = get_giga_test(&html_preprocessor);
    let questions_db = &giga_test.get_questions().to_owned();

    let state = AppState {
        giga_test: giga_test.to_owned(),
        questions_db: questions_db.clone(),
    };

    let pool = SqlitePool::connect(&sqlite_pool).await?;
    let session_store = SqliteStore::new(pool).with_table_name("sessions")?;
    session_store.migrate().await?;

    tracing::info!("serving on {bind_addr}");
    tracing::info!("timeout set to {timeout:?}");
    tracing::info!("using SQLite db at {sqlite_pool}");

    let service = Router::new()
        .nest_service("/static", ServeDir::new(&STATIC_ASSETS_DIR))
        .merge(routes::routes())
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(timeout))
                .layer(SessionManagerLayer::new(session_store)
                       .with_name("giga_test_session")),
        )
        .with_state(state);

    let listener = TcpListener::bind(&bind_addr).await?;

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
