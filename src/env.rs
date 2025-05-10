use std::net::{SocketAddr, AddrParseError};
use std::num::ParseIntError;
use std::time::Duration;

const GIGA_TEST_PORT: &str = "GIGA_TEST_PORT";
const GIGA_TEST_ADDRESS: &str = "GIGA_TEST_ADDRESS";
const GIGA_TEST_HTTP_TIMEOUT: &str = "GIGA_TEST_HTTP_TIMEOUT";
const GIGA_TEST_SQLITE_PATH: &str = "GIGA_TEST_SQLITE_PATH";

pub(crate) const DEFAULT_PORT: usize = 8088;
pub(crate) const DEFAULT_ADDRESS: &str = "127.0.0.1";
pub(crate) const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(5);
pub(crate) const DEFAULT_SQLITE_PATH: &str = ":memory:";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to parse {GIGA_TEST_PORT}: {0}")]
    InvalidPort(ParseIntError),
    #[error("failed to parse {GIGA_TEST_HTTP_TIMEOUT}: {0}")]
    HttpTimeout(ParseIntError),
    #[error("failed to parse socket address: {0}")]
    InvalidAddress(AddrParseError),
}

pub(crate) fn bind_addr() -> Result<SocketAddr, Error> {
    let address = std::env::var(GIGA_TEST_ADDRESS)
        .or(Ok(DEFAULT_ADDRESS.to_string()))?;
    let port = std::env::var(GIGA_TEST_PORT)
        .map_or_else(
            |_| Ok(DEFAULT_PORT),
            |p| p.parse::<usize>()
        )
        .map_err(Error::InvalidPort)?;
    format!("{address}:{port}").parse().map_err(Error::InvalidAddress)
}

pub(crate) fn http_timeout() -> Result<Duration, Error> {
    std::env::var(GIGA_TEST_HTTP_TIMEOUT)
        .map_or_else(
            |_| Ok(DEFAULT_HTTP_TIMEOUT),
            |s| s.parse::<u64>().map(|v| Duration::new(v, 0)),
        )
        .map_err(Error::HttpTimeout)
}

pub(crate) fn sqlite_pool() -> Result<String, Error> {
    std::env::var(GIGA_TEST_SQLITE_PATH)
        .or(Ok(DEFAULT_SQLITE_PATH.to_string()))
        .map(|s| format!("sqlite:{s}").to_string())
}
