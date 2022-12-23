use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use sea_orm::EntityTrait;

use crate::{AppState, entities::{prelude::*, session}};

pub async fn get_session_list(
  State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<session::Model>>) {
  warn!("GET /sessions");

  let sessions = Session::find().all(&state.db).await;

  match sessions {
    Ok(sessions) => (StatusCode::OK, Json(sessions)),
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
  }
}
