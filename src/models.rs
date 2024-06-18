use std::collections::HashMap;
use serde::Deserialize;

fn ret_false() -> bool {
    false
}

pub(crate) type Test = HashMap<String, TestPart>;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct TestPart {
    introduction: String,
    sections: HashMap<String, Section>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Section {
    introduction: String,
    questions: Vec<Question>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Question {
    question: String,
    choices: Vec<AnswerChoice>,
    #[serde(default = "ret_false")]
    canceled: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AnswerChoice {
    answer: String,
    correct: bool,
}

pub(crate) type TestPartsIterable = Vec<String>;


pub(crate) fn get_test_parts(test: &Test) -> TestPartsIterable {
    test.keys().map(|s| s.to_owned()).collect()
}
