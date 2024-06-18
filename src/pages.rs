use crate::env;
use crate::models::{Test, TestStateMainPageElem, TestPart};
//use crate::routes::paste::{Format, QueryData};
use askama::Template;
use axum::http::StatusCode;

/// Error page showing a message.
#[derive(Template)]
#[template(path = "error.html")]
pub struct Error<'a> {
    meta: &'a env::Metadata<'a>,
    base_path: &'static env::BasePath,
    description: String,
}

/// Error response carrying a status code and the page itself.
pub type ErrorResponse<'a> = (StatusCode, Error<'a>);

impl From<crate::Error> for ErrorResponse<'_> {
    fn from(err: crate::Error) -> Self {
        let html = Error {
            meta: env::metadata(),
            base_path: env::base_path(),
            description: err.to_string(),
        };

        (err.into(), html)
    }
}

/// Index page displaying a form for paste insertion and a selection box for languages.
#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    meta: &'a env::Metadata<'a>,
    base_path: &'static env::BasePath,
    tests_state: &'a [TestStateMainPageElem],
    giga_test_finished: bool,
}

impl<'a> Index<'a> {
    pub fn new(
        tests_state: &'a [TestStateMainPageElem],
        giga_test_finished: bool,
    ) -> Self {
        Self {
            meta: env::metadata(),
            base_path: env::base_path(),
            tests_state,
            giga_test_finished,
        }
    }
}

/// Paste view showing the formatted paste as well as a bunch of links.
#[derive(Template)]
#[template(path = "part.html")]
pub struct Part<'a> {
    meta: &'a env::Metadata<'a>,
    base_path: &'static env::BasePath,
    part_state: &'a TestPart,
}

impl<'a> Part<'a> {
    /// Construct new paste view from cache `key` and paste `html`.
    pub fn new(part_state: &'a TestPart) -> Self {

        Self {
            meta: env::metadata(),
            base_path: env::base_path(),
            part_state,
        }
    }
}
