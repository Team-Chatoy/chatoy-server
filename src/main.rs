mod entities;
mod utils;
mod routers;
mod msg;
mod ws;

use std::sync::Arc;

use tokio::sync::broadcast;
use axum::{Router, routing::{get, post}};
use sea_orm::{Database, DatabaseConnection};

use crate::msg::Msg;

#[macro_use]
extern crate log;

pub struct AppState {
  db: DatabaseConnection,
  sender: broadcast::Sender<Msg>,
}

#[tokio::main]
async fn main() {
  env_logger::init();

  let db = Database::connect("sqlite:./data.db?mode=rwc").await
    .expect("Error opening database!");

  info!("Database connected!");

  let (sender, _) = broadcast::channel::<Msg>(256);

  info!("Broadcast channel created!");

  let shared_state = Arc::new(AppState { db, sender });

  let app = Router::new()
    .route("/", get(|| async { "Hello, Chatoy!" }))
    .route("/ws", get(ws::ws))
    .route("/login", post(routers::login))
    .route("/users", post(routers::register))
    .route("/users", get(routers::get_user_list))
    .route("/sessions", get(routers::get_session_list))
    .route("/rooms", post(routers::new_room))
    .route("/rooms", get(routers::get_room_list))
    .route("/members", get(routers::get_member_list))
    .with_state(shared_state);

  axum::Server::bind(&"0.0.0.0:4000".parse().unwrap())
    .serve(app.into_make_service()).await.unwrap();
}
