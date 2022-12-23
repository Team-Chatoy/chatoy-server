mod user;
mod session;
mod room;

use serde::Serialize;

pub use user::{login, register, get_user_list};
pub use session::get_session_list;
pub use room::{new_room, get_room_list, get_member_list, join_room};

#[derive(Serialize)]
pub struct Resp {
  code: i32,
  msg: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ErrOr<T> {
  Res(T),
  Err(Resp),
}
