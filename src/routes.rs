use crate::giga_test::{
    get_index_tests_state, get_index_totals, get_part_state, responses_from_form_data,
};
use crate::models::UserResponseData;
use crate::pages::{About, Index, Part};
use crate::AppState;
use crate::Error;
use askama::Template;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::{get, post, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tower_sessions::Session;

const GT_RESP_KEY: &str = "giga_test_responses";
const GT_FINISHED_KEY: &str = "giga_test_finished";
const GT_COUNT_CANCELED_KEY: &str = "giga_test_count_canceled";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CountCanceled(bool);

impl Default for CountCanceled {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct TestFinished(bool);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct Tmpl {
            description: String,
        }

        let status = match &self {
            Error::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::IllegalCharacters
            | Error::IntConversion(_)
            | Error::WrongSize
            | Error::CookieParsing(_) => StatusCode::BAD_REQUEST,
            Error::Join(_) | Error::Compression(_) | Error::Axum(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        let tmpl = Tmpl {
            description: "nie wiem".to_string(),
        };
        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}

async fn get_index(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let test_responses: UserResponseData = session
        .get(GT_RESP_KEY)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    let count_canceled: CountCanceled = session
        .get(GT_COUNT_CANCELED_KEY)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    let test_finished: TestFinished = session
        .get(GT_FINISHED_KEY)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    let index_tests_state =
        get_index_tests_state(&state.giga_test, &test_responses, count_canceled.0);
    let totals = get_index_totals(&index_tests_state);
    Ok(Html(
        Index::new(
            &index_tests_state,
            &totals,
            count_canceled.0,
            test_finished.0,
        )
        .render()?,
    ))
}

async fn get_part(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<usize>,
) -> Result<impl IntoResponse, Error> {
    let test_id = id.to_string();
    let test_part = state
        .giga_test
        .get(&test_id)
        .ok_or(crate::Error::NotFound)?;
    let test_responses: UserResponseData = session
        .get(GT_RESP_KEY)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    let count_canceled: CountCanceled = session
        .get(GT_COUNT_CANCELED_KEY)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    let test_finished: TestFinished = session
        .get(GT_FINISHED_KEY)
        .await
        .unwrap_or_default()
        .unwrap_or_default();

    let part_state = get_part_state(test_part, &test_responses, count_canceled.0);

    Ok(Html(Part::new(&part_state, test_finished.0).render()?))
}

async fn get_about() -> Result<impl IntoResponse, Error> {
    Ok(Html(About::new().render()?))
}

async fn post_answers(
    State(state): State<AppState>,
    session: Session,
    form: Form<HashMap<String, String>>,
) -> Redirect {
    let new_responses: UserResponseData = responses_from_form_data(&form.0, &state.questions_db);

    if !new_responses.is_empty() {
        let test_responses: UserResponseData = session
            .get(GT_RESP_KEY)
            .await
            .unwrap_or_default()
            .unwrap_or_default();
        let all_responses: UserResponseData =
            test_responses.into_iter().chain(new_responses).collect();
        session
            .insert(GT_RESP_KEY, all_responses)
            .await
            .unwrap_or_default();
    }

    Redirect::to("/")
}

async fn submit_toggle_canceled(session: Session) -> Redirect {
    let count_canceled: CountCanceled = session
        .get(GT_COUNT_CANCELED_KEY)
        .await
        .unwrap_or_default()
        .unwrap_or_default();
    session
        .insert(GT_COUNT_CANCELED_KEY, !count_canceled.0)
        .await
        .unwrap_or_default();
    Redirect::to("/")
}

async fn submit_test(session: Session) -> Redirect {
    session
        .insert(GT_FINISHED_KEY, true)
        .await
        .unwrap_or_default();
    Redirect::to("/")
}

async fn start_new_test(session: Session) -> Redirect {
    session
        .insert(GT_RESP_KEY, UserResponseData::new())
        .await
        .unwrap_or_default();
    session
        .insert(GT_FINISHED_KEY, false)
        .await
        .unwrap_or_default();
    Redirect::to("/")
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_index))
        .route("/czesc-{id}", get(get_part))
        .route("/o-co-chodzi-jakby", get(get_about))
        .route("/odpowiedzi", post(post_answers))
        .route("/licz-anulowane", post(submit_toggle_canceled))
        .route("/zakoncz", post(submit_test))
        .route("/od-nowa", post(start_new_test))
}
