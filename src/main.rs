mod entities;
mod utils;
mod routers;
mod msg;
mod channel;
mod ws;

use std::sync::Arc;

use tokio::sync::broadcast;
use tower_http::cors::{CorsLayer, self};
use axum::{Router, routing::{get, post}, http::{self, Method}};
use sea_orm::{Database, DatabaseConnection};

use crate::channel::ChannelEvent;

#[macro_use]
extern crate log;

pub struct AppState {
  db: DatabaseConnection,
  sender: broadcast::Sender<ChannelEvent>,
}

#[tokio::main]
async fn main() {
  env_logger::init();

  let db = Database::connect("sqlite:./data.db?mode=rwc").await
    .expect("Error opening database!");

  info!("Database connected!");

  let (sender, _) = broadcast::channel::<ChannelEvent>(256);

  info!("Broadcast channel created!");

  let shared_state = Arc::new(AppState { db, sender });

  let app = Router::new()
    .route("/", get(|| async { "Hello, Chatoy!" }))
    .route("/ws", get(ws::ws))
    .route("/login", post(routers::login))
    .route("/users", post(routers::register))
    .route("/users/:id", get(routers::get_user))
    .route("/users", get(routers::get_user_list))
    .route("/sessions", get(routers::get_session_list))
    .route("/rooms", post(routers::new_room))
    .route("/rooms/:id", get(routers::get_room))
    .route("/rooms/me", get(routers::get_my_room))
    .route("/rooms", get(routers::get_room_list))
    .route("/members", get(routers::get_member_list))
    .route("/rooms/:id/join", post(routers::join_room))
    .layer(
      CorsLayer::new()
        .allow_origin(cors::Any)
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![
          http::header::CONTENT_TYPE,
          http::header::AUTHORIZATION,
        ]),
    )
    .with_state(shared_state);

  axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
    .serve(app.into_make_service()).await.unwrap();
}
