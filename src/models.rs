use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;

fn ret_false() -> bool {
    false
}

pub(crate) type UserResponseData = HashMap<String, UserResponse>;
pub(crate) type AnswersDB = HashMap<String, Option<char>>;

pub(crate) enum PlaceBucket {
    Winner,
    ConsolationPrize,
    NamePrinted,
    NameWebsite,
    Loser,
}

impl fmt::Display for PlaceBucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display_text = match self {
            PlaceBucket::Winner => "winner",
            PlaceBucket::ConsolationPrize => "consolation-prize",
            PlaceBucket::NamePrinted => "name-printed",
            PlaceBucket::NameWebsite => "name-website",
            PlaceBucket::Loser => "loser",
        };
        write!(f, "{display_text}")
    }
}

#[derive(Clone, Debug, Deserialize, Default)]
pub(crate) struct Test(BTreeMap<String, TestPart>);

impl Test {
    pub(crate) fn iter(&self) -> std::collections::btree_map::Iter<'_, String, TestPart> {
        self.0.iter()
    }

    pub(crate) fn get(&self, key: &str) -> Option<&TestPart> {
        self.0.get(key)
    }

    pub(crate) fn get_correct_answers(&self) -> AnswersDB {
        self.0
            .values()
            .flat_map(|part| part.sections.values())
            .flat_map(|section| section.questions.iter())
            .map(|question| {
                let correct_answer = question
                    .choices
                    .iter()
                    .find(|choice| choice.1.correct)
                    .map(|choice| *choice.0);
                (question.id.clone(), correct_answer)
            })
            .collect()
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TestPart {
    pub(crate) introduction: String,
    pub(crate) sections: BTreeMap<String, Section>,
}

impl TestPart {
    pub(crate) fn get_questions(&self) -> Vec<&Question> {
        self.sections
            .iter()
            .flat_map(|(_, section)| section.questions.iter())
            .collect()
    }
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
    pub(crate) canceled: bool,
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

impl From<RawTest> for Test {
    fn from(val: RawTest) -> Self {
        let new_test = val
            .0
            .iter()
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
        let new_sections = value
            .sections
            .iter()
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

static QUESTION_IDS: [char; 8] = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];

impl Section {
    fn from_raw(value: &RawSection, part_id: &str, section_id: &str) -> Self {
        let new_questions = value
            .questions
            .iter()
            .enumerate()
            .map(|(i, question)| {
                let question_id = format!("q{part_id}_{section_id}_{i}");
                let new_choices = QUESTION_IDS
                    .into_iter()
                    .zip(question.choices.clone())
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

pub(crate) struct TestPartTally {
    answered_q: usize,
    total_q: usize,
    answered_good_q: usize,
    answered_bad_q: usize,
}

impl TestPartTally {
    pub(crate) fn new(
        answered_q: usize,
        total_q: usize,
        answered_good_q: usize,
        answered_bad_q: usize,
    ) -> Self {
        Self {
            answered_q,
            total_q,
            answered_good_q,
            answered_bad_q,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct UserResponse {
    pub(crate) user_answer: char,
    pub(crate) correct_answer: Option<char>,
}

pub(crate) struct TestStateMainPageElem {
    pub(crate) test_id: String,
    pub(crate) pe_id: String,
    pub(crate) pe_date: String,
    pub(crate) answered_q: usize,
    pub(crate) total_q: usize,
    pub(crate) answered_good_q: usize,
    pub(crate) answered_bad_q: usize,
}

impl TestStateMainPageElem {
    pub(crate) fn from(test_id: &str, test_part_tally: TestPartTally) -> Self {
        let TestPartTally {
            answered_q,
            total_q,
            answered_good_q,
            answered_bad_q,
        } = test_part_tally;
        let (pe_id, pe_date) = match test_id {
            "1" => ("37", "wrzesień 2000"),
            "2" => ("38", "październik 2000"),
            "3" => ("39", "listopad 2000"),
            "4" => ("40", "grudzień 2000"),
            "5" => ("41", "styczeń 2001"),
            "6" => ("42", "luty 2001"),
            _ => ("brak", "brak"),
        };
        Self {
            test_id: test_id.to_string(),
            pe_id: pe_id.to_string(),
            pe_date: pe_date.to_string(),
            answered_q,
            total_q,
            answered_good_q,
            answered_bad_q,
        }
    }
}

pub(crate) struct TestStateMainPageTotals {
    pub(crate) answered_good_q: usize,
    pub(crate) answered_bad_q: usize,
    pub(crate) answered_total_q: usize,
    pub(crate) total_q: usize,
    pub(crate) place: usize,
    pub(crate) place_bucket: PlaceBucket,
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
    #[allow(dead_code)]
    pub(crate) user_answer: Option<char>,
    pub(crate) canceled: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct TestStatePartPageAnswerChoice {
    pub(crate) answer: String,
    #[allow(dead_code)]
    pub(crate) correct: bool,
    pub(crate) user_selected: bool,
    pub(crate) choice_class: String,
    pub(crate) id: String,
}
