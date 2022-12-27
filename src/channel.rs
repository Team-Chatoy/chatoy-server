use crate::msg::Msg;

#[derive(Clone, Debug)]
pub struct MsgEvent {
  pub msg: Msg,
}

#[derive(Clone, Debug)]
pub struct CloseEvent {
  pub token: String,
}

#[derive(Clone, Debug)]
pub enum ChannelEvent {
  Msg(MsgEvent),
  Close(CloseEvent),
}

impl ChannelEvent {
  pub fn new_msg(msg: Msg) -> Self {
    Self::Msg(MsgEvent { msg })
  }

  pub fn new_close(token: String) -> Self {
    Self::Close(CloseEvent { token })
  }
}
