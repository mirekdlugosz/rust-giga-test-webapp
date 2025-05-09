use crate::pages::{Index, Part, ErrorResponse};
use crate::AppState;
use crate::models::UserResponseData;
use crate::giga_test::{get_index_tests_state, get_part_state, get_index_totals, responses_from_form_data};
use std::collections::HashMap;
use axum::routing::{get, post, Router};
use axum::extract::{Form, Path, State};
use axum::response::{IntoResponse, Redirect};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

const GT_RESP_KEY: &str = "giga_test_responses";
const GT_FINISHED_KEY: &str = "giga_test_finished";
const GT_COUNT_CANCELED_KEY: &str = "giga_test_count_canceled";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct CountCanceled(bool);

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TestFinished(bool);

impl Default for CountCanceled {
    fn default() -> Self {
        Self(false)
    }
}

impl Default for TestFinished {
    fn default() -> Self {
        Self(false)
    }
}

async fn get_index(
    State(state): State<AppState>,
    session: Session,
) -> Result<impl IntoResponse, ErrorResponse<'static>> {
    let test_responses: UserResponseData = session.get(GT_RESP_KEY).await.unwrap().unwrap_or_default();
    let count_canceled: CountCanceled = session.get(GT_COUNT_CANCELED_KEY).await.unwrap().unwrap_or_default();
    let test_finished: TestFinished = session.get(GT_FINISHED_KEY).await.unwrap().unwrap_or_default();
    let index_tests_state = get_index_tests_state(&state.giga_test, &test_responses, count_canceled.0);
    let totals = get_index_totals(&index_tests_state);
    Ok(Index::new(&index_tests_state, &totals, count_canceled.0, test_finished.0).into_response())
}

async fn get_part(
    State(state): State<AppState>,
    session: Session,
    Path(id): Path<usize>,
) -> Result<impl IntoResponse, ErrorResponse<'static>> {
    let test_id = id.to_string();
    let test_part = state.giga_test.get(&test_id).ok_or(crate::Error::NotFound)?;
    let test_responses: UserResponseData = session.get(GT_RESP_KEY).await.unwrap().unwrap_or_default();
    let count_canceled: CountCanceled = session.get(GT_COUNT_CANCELED_KEY).await.unwrap().unwrap_or_default();
    let test_finished: TestFinished = session.get(GT_FINISHED_KEY).await.unwrap().unwrap_or_default();

    let part_state = get_part_state(test_part, &test_responses, count_canceled.0);

    Ok(Part::new(&part_state, test_finished.0).into_response())
}

async fn post_answers(
    State(state): State<AppState>,
    session: Session,
    form: Option<Form<HashMap<String, String>>>,
) -> Result<impl IntoResponse, ErrorResponse<'static>> {
    let new_responses: UserResponseData = form.map_or_else(
        || UserResponseData::new(),
        |form| responses_from_form_data(&form.0, &state.questions_db)
    );

    if ! new_responses.is_empty() {
        let test_responses: UserResponseData = session.get(GT_RESP_KEY).await.unwrap().unwrap_or_default();
        let all_responses: UserResponseData = test_responses.into_iter().chain(new_responses).collect();
        session.insert(GT_RESP_KEY, all_responses).await.unwrap();
    }

    get_index(axum::extract::State(state), session).await
}

async fn submit_toggle_canceled(
    session: Session,
    _form: Option<Form<HashMap<String, String>>>,
) -> Redirect {
    let count_canceled: CountCanceled = session.get(GT_COUNT_CANCELED_KEY).await.unwrap().unwrap_or_default();
    session.insert(GT_COUNT_CANCELED_KEY, !count_canceled.0).await.unwrap();
    Redirect::to("/")
}

async fn submit_test(
    session: Session,
    _form: Option<Form<HashMap<String, String>>>,
) -> Redirect {
    session.insert(GT_FINISHED_KEY, true).await.unwrap();
    Redirect::to("/")
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_index).post(post_answers))
        .route("/czesc-:id", get(get_part))
        .route("/licz-anulowane", post(submit_toggle_canceled))
        .route("/zakoncz", post(submit_test))
}
