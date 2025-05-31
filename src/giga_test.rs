use crate::models::{
    AnswerChoice, AnswersDB, PlaceBucket, Question, RawTest, Section, Test, TestPart,
    TestPartTally, TestStateMainPageElem, TestStateMainPageTotals, TestStatePartPage,
    TestStatePartPageAnswerChoice, TestStatePartPageQuestion, TestStatePartPageSection,
    UserResponse, UserResponseData,
};
use std::collections::HashMap;

// Table with a number of points received by each participant of original competition,
// copied from http://www.psxextreme.bmp.net.pl/gigatest.htm (Web Archive)
// get_user_place() uses that to tell which place user would have, had she participated
static GT_RESULTS: &[usize] = &[
    276, 274, 271, 270, 267, 264, 262, 260, 259, 256, 255, 254, 252, 250, 248, 247, 246, 245, 244,
    243, 242, 241, 240, 239, 238, 236, 235, 234, 233, 232, 231, 230, 228, 227, 226, 225, 224, 223,
    222, 221, 220, 219, 218, 217, 216, 215, 214, 213, 212, 211, 210, 209, 208, 207, 206, 205, 204,
    203, 202, 201, 200, 199, 198, 197, 196, 195, 194, 193, 192, 191, 190, 189, 188, 187, 186, 185,
    184, 183, 182, 181, 180, 179, 178, 177, 176, 175, 174, 173, 172, 171, 170, 169, 168, 167, 166,
    165, 164, 163, 162, 161, 160, 159, 157, 156, 155, 154, 153, 152, 151, 150, 149, 148, 147, 146,
    145, 144, 143, 142, 141, 140, 139, 138, 137, 136, 135, 134, 133, 132, 131, 130, 129, 128, 127,
    126, 125, 124, 123, 122, 121, 120, 119, 118, 117, 115, 114, 113, 112, 111, 110, 109, 108, 107,
    106, 105, 104, 103, 102, 100, 99, 98, 96, 95, 94, 93, 92, 91, 90, 89, 88, 87, 86, 85, 84, 82,
    81, 80, 79, 78, 77, 76, 75, 74, 73, 72, 71, 70, 69, 68, 67, 66, 64, 63, 62, 61, 60, 59, 58, 57,
    56, 55, 53, 52, 51, 50, 49, 47, 46, 45, 44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 30,
    29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 0,
];

pub(crate) fn get_giga_test(preprocessor: &dyn Fn(&str) -> String) -> Test {
    let giga_test_toml = include_str!("../resources/gigatest.toml");
    let processed_giga_test_toml = preprocessor(giga_test_toml);
    toml::from_str::<RawTest>(&processed_giga_test_toml)
        .unwrap_or_default()
        .into()
}

fn display_canceled_question(count_canceled: bool, question: &Question) -> bool {
    count_canceled || !question.canceled
}

fn get_user_place(correct_answers: usize) -> usize {
    GT_RESULTS
        .iter()
        .position(|&t| correct_answers >= t)
        .unwrap_or(0)
        + 1
}

fn tally_test_part(
    test_part: &TestPart,
    test_responses: &UserResponseData,
    count_canceled: bool,
) -> TestPartTally {
    let part_questions: Vec<&Question> = test_part
        .get_questions()
        .iter()
        .copied()
        .filter(|q| display_canceled_question(count_canceled, q))
        .collect();
    let total_q = part_questions.len();

    let (answered_good_q, answered_bad_q) = part_questions
        .iter()
        .filter_map(|question| test_responses.get(&question.id))
        .map(|user_response| user_response.correct_answer == Some(user_response.user_answer))
        .fold((0, 0), |(t, f), is_correct| match is_correct {
            true => (t + 1, f),
            false => (t, f + 1),
        });

    let answered_q = answered_good_q + answered_bad_q;

    TestPartTally::new(answered_q, total_q, answered_good_q, answered_bad_q)
}

pub(crate) fn get_index_tests_state(
    test: &Test,
    test_responses: &UserResponseData,
    count_canceled: bool,
) -> Vec<TestStateMainPageElem> {
    test.iter()
        .map(|(test_id, part)| {
            let part_tally = tally_test_part(part, test_responses, count_canceled);
            TestStateMainPageElem::from(test_id, part_tally)
        })
        .collect()
}

pub(crate) fn get_index_totals(
    index_tests_state: &[TestStateMainPageElem],
) -> TestStateMainPageTotals {
    let (answered_good_q, answered_bad_q, total_q) =
        index_tests_state.iter().fold((0, 0, 0), |(g, b, t), x| {
            (g + x.answered_good_q, b + x.answered_bad_q, t + x.total_q)
        });
    let answered_total_q = answered_good_q + answered_bad_q;
    let place = get_user_place(answered_good_q);
    let place_bucket = match place {
        1 => PlaceBucket::Winner,
        2..=7 => PlaceBucket::ConsolationPrize,
        8..=61 => PlaceBucket::NamePrinted,
        62..=232 => PlaceBucket::NameWebsite,
        _ => PlaceBucket::Loser,
    };
    TestStateMainPageTotals {
        answered_good_q,
        answered_bad_q,
        answered_total_q,
        total_q,
        place,
        place_bucket,
    }
}

pub(crate) fn get_part_state(
    test_part: &TestPart,
    test_responses: &UserResponseData,
    count_canceled: bool,
) -> TestStatePartPage {
    fn generate_answers(
        question_id: &str,
        answer_id: char,
        answer: &AnswerChoice,
        user_answer: Option<char>,
    ) -> (char, TestStatePartPageAnswerChoice) {
        let user_selected = user_answer.is_some_and(|r| r == answer_id);
        let choice_class = if answer.correct {
            "correct"
        } else if user_selected {
            "incorrect "
        } else {
            ""
        }
        .to_string();
        let id = format!("{question_id}_{answer_id}").to_string();
        let obj = TestStatePartPageAnswerChoice {
            answer: answer.answer.clone(),
            correct: answer.correct,
            user_selected,
            choice_class,
            id,
        };
        (answer_id, obj)
    }

    let generate_questions = |question: &Question| {
        let user_answer = test_responses.get(&question.id).map(|r| r.user_answer);
        let new_answers = question
            .choices
            .iter()
            .map(|(answer_id, answer)| {
                generate_answers(&question.id, *answer_id, answer, user_answer)
            })
            .collect();
        TestStatePartPageQuestion {
            id: question.id.clone(),
            question: question.question.clone(),
            choices: new_answers,
            user_answer,
            canceled: question.canceled,
        }
    };

    let generate_sections = |section: &Section| {
        let new_questions = section
            .questions
            .iter()
            .filter(|q| display_canceled_question(count_canceled, q))
            .map(generate_questions)
            .collect();
        TestStatePartPageSection {
            introduction: section.introduction.clone(),
            questions: new_questions,
        }
    };

    let mut sorted: Vec<_> = test_part.sections.iter().collect();
    sorted.sort_by_key(|(key, _)| key.parse().unwrap_or(i32::MAX));
    let new_sections = sorted
        .iter()
        .map(|(_, section)| generate_sections(section))
        .collect();

    TestStatePartPage {
        introduction: test_part.introduction.clone(),
        sections: new_sections,
    }
}

pub(crate) fn responses_from_form_data(
    form_data: &HashMap<String, String>,
    questions_db: &AnswersDB,
) -> UserResponseData {
    form_data
        .iter()
        .filter_map(|answer| {
            let question_id = answer.0;
            let user_answer = answer.1.chars().next()?;

            let correct_answer = questions_db.get(question_id).copied().flatten();

            let ur = UserResponse {
                user_answer,
                correct_answer,
            };
            Some((question_id.clone(), ur))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_place_best() {
        let user_place = get_user_place(300);
        assert_eq!(user_place, 1);
    }

    #[test]
    fn test_user_place_same() {
        let user_place = get_user_place(270);
        assert_eq!(user_place, 4);
    }

    #[test]
    fn test_user_place_different() {
        let user_place = get_user_place(272);
        assert_eq!(user_place, 3);
    }

    #[test]
    fn test_user_place_worst() {
        let user_place = get_user_place(3);
        assert_eq!(user_place, 233);
    }

    #[test]
    fn test_user_place_zero() {
        let user_place = get_user_place(0);
        assert_eq!(user_place, 233);
    }
}
