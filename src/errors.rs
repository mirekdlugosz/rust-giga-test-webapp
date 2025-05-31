use std::num::TryFromIntError;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("axum http error: {0}")]
    Axum(#[from] axum::http::Error),
    #[error("compression error: {0}")]
    Compression(String),
    #[error("not found")]
    NotFound,
    #[error("wrong size")]
    WrongSize,
    #[error("illegal characters")]
    IllegalCharacters,
    #[error("integer conversion error: {0}")]
    IntConversion(#[from] TryFromIntError),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("could not parse cookie: {0}")]
    CookieParsing(String),
    #[error("could not render template")]
    Render(#[from] askama::Error),
}
