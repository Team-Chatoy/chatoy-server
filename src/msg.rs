use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Local};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TextMsg {
  text: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MsgContent {
  Text(TextMsg),
}

#[derive(Clone, Debug, Serialize)]
pub struct Msg {
  pub uuid: Uuid,
  pub sender: i32,
  pub room: i32,
  pub data: MsgContent,
  pub sent: DateTime<Local>,
  pub modified: bool,
}
