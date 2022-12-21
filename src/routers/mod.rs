mod user;

use serde::Serialize;

pub use user::{login, register, get_user_list};

#[derive(Serialize)]
pub struct Resp {
  code: i32,
  msg: &'static str,
}
