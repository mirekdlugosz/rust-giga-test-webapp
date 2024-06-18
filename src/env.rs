use std::collections::HashMap;
use std::net::SocketAddr;
use std::num::ParseIntError;
use std::sync::OnceLock;
use std::time::Duration;

use crate::routes::MetadataRoutes;
use crate::models::{Test, RawTest, Question};

pub struct Metadata<'a> {
    pub title: String,
    pub version: &'a str,
    pub routes: MetadataRoutes,
}

pub const DEFAULT_HTTP_TIMEOUT: Duration = Duration::from_secs(5);

pub const CSS_MAX_AGE: Duration = Duration::from_secs(60 * 60 * 24 * 30 * 6);

pub const FAVICON_MAX_AGE: Duration = Duration::from_secs(86400);

const VAR_ADDRESS_PORT: &str = "GIGA_TEST_ADDRESS_PORT";
const VAR_MAX_BODY_SIZE: &str = "GIGA_TEST_MAX_BODY_SIZE";
const VAR_HTTP_TIMEOUT: &str = "GIGA_TEST_HTTP_TIMEOUT";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to parse {VAR_MAX_BODY_SIZE}, expected number of bytes: {0}")]
    MaxBodySize(ParseIntError),
    #[error("failed to parse {VAR_ADDRESS_PORT}, expected `host:port`")]
    AddressPort,
    #[error("failed to parse {VAR_HTTP_TIMEOUT}: {0}")]
    HttpTimeout(ParseIntError),
}

pub struct BasePath(String);

impl BasePath {
    pub fn path(&self) -> &str {
        &self.0
    }

    pub fn join(&self, s: &str) -> String {
        let b = &self.0;
        format!("{b}{s}")
    }
}

impl Default for BasePath {
    fn default() -> Self {
        BasePath("/".to_string())
    }
}

/// Retrieve reference to initialized metadata.
pub fn metadata() -> &'static Metadata<'static> {
    static DATA: OnceLock<Metadata> = OnceLock::new();

    DATA.get_or_init(|| {
        let title = std::env::var("GIGA_TEST_TITLE").unwrap_or_else(|_| "PSX Extreme Giga Test".to_string());
        let version = env!("CARGO_PKG_VERSION");
        let routes = MetadataRoutes::default();

        Metadata {
            title,
            version,
            routes,
        }
    })
}

pub fn addr() -> Result<SocketAddr, Error> {
    std::env::var(VAR_ADDRESS_PORT)
        .as_ref()
        .map(String::as_str)
        .unwrap_or("0.0.0.0:8088")
        .parse()
        .map_err(|_| Error::AddressPort)
}

pub fn max_body_size() -> Result<usize, Error> {
    std::env::var(VAR_MAX_BODY_SIZE)
        .map_or_else(|_| Ok(1024 * 1024), |s| s.parse::<usize>())
        .map_err(Error::MaxBodySize)
}

pub fn base_path() -> &'static BasePath {
    static BASE_PATH: OnceLock<BasePath> = OnceLock::new();

    BASE_PATH.get_or_init(|| BasePath::default())
}

pub fn http_timeout() -> Result<Duration, Error> {
    std::env::var(VAR_HTTP_TIMEOUT)
        .map_or_else(
            |_| Ok(DEFAULT_HTTP_TIMEOUT),
            |s| s.parse::<u64>().map(|v| Duration::new(v, 0)),
        )
        .map_err(Error::HttpTimeout)
}

pub fn giga_test() -> &'static Test {
    static GIGA_TEST: OnceLock<Test> = OnceLock::new();

    GIGA_TEST.get_or_init(|| {
        let giga_test_toml = include_str!("../resources/gigatest.toml");
        toml::from_str::<RawTest>(giga_test_toml)
            .unwrap_or_default()
            .into()
    })
}

pub fn giga_test_questions() -> &'static HashMap<String, &'static Question> {
    static QUESTIONS_DB: OnceLock<HashMap<String, &Question>> = OnceLock::new();

    QUESTIONS_DB.get_or_init(|| giga_test().get_questions())
}
