mod entities;
mod utils;
mod routers;
mod ws;

use std::sync::Arc;

use axum::{Router, routing::{get, post}};
use sea_orm::{Database, DatabaseConnection};

pub struct AppState {
  db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
  let db = Database::connect("sqlite:./data.db?mode=rwc").await
    .expect("Error opening database!");

  let shared_state = Arc::new(AppState { db });

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
