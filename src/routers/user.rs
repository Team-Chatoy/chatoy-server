use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;
use sea_orm::{ActiveValue, EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection};
use axum::{extract::{Json, State}, http::StatusCode};

use crate::{AppState, entities::{prelude::*, user}};

use super::Resp;

#[derive(Deserialize)]
pub struct RegisterPayload {
  username: String,
  password: String,
}

async fn check_username(
  db: &DatabaseConnection,
  username: &str,
) -> Result<bool> {
  let user = User::find()
    .filter(user::Column::Username.eq(username))
    .one(db).await?;

  Ok(user.is_none())
}

pub async fn register(
  State(state): State<Arc<AppState>>,
  Json(payload): Json<RegisterPayload>,
) -> (StatusCode, Json<Resp>) {
  match check_username(&state.db, &payload.username).await {
    Err(_) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Resp { code: 1, msg: "Error accessing database!" }),
      );
    },
    Ok(false) => {
      return (
        StatusCode::BAD_REQUEST,
        Json(Resp { code: 2, msg: "This username has been used!" }),
      );
    },
    _ => (),
  }

  let nickname = payload.username.clone();
  let password_hashed = blake3::hash(payload.password.as_bytes()).to_string();

  let new_user = user::ActiveModel {
    username: ActiveValue::Set(payload.username),
    nickname: ActiveValue::Set(nickname),
    password: ActiveValue::Set(password_hashed),
    status: ActiveValue::Set(0),
    ..Default::default()
  };

  match User::insert(new_user).exec(&state.db).await {
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(Resp { code: 3, msg: "Failed to insert a new user into the database!" }),
    ),
    Ok(_) => (
      StatusCode::CREATED,
      Json(Resp { code: 0, msg: "" }),
    ),
  }
}

pub async fn get_user_list(
  State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<user::Model>>) {
  let users = User::find().all(&state.db).await;

  match users {
    Ok(users) => (StatusCode::OK, Json(users)),
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
  }
}
