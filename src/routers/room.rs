use std::sync::Arc;

use chrono::Local;
use serde::{Deserialize, Serialize};
use axum::{extract::State, http::StatusCode, Json};
use sea_orm::{EntityTrait, ActiveValue};

use crate::{AppState, entities::{prelude::*, room, member}, utils::{auth, join_room}};

use super::{ErrOr, Resp};

#[derive(Deserialize)]
pub struct NewRoomPayload {
  token: String,
  name: String,
}

#[derive(Serialize)]
pub struct NewRoomResp {
  id: i32,
}

pub async fn new_room(
  State(state): State<Arc<AppState>>,
  Json(payload): Json<NewRoomPayload>,
) -> (StatusCode, Json<ErrOr<NewRoomResp>>) {
  let user = match auth(&state.db, &payload.token).await {
    Ok(user) => user,
    Err(err) => return (
      StatusCode::UNAUTHORIZED,
      Json(ErrOr::Err(Resp { code: 1, msg: err.to_string() })),
    ),
  };

  let new_room = room::ActiveModel {
    name: ActiveValue::Set(payload.name),
    description: ActiveValue::Set(String::new()),
    created: ActiveValue::Set(Local::now()),
    ..Default::default()
  };

  let room_id = match Room::insert(new_room).exec(&state.db).await {
    Err(_) => return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrOr::Err(Resp { code: 2, msg: "Failed to insert a new room into the database!".to_string() })),
    ),
    Ok(room) => room.last_insert_id,
  };

  match join_room(&state.db, user.id, room_id).await {
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrOr::Err(Resp { code: 3, msg: "Failed to join the room!".to_string() })),
    ),
    Ok(_) => (
      StatusCode::CREATED,
      Json(ErrOr::Res(NewRoomResp { id: room_id })),
    ),
  }
}

pub async fn get_room_list(
  State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<room::Model>>) {
  let rooms = Room::find().all(&state.db).await;

  match rooms {
    Ok(rooms) => (StatusCode::OK, Json(rooms)),
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
  }
}

pub async fn get_member_list(
  State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<member::Model>>) {
  let members = Member::find().all(&state.db).await;

  match members {
    Ok(members) => (StatusCode::OK, Json(members)),
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
  }
}
