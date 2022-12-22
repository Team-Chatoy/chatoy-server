use anyhow::{Result, bail};
use chrono::Local;
use sea_orm::{EntityTrait, DatabaseConnection, ActiveValue};

use crate::entities::{prelude::*, user, member};

pub async fn auth(
  db: &DatabaseConnection,
  token: &str,
) -> Result<user::Model> {
  let session = Session::find_by_id(token.to_string())
    .one(db).await?;

  if session.is_none() {
    bail!("Please login first!");
  }

  let session = session.unwrap();

  let now = Local::now();

  if session.expired < now {
    bail!("Login status expired!");
  }

  let id = session.user;

  let user = User::find_by_id(id)
    .one(db).await?;

  match user {
    Some(user) => Ok(user),
    None => bail!("User not found!"),
  }
}

pub async fn join_room(
  db: &DatabaseConnection,
  user: i32,
  room: i32,
) -> Result<()> {
  let new_member = member::ActiveModel {
    user: ActiveValue::Set(user),
    room: ActiveValue::Set(room),
    joined: ActiveValue::Set(Local::now()),
  };

  Member::insert(new_member).exec(db).await?;

  Ok(())
}
