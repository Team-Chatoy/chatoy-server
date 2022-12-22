use std::sync::Arc;

use anyhow::Result;
use chrono::Local;
use rand::Rng;
use serde::Deserialize;
use sea_orm::{ActiveValue, EntityTrait, QueryFilter, ColumnTrait, DatabaseConnection};
use axum::{extract::{Json, State, TypedHeader}, http::StatusCode, headers::UserAgent};

use crate::{AppState, entities::{prelude::*, user, session}};

use super::Resp;

#[derive(Deserialize)]
pub struct UserPayload {
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
  Json(payload): Json<UserPayload>,
) -> (StatusCode, Json<Resp>) {
  match check_username(&state.db, &payload.username).await {
    Err(_) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Resp { code: 1, msg: "Error accessing database!".to_string() }),
      );
    },
    Ok(false) => {
      return (
        StatusCode::BAD_REQUEST,
        Json(Resp { code: 2, msg: "This username has been used!".to_string() }),
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
    slogan: ActiveValue::Set(String::new()),
    status: ActiveValue::Set(0),
    registered: ActiveValue::Set(Local::now()),
    ..Default::default()
  };

  match User::insert(new_user).exec(&state.db).await {
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(Resp { code: 3, msg: "Failed to insert a new user into the database!".to_string() }),
    ),
    Ok(_) => (
      StatusCode::CREATED,
      Json(Resp { code: 0, msg: String::new() }),
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

pub async fn login(
  State(state): State<Arc<AppState>>,
  TypedHeader(user_agent): TypedHeader<UserAgent>,
  Json(payload): Json<UserPayload>,
) -> (StatusCode, Json<Resp>) {
  let user = User::find()
    .filter(user::Column::Status.ne(1))
    .filter(user::Column::Username.eq(payload.username))
    .one(&state.db).await;

  if user.is_err() {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(Resp { code: 1, msg: "Error accessing database!".to_string() }),
    );
  }

  let user = user.unwrap();

  if user.is_none() {
    return (
      StatusCode::BAD_REQUEST,
      Json(Resp { code: 2, msg: "The user does not exist!".to_string() }),
    );
  }

  let user = user.unwrap();

  if user.status == 2 {
    return (
      StatusCode::BAD_REQUEST,
      Json(Resp { code: 3, msg: "The user has been banned!".to_string() }),
    );
  }

  let password_hashed = blake3::hash(payload.password.as_bytes()).to_string();

  if user.password != password_hashed {
    return (
      StatusCode::BAD_REQUEST,
      Json(Resp { code: 4, msg: "Password error!".to_string() }),
    );
  }

  let token: String = { // strange lifetime problem
    const CHARSET: &[u8; 16] = b"0123456789abcdef";
    let mut rng = rand::thread_rng();

    (0..64)
      .map(|_| {
        let idx = rng.gen_range(0..16);
        CHARSET[idx] as char
      })
      .collect()
  };

  let new_session = session::ActiveModel {
    token: ActiveValue::Set(token.clone()),
    user: ActiveValue::Set(user.id),
    agent: ActiveValue::Set(user_agent.to_string()),
    generated: ActiveValue::Set(Local::now()),
    expired: ActiveValue::Set(Local::now() + chrono::Duration::days(2)),
  };

  match Session::insert(new_session).exec(&state.db).await {
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(Resp { code: 5, msg: "Failed to insert new session into the database!".to_string() }),
    ),
    Ok(_) => (
      StatusCode::OK,
      Json(Resp { code: 0, msg: token }),
    ),
  }
}