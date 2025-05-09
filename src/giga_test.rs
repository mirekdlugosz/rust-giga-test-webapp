use std::collections::HashMap;

use crate::models::{TestStateMainPageElem, Test, RawTest, TestPart, Question, UserResponseData, QuestionsDB,
    TestStatePartPage, TestStatePartPageSection, TestStatePartPageQuestion, TestStatePartPageAnswerChoice,
    TestStateMainPageTotals, Section, AnswerChoice, TestPartTally, UserResponse
};
// FIXME: responsibilities:
// - reading TOML and providing state-compatible test model
// - modifying session-backed response structure
// - preparing data for displaying in page templates - including all necessary calculations

pub(crate) fn get_giga_test(preprocessor: &dyn Fn(&str) -> String) -> Test {
    let giga_test_toml = include_str!("../resources/gigatest.toml");
    let processed_giga_test_toml = preprocessor(giga_test_toml);
    toml::from_str::<RawTest>(&processed_giga_test_toml)
        .unwrap_or_default()
        .into()
}

fn tally_test_part(
    test_part: &TestPart,
    test_responses: &UserResponseData
) -> TestPartTally {
    let part_questions = test_part.get_questions();
    let total_q = part_questions.len();

    let (answered_good_q, answered_bad_q) = part_questions.iter()
        .filter_map(|question| test_responses.get(&question.id))
        .map(|user_response| user_response.user_answer == user_response.correct_answer)
        .fold((0, 0), |(t, f), is_correct| {
            match is_correct {
                true => (t + 1, f),
                false => (t, f + 1),
            }
        });

    let answered_q = answered_good_q + answered_bad_q;

    TestPartTally::new(
        answered_q,
        total_q,
        answered_good_q,
        answered_bad_q,
    )
}

pub(crate) fn get_index_tests_state(
    test: &Test,
    test_responses: &UserResponseData,
) -> Vec<TestStateMainPageElem> {
    test.iter()
        .map(|(test_id, part)| {
            let part_tally = tally_test_part(part, test_responses);
            TestStateMainPageElem::from(test_id, part_tally)
        })
        .collect()
}

pub(crate) fn get_index_totals(index_tests_state: &[TestStateMainPageElem]) -> TestStateMainPageTotals {
    let (answered_good_q, answered_bad_q, total_q) = index_tests_state.iter()
        .fold((0, 0, 0), |(g, b, t), x| (g + x.answered_good_q, b + x.answered_bad_q, t + x.total_q));
    let no_answer = total_q - answered_good_q - answered_bad_q;
    TestStateMainPageTotals{
        answered_good_q,
        answered_bad_q,
        no_answer,
        total_q,
    }
}

pub(crate) fn get_part_state(
    test_part: &TestPart,
    test_responses: &UserResponseData,
) -> TestStatePartPage {
    fn generate_answers(
        question_id: &str,
        answer_id: &char,
        answer: &AnswerChoice,
        user_answer: Option<char>,
    ) -> (char, TestStatePartPageAnswerChoice) {
        let user_selected = match user_answer {
            None => false,
            Some(r) => &r == answer_id,
        };
        let choice_class = match (user_selected, answer.correct) {
            (true, true) => "poprawnie".to_string(),
            (true, false) => "niepoprawnie".to_string(),
            (false, true) => "poprawnie".to_string(),
            (false, false) => "".to_string(),
        };
        let id = format!("{}_{}", question_id, answer_id).to_string();
        let obj = TestStatePartPageAnswerChoice {
            answer: answer.answer.clone(),
            correct: answer.correct,
            user_selected,
            choice_class,
            id,
        };
        (answer_id.clone(), obj)
    }

    let generate_questions = |question: &Question| {
        let user_answer = match test_responses.get(&question.id) {
            None => None,
            Some(r) => Some(r.user_answer),
        };
        let new_answers = question.choices.iter()
            .map(|(answer_id, answer)| generate_answers(&question.id, answer_id, answer, user_answer))
            .collect();
        TestStatePartPageQuestion {
            id: question.id.clone(),
            question: question.question.clone(),
            choices: new_answers,
            user_answer,
        }
    };

    let generate_sections = |section: &Section| {
        let new_questions = section.questions.iter()
            .map(generate_questions)
            .collect();
        TestStatePartPageSection {
            introduction: section.introduction.clone(),
            questions: new_questions,
        }
    };

    let new_sections = test_part.sections.values()
        .map(generate_sections)
        .collect();

    TestStatePartPage {
        introduction: test_part.introduction.clone(),
        sections: new_sections,
    }
}

pub(crate) fn responses_from_form_data(form_data: &HashMap<String, String>, questions_db: &QuestionsDB) -> UserResponseData {
    form_data.iter()
        .map(|answer| {
            let question_id = answer.0;
            let user_answer = answer.1.chars().next().unwrap();

            let correct_answer = questions_db.get(question_id)
                .unwrap()
                .choices.iter()
                .find(|choice| choice.1.correct)
                .map(|choice| choice.0)
                .unwrap()
                .clone();

            let ur = UserResponse {
                user_answer,
                correct_answer,
            };
            (question_id.clone(), ur)
        })
    .collect()
}
