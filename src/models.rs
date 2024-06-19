use std::collections::{BTreeMap, HashMap};
use serde::{Deserialize, Serialize};

fn ret_false() -> bool {
    false
}


pub(crate) type UserResponseData = HashMap<String, UserResponse>;
pub(crate) type QuestionsDB = HashMap<String, &'static Question>;

#[derive(Clone, Debug, Deserialize, Default)]
pub(crate) struct Test(BTreeMap<String, TestPart>);

impl Test {
    pub(crate) fn iter(&self) -> std::collections::btree_map::Iter<'_, String, TestPart> {
        self.0.iter()
    }

    pub(crate) fn get(&self, key: &str) -> Option<&TestPart> {
        self.0.get(key)
    }

    pub(crate) fn get_questions(&self) -> HashMap<String, &Question> {
        self.0.values()
            .flat_map(|part| part.sections.values())
            .flat_map(|section| section.questions.iter())
            .map(|question| (question.id.clone(), question))
            .collect()
    }

    pub(crate) fn get_part_questions(&self, part_id: &str) -> Vec<&Question> {
        match self.0.get(part_id) {
            None => Default::default(),
            Some(part) => part.sections.iter()
                .flat_map(|section| section.1.questions.iter())
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TestPart {
    pub(crate) introduction: String,
    pub(crate) sections: BTreeMap<String, Section>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Section {
    pub(crate) introduction: String,
    pub(crate) questions: Vec<Question>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Question {
    pub(crate) id: String,
    pub(crate) question: String,
    pub(crate) choices: BTreeMap<char, AnswerChoice>,
    // FIXME: brak logiki tego pola
    canceled: bool,
}

#[derive(Clone, Debug, Deserialize, Default)]
pub(crate) struct RawTest(BTreeMap<String, RawTestPart>);

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RawTestPart {
    introduction: String,
    sections: BTreeMap<String, RawSection>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RawSection {
    introduction: String,
    questions: Vec<RawQuestion>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RawQuestion {
    question: String,
    choices: Vec<AnswerChoice>,
    #[serde(default = "ret_false")]
    canceled: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AnswerChoice {
    pub(crate) answer: String,
    pub(crate) correct: bool,
}

impl Into<Test> for RawTest {
    fn into(self) -> Test {
        let new_test = self.0.iter()
            .map(|part: (&String, &RawTestPart)| {
                let part_id = part.0.clone();
                let test_part = TestPart::from_raw(part.1, &part_id);
                (part_id, test_part)
            })
            .collect();
        Test(new_test)
    }
}

impl TestPart {
    fn from_raw(value: &RawTestPart, part_id: &str) -> Self {
        let new_sections = value.sections.iter()
            .map(|section: (&String, &RawSection)| {
                let section_id = section.0.clone();
                let new_section = Section::from_raw(section.1, part_id, &section_id);
                (section_id, new_section)
            })
            .collect();
        TestPart {
            introduction: value.introduction.clone(),
            sections: new_sections,
        }
    }
}

static QUESTION_IDS: [char; 8] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H',
];

impl Section {
    fn from_raw(value: &RawSection, part_id: &str, section_id: &str) -> Self {
        let new_questions = value.questions.iter()
            .enumerate()
            .map(|(i, question)| {
                let question_id = format!("q{part_id}_{section_id}_{i}");
                let new_choices = QUESTION_IDS.into_iter()
                    .zip(question.choices.clone().into_iter())
                    .collect();
                Question {
                    id: question_id,
                    question: question.question.clone(),
                    canceled: question.canceled,
                    choices: new_choices,
                }
            })
            .collect();
        Section {
            introduction: value.introduction.clone(),
            questions: new_questions,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct UserResponse {
    pub(crate) user_answer: char,
    pub(crate) correct_answer: char,
}

pub(crate) struct TestStateMainPageElem {
    pub(crate) test_id: String,
    pub(crate) answered_q: usize,
    pub(crate) total_q: usize,
    pub(crate) answered_good_q: usize,
    pub(crate) answered_bad_q: usize,
}

pub(crate) struct TestStateMainPageTotals {
    pub(crate) answered_good_q: usize,
    pub(crate) answered_bad_q: usize,
    pub(crate) no_answer: usize,
    pub(crate) total_q: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TestStatePartPage {
    pub(crate) introduction: String,
    pub(crate) sections: Vec<TestStatePartPageSection>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TestStatePartPageSection {
    pub(crate) introduction: String,
    pub(crate) questions: Vec<TestStatePartPageQuestion>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TestStatePartPageQuestion {
    pub(crate) id: String,
    pub(crate) question: String,
    pub(crate) choices: BTreeMap<char, TestStatePartPageAnswerChoice>,
    pub(crate) user_answer: Option<char>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TestStatePartPageAnswerChoice {
    pub(crate) answer: String,
    pub(crate) correct: bool,
    pub(crate) user_selected: bool,
    pub(crate) choice_class: String,
    pub(crate) id: String,
}
