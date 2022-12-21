mod entities;
mod routers;

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
    .route("/", get(|| async { "Hello, world!" }))
    .route("/users", get(routers::get_user_list))
    .route("/users", post(routers::register))
    .with_state(shared_state);

  axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
    .serve(app.into_make_service()).await.unwrap();
}
