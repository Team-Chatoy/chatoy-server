use std::sync::{Arc, Mutex};

use chrono::Local;
use serde::{Deserialize, Serialize};
use axum::{
  extract::{State, Path},
  http::StatusCode,
  Json,
  TypedHeader,
  headers::{Authorization, authorization::Bearer},
};
use sea_orm::{EntityTrait, ActiveValue, QueryFilter, ColumnTrait};
use tokio::task::JoinHandle;

use crate::{AppState, entities::{prelude::*, room, member}, utils::{auth, self}};

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
  info!("POST /rooms");

  let user = match auth(&state.db, &payload.token).await {
    Ok(user) => user,
    Err(err) => {
      error!("{err}");
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrOr::Err(Resp { code: 1, msg: err.to_string() })),
      );
    },
  };

  let new_room = room::ActiveModel {
    name: ActiveValue::Set(payload.name),
    description: ActiveValue::Set(String::new()),
    created: ActiveValue::Set(Local::now()),
    ..Default::default()
  };

  let room_id = match Room::insert(new_room).exec(&state.db).await {
    Err(_) => {
      error!("Failed to insert a new room into the database!");
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrOr::Err(Resp { code: 2, msg: "Failed to insert a new room into the database!".to_string() })),
      );
    },
    Ok(room) => room.last_insert_id,
  };

  let room = Room::find_by_id(room_id)
    .one(&state.db).await.unwrap()
    .unwrap();

  match utils::join_room(&state.db, &user, &room).await {
    Err(_) => {
      error!("Failed to join the new room!");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrOr::Err(Resp { code: 3, msg: "Failed to join the new room!".to_string() })),
      )
    },
    Ok(_) => {
      info!("New room created: `{room_id}`");
      (
        StatusCode::CREATED,
        Json(ErrOr::Res(NewRoomResp { id: room_id })),
      )
    },
  }
}

pub async fn get_room_list(
  State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<room::Model>>) {
  warn!("GET /rooms");

  let rooms = Room::find().all(&state.db).await;

  match rooms {
    Ok(rooms) => (StatusCode::OK, Json(rooms)),
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
  }
}

pub async fn get_member_list(
  State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Vec<member::Model>>) {
  warn!("GET /members");

  let members = Member::find().all(&state.db).await;

  match members {
    Ok(members) => (StatusCode::OK, Json(members)),
    Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(vec![])),
  }
}

#[derive(Deserialize)]
pub struct JoinRoomPayload {
  token: String,
}

pub async fn join_room(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i32>,
  Json(payload): Json<JoinRoomPayload>,
) -> (StatusCode, Json<Resp>) {
  info!("POST /rooms/{id}/join");

  let user = match auth(&state.db, &payload.token).await {
    Ok(user) => user,
    Err(err) => {
      error!("{err}");
      return (
        StatusCode::UNAUTHORIZED,
        Json(Resp { code: 1, msg: err.to_string() }),
      );
    },
  };

  let room = Room::find_by_id(id)
    .one(&state.db).await;

  if let Err(err) = room {
    error!("{err}");
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(Resp { code: 2, msg: "Failed to get the room from the database!".to_string() }),
    );
  }

  let room = room.unwrap();

  if room.is_none() {
    error!("Room `{id}` not found!");
    return (
      StatusCode::BAD_REQUEST,
      Json(Resp { code: 3, msg: format!("Room `{id}` not found!") }),
    );
  }

  let room = room.unwrap();

  match utils::join_room(&state.db, &user, &room).await {
    Err(_) => {
      error!("Failed to join the room `{id}`!");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(Resp { code: 4, msg: format!("Failed to join the room `{id}`!") }),
      )
    },
    Ok(_) => {
      info!("User `{}` joined the room `{}`", user.id, room.id);
      (
        StatusCode::CREATED,
        Json(Resp { code: 0, msg: String::new() }),
      )
    },
  }
}

pub async fn get_room(
  State(state): State<Arc<AppState>>,
  Path(id): Path<i32>,
) -> (StatusCode, Json<ErrOr<room::Model>>) {
  info!("GET /rooms/{}", id);

  let room = Room::find_by_id(id)
    .one(&state.db).await;

  if room.is_err() {
    error!("Error accessing database!");
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(ErrOr::Err(Resp { code: 1, msg: "Error accessing database!".to_string() })),
    );
  }

  let room = room.unwrap();

  if room.is_none() {
    info!("The room `{id}` does not exist!");
    return (
      StatusCode::BAD_REQUEST,
      Json(ErrOr::Err(Resp { code: 2, msg: format!("The room `{id}` does not exist!") })),
    );
  }

  let room = room.unwrap();

  (StatusCode::OK, Json(ErrOr::Res(room)))
}

pub async fn get_my_room(
  State(state): State<Arc<AppState>>,
  TypedHeader(token): TypedHeader<Authorization<Bearer>>
) -> (StatusCode, Json<Vec<room::Model>>) {
  let user = auth(&state.db, token.token()).await;

  if let Err(err) = user {
    error!("{err}");
    return (
      StatusCode::UNAUTHORIZED,
      Json(vec![]),
    );
  }

  let user = user.unwrap();

  let members = Member::find()
    .filter(member::Column::User.eq(user.id))
    .all(&state.db).await;

  if let Err(err) = members {
    error!("{err}");
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(vec![]),
    );
  }

  let members = members.unwrap();

  let rooms = Arc::new(Mutex::new(vec![]));

  let tasks: Vec<JoinHandle<()>> = members.into_iter()
    .map(|member| {
      let state = state.clone();
      let rooms = rooms.clone();
      tokio::task::spawn(async move {
        let room = Room::find_by_id(member.room)
          .one(&state.db).await;
  
        if let Err(err) = room {
          error!("{err}");
          return;
        }
  
        let room = room.unwrap();
  
        if room.is_none() {
          error!("Room `{}` not found!", member.room);
          return;
        }
  
        let room = room.unwrap();
  
        rooms.lock().unwrap().push(room);
      })
    })
    .collect();

  for task in tasks {
    task.await.unwrap();
  }

  let rooms = rooms.lock().unwrap().clone();

  (StatusCode::OK, Json(rooms))
}
