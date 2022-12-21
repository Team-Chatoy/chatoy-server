mod user;

use serde::Serialize;

pub use user::{get_user_list, register};

#[derive(Serialize)]
pub struct Resp {
  code: i32,
  msg: &'static str,
}
