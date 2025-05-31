use crate::models::{
    PlaceBucket, TestStateMainPageElem, TestStateMainPageTotals, TestStatePartPage,
};
use askama::Template;

#[derive(Debug, Template)]
#[template(path = "error.html")]
pub struct ErrorPage {
    description: String,
}

impl ErrorPage {
    pub fn new(description: String) -> Self {
        Self { description }
    }
}

/// Index page - list of parts
#[derive(Template)]
#[template(path = "index.html")]
pub struct Index<'a> {
    tests_state: &'a [TestStateMainPageElem],
    totals: &'a TestStateMainPageTotals,
    count_canceled: bool,
    giga_test_finished: bool,
}

impl<'a> Index<'a> {
    pub fn new(
        tests_state: &'a [TestStateMainPageElem],
        totals: &'a TestStateMainPageTotals,
        count_canceled: bool,
        giga_test_finished: bool,
    ) -> Self {
        Self {
            tests_state,
            totals,
            count_canceled,
            giga_test_finished,
        }
    }
}

/// Part page - introduction and questions
#[derive(Template)]
#[template(path = "part.html")]
pub struct Part<'a> {
    part_state: &'a TestStatePartPage,
    giga_test_finished: bool,
}

impl<'a> Part<'a> {
    pub fn new(part_state: &'a TestStatePartPage, giga_test_finished: bool) -> Self {
        Self {
            part_state,
            giga_test_finished,
        }
    }
}

/// About page - static text
#[derive(Template)]
#[template(path = "about.html")]
pub struct About {}

impl About {
    pub fn new() -> Self {
        Self {}
    }
}
