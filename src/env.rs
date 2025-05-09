use std::net::SocketAddr;
use std::num::ParseIntError;
use std::time::Duration;

pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(5);

//pub const CSS_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 30 * 6);

//pub const FAVICON_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 30 * 6);

const VAR_ADDRESS_PORT: &str = "GIGA_TEST_ADDRESS_PORT";
const VAR_HTTP_TIMEOUT: &str = "GIGA_TEST_HTTP_TIMEOUT";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to parse {VAR_ADDRESS_PORT}, expected `host:port`")]
    AddressPort,
    #[error("failed to parse {VAR_HTTP_TIMEOUT}: {0}")]
    HttpTimeout(ParseIntError),
}

pub fn addr() -> Result<SocketAddr, Error> {
    std::env::var(VAR_ADDRESS_PORT)
        .as_ref()
        .map(String::as_str)
        .unwrap_or("0.0.0.0:8088")
        .parse()
        .map_err(|_| Error::AddressPort)
}

pub fn http_timeout() -> Result<Duration, Error> {
    std::env::var(VAR_HTTP_TIMEOUT)
        .map_or_else(
            |_| Ok(DEFAULT_HTTP_TIMEOUT),
            |s| s.parse::<u64>().map(|v| Duration::new(v, 0)),
        )
        .map_err(Error::HttpTimeout)
}
