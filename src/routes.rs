use crate::pages::{Index, Part};
use crate::{AppState, Error};
use crate::env;
use crate::models::{TestStateMainPageElem, UserResponse, Test, TestPart, Question, UserResponseData, QuestionsDB, 
    TestStatePartPage, TestStatePartPageSection, TestStatePartPageQuestion, TestStatePartPageAnswerChoice};
use std::collections::HashMap;
use axum::routing::{get, Router};
use axum::body::Body;
use axum::extract::{Form, Json, Path, Query, State};
use axum::http::header::{self, HeaderMap};
use axum::http::{Request, StatusCode};
use axum::response::{AppendHeaders, IntoResponse, Redirect, Response};
use axum::RequestExt;
use tower_http::trace::TraceLayer;
use tower_sessions::Session;
use tower::util::ServiceExt;

const GT_RESP_KEY: &str = "giga_test_responses";
const GT_FINISHED_KEY: &str = "giga_test_finished";

pub struct MetadataRoutes {
    pub css: String,
    pub favicon: String,
    pub index: String,
    pub part: String,
}

impl Default for MetadataRoutes {
    fn default() -> Self {
        MetadataRoutes {
            css: "/style.css".to_string(),
            favicon: "/favicon.png".to_string(),
            index: "/".to_string(),
            part: "/czesc-".to_string(),
        }
    }
}

fn get_index_tests_state(
    test: &Test,
    questions_db: &QuestionsDB,
    test_responses: &UserResponseData,
    test_finished: bool
) -> Vec<TestStateMainPageElem> {
    test.iter()
        .map(|part| {
            let test_id = part.0.clone();
            let part_questions = test.get_part_questions(&test_id);
            let total_q = part_questions.iter().count();

            let (mut answered_q, mut answered_good_q, mut answered_bad_q) = (0, 0, 0);

            for part_question in part_questions {
                let question_id = &part_question.id;
                match test_responses.get(question_id) {
                    None => continue,
                    Some(user_response) => {
                        if ! test_finished {
                            answered_q += 1;
                            continue;
                        }
                        if user_response.user_answer == user_response.correct_answer {
                            answered_good_q += 1;
                        } else {
                            answered_bad_q += 1;
                        }
                    }
                }
            }

            TestStateMainPageElem {
                test_id,
                answered_q,
                total_q,
                answered_good_q,
                answered_bad_q,
            }
        })
        .collect()
}


fn get_part_state(
    test_part: &TestPart,
    test_responses: &UserResponseData,
) -> TestStatePartPage {
    let new_sections = test_part.sections.values()
        .map(|section| {
            let new_questions = section.questions.iter()
                .map(|question| {
                    let user_answer = match test_responses.get(&question.id) {
                        None => None,
                        Some(r) => Some(r.user_answer),
                    };
                    let new_answers = question.choices.iter()
                        .map(|answer| {
                            let user_selected = match user_answer {
                                None => false,
                                Some(r) => &r == answer.0,
                            };
                            let choice_class = "".to_string();
                            let id = format!("{}_{}", question.id, answer.0).to_string();
                            let obj = TestStatePartPageAnswerChoice {
                                answer: answer.1.answer.clone(),
                                correct: answer.1.correct,
                                user_selected,
                                choice_class,
                                id,
                            };
                            (answer.0.clone(), obj)
                        })
                        .collect();
                    TestStatePartPageQuestion {
                        id: question.id.clone(),
                        question: question.question.clone(),
                        choices: new_answers,
                        user_answer,
                    }
                })
                .collect();
            TestStatePartPageSection {
                introduction: section.introduction.clone(),
                questions: new_questions,
            }
        })
        .collect();

    TestStatePartPage {
        introduction: test_part.introduction.clone(),
        sections: new_sections,
    }
}

async fn get_index(
    session: Session,
    request: Request<Body>,
) -> Result<impl IntoResponse, Index<'static>> {
    let test_responses: UserResponseData = session.get(GT_RESP_KEY).await.unwrap().unwrap_or_default();
    let test_finished: bool = session.get(GT_FINISHED_KEY).await.unwrap().unwrap_or(false);
    let index_tests_state = get_index_tests_state(env::giga_test(), env::giga_test_questions(), &test_responses, test_finished);
    Ok(Index::new(&index_tests_state, test_finished).into_response())
}

async fn get_part(
    session: Session,
    Path(id): Path<usize>,
    request: Request<Body>,
) -> Result<impl IntoResponse, Part<'static>> {
    let test_id = id.to_string();
    let test_part = env::giga_test().get(&test_id).unwrap(); // FIXME: poprawna strona błędu
    let test_responses: UserResponseData = session.get(GT_RESP_KEY).await.unwrap().unwrap_or_default();
    let test_finished: bool = session.get(GT_FINISHED_KEY).await.unwrap().unwrap_or(false);

    let part_state = get_part_state(test_part, &test_responses);

    Ok(Part::new(&part_state, test_finished).into_response())
}

async fn post_answers(
    session: Session,
    form: Option<Form<HashMap<String, String>>>,
) -> Result<impl IntoResponse, Index<'static>> {
    let test_responses: UserResponseData = session.get(GT_RESP_KEY).await.unwrap().unwrap_or_default();

    if let Some(form) = form {
        let new_responses: UserResponseData = form.iter()
            .map(|answer| {
                let question_id = answer.0;
                let user_answer = answer.1.chars().next().unwrap();

                let correct_answer = env::giga_test_questions().get(question_id)
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
            .collect();
        let all_responses: UserResponseData = test_responses.into_iter().chain(new_responses).collect();
        session.insert(GT_RESP_KEY, all_responses).await.unwrap();
    }

    // FIXME: odczytuje choć przed chwilą zapisałem
    let test_responses: UserResponseData = session.get(GT_RESP_KEY).await.unwrap().unwrap_or_default();

    // FIXME: duplikat
    let test_finished: bool = session.get(GT_FINISHED_KEY).await.unwrap().unwrap_or(false);
    let index_tests_state = get_index_tests_state(env::giga_test(), env::giga_test_questions(), &test_responses, test_finished);
    Ok(Index::new(&index_tests_state, test_finished).into_response())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_index).post(post_answers))
        .route("/czesc-:id", get(get_part))
        //.merge(assets::routes())
}

#[cfg(test)]
mod tests {
    use crate::db::write::Entry;
    use crate::env::base_path;
    use crate::routes;
    use crate::test_helpers::{make_app, Client};
    use reqwest::{header, StatusCode};
    use serde::Serialize;

    #[tokio::test]
    async fn unknown_paste() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;

        let res = client.get(&base_path().join("000000")).send().await?;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn insert_via_form() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;

        let data = routes::form::Entry {
            text: "FooBarBaz".to_string(),
            extension: Some("rs".to_string()),
            expires: "0".to_string(),
            password: "".to_string(),
        };

        let res = client.post(base_path().path()).form(&data).send().await?;
        assert_eq!(res.status(), StatusCode::SEE_OTHER);

        let location = res.headers().get("location").unwrap().to_str()?;

        let res = client
            .get(location)
            .header(header::ACCEPT, "text/html; charset=utf-8")
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);

        let header = res.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(header.to_str().unwrap().contains("text/html"));

        let content = res.text().await?;
        assert!(content.contains("FooBarBaz"));

        let res = client
            .get(location)
            .header(header::ACCEPT, "text/html; charset=utf-8")
            .query(&[("fmt", "raw")])
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);

        let header = res.headers().get(header::CONTENT_TYPE).unwrap();
        assert!(header.to_str().unwrap().contains("text/plain"));

        let content = res.text().await?;
        assert_eq!(content, "FooBarBaz");

        Ok(())
    }

    #[tokio::test]
    async fn burn_after_reading() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;

        let data = routes::form::Entry {
            text: "FooBarBaz".to_string(),
            extension: None,
            expires: "burn".to_string(),
            password: "".to_string(),
        };

        let res = client.post(base_path().path()).form(&data).send().await?;
        assert_eq!(res.status(), StatusCode::SEE_OTHER);

        let location = res.headers().get("location").unwrap().to_str()?;

        // Location is the `/burn/foo` page not the paste itself, so remove the prefix.
        let location = location.replace("burn/", "");

        let res = client
            .get(&location)
            .header(header::ACCEPT, "text/html; charset=utf-8")
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .get(&location)
            .header(header::ACCEPT, "text/html; charset=utf-8")
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn burn_after_reading_with_encryption() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;
        let password = "asd";

        let data = routes::form::Entry {
            text: "FooBarBaz".to_string(),
            extension: None,
            expires: "burn".to_string(),
            password: password.to_string(),
        };

        let res = client.post(base_path().path()).form(&data).send().await?;
        assert_eq!(res.status(), StatusCode::SEE_OTHER);

        let location = res.headers().get("location").unwrap().to_str()?;

        // Location is the `/burn/foo` page not the paste itself, so remove the prefix.
        let location = location.replace("burn/", "");

        let res = client
            .get(&location)
            .header(header::ACCEPT, "text/html; charset=utf-8")
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);

        #[derive(Debug, Serialize)]
        struct Form {
            password: String,
        }

        let data = Form {
            password: password.to_string(),
        };

        let res = client
            .post(&location)
            .form(&data)
            .header(header::ACCEPT, "text/html; charset=utf-8")
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);

        let res = client
            .get(&location)
            .header(header::ACCEPT, "text/html; charset=utf-8")
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn insert_via_json() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;

        let entry = Entry {
            text: "FooBarBaz".to_string(),
            ..Default::default()
        };

        let res = client.post(base_path().path()).json(&entry).send().await?;
        assert_eq!(res.status(), StatusCode::OK);

        let payload = res.json::<routes::json::RedirectResponse>().await?;

        let res = client.get(&payload.path).send().await?;
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await?, "FooBarBaz");

        Ok(())
    }

    #[tokio::test]
    async fn insert_via_json_encrypted() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;
        let password = "SuperSecretPassword";

        let entry = Entry {
            text: "FooBarBaz".to_string(),
            password: Some(password.to_string()),
            ..Default::default()
        };

        let res = client.post(base_path().path()).json(&entry).send().await?;
        assert_eq!(res.status(), StatusCode::OK);

        let payload = res.json::<routes::json::RedirectResponse>().await?;

        let res = client
            .get(&payload.path)
            .header("Wastebin-Password", password)
            .send()
            .await?;

        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.text().await?, "FooBarBaz");

        Ok(())
    }

    #[tokio::test]
    async fn delete_via_link() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;

        let data = routes::form::Entry {
            text: "FooBarBaz".to_string(),
            extension: None,
            expires: "0".to_string(),
            password: "".to_string(),
        };

        let res = client.post(base_path().path()).form(&data).send().await?;
        let uid_cookie = res.cookies().find(|cookie| cookie.name() == "uid");
        assert!(uid_cookie.is_some());
        assert_eq!(res.status(), StatusCode::SEE_OTHER);

        let location = res.headers().get("location").unwrap().to_str()?;
        let id = location.replace(base_path().path(), "");

        let res = client
            .get(&base_path().join(&format!("delete/{id}")))
            .send()
            .await?;
        assert_eq!(res.status(), StatusCode::SEE_OTHER);

        let res = client.get(&base_path().join(&id)).send().await?;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);

        Ok(())
    }

    #[tokio::test]
    async fn download() -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::new(make_app()?).await;

        let data = routes::form::Entry {
            text: "FooBarBaz".to_string(),
            extension: None,
            expires: "0".to_string(),
            password: "".to_string(),
        };

        let res = client.post(base_path().path()).form(&data).send().await?;
        assert_eq!(res.status(), StatusCode::SEE_OTHER);

        let location = res.headers().get("location").unwrap().to_str()?;
        let res = client.get(&format!("{location}?dl=cpp")).send().await?;
        assert_eq!(res.status(), StatusCode::OK);

        let content = res.text().await?;
        assert_eq!(content, "FooBarBaz");

        Ok(())
    }
}
