use crate::models::{
    PlaceBucket, TestStateMainPageElem, TestStateMainPageTotals, TestStatePartPage,
};
use askama::Template;

/// Index page displaying a form for paste insertion and a selection box for languages.
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

#[derive(Template)]
#[template(path = "about.html")]
pub struct About {}

impl About {
    pub fn new() -> Self {
        Self {}
    }
}
