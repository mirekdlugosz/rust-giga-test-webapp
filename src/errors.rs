use axum::http::StatusCode;
use std::num::TryFromIntError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("axum http error: {0}")]
    Axum(#[from] axum::http::Error),
    #[error("compression error: {0}")]
    Compression(String),
    #[error("entry not found")]
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
}

impl From<Error> for StatusCode {
    fn from(err: Error) -> Self {
        match err {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::IllegalCharacters
            | Error::IntConversion(_)
            | Error::WrongSize
            | Error::CookieParsing(_) => StatusCode::BAD_REQUEST,
            Error::Join(_)
            | Error::Compression(_)
            | Error::Axum(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
