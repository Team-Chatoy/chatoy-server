use std::sync::Arc;

use chrono::Local;
use futures::{stream::{StreamExt, SplitSink, SplitStream}, SinkExt};
use axum::{extract::{ws::{WebSocketUpgrade, WebSocket, Message}, State}, response::Response};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, utils::auth, entities::user, msg::{MsgContent, Msg}};

#[derive(Debug, Deserialize)]
struct AuthEvent {
  token: String,
}

#[derive(Debug, Deserialize)]
struct MsgEvent {
  uuid: Uuid,
  room: i32,
  data: MsgContent,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum WsEvent {
  Auth(AuthEvent),
  Msg(MsgEvent),
}

pub async fn ws(
  State(state): State<Arc<AppState>>,
  ws: WebSocketUpgrade,
) -> Response {
  ws.on_upgrade(|socket| handle_ws(state, socket))
}

async fn handle_ws(
  state: Arc<AppState>,
  socket: WebSocket,
) {
  info!("New WebSocket connection...");

  let (ws_out, mut ws_in) = socket.split();

  let user: user::Model;
  let token: String;

  loop {
    let msg = ws_in.next().await;

    if let Some(Ok(msg)) = msg {
      let msg = msg.to_text().unwrap();

      let msg = serde_json::from_str(msg);

      if let Err(err) = msg {
        error!("{err}");
        continue;
      }

      let msg: WsEvent = msg.unwrap();

      if let WsEvent::Auth(auth_msg) = msg {
        info!("Authenticating WebSocket connection...");

        user = match auth(&state.db, &auth_msg.token).await {
          Ok(user) => user,
          Err(err) => {
            error!("{err}");
            continue;
          },
        };

        token = auth_msg.token;

        info!("WebSocket connection authenticated!");

        break;
      }
    }
  }

  tokio::spawn(
    write(
      token.clone(),
      state.clone(),
      ws_out,
    )
  );

  tokio::spawn(read(user, token, state, ws_in));
}

async fn read(
  user: user::Model,
  token: String,
  state: Arc<AppState>,
  ws_in: SplitStream<WebSocket>,
) {
  ws_in
    .for_each(|msg| async {
      let msg = msg.unwrap();
      let msg = msg.to_text().unwrap();
      let msg: WsEvent = serde_json::from_str(msg).unwrap();

      info!("Received message: {:?}", msg);

      if let WsEvent::Msg(msg) = msg {
        state.sender
          .send(crate::MsgChannel {
            token: token.to_string(),
            msg: Msg {
              uuid: msg.uuid,
              sender: user.id,
              room: msg.room,
              data: msg.data,
              sent: Local::now(),
              modified: false,
            },
          }).unwrap();
      }
    }).await;
}

#[derive(Serialize)]
struct MsgForward {
  r#type: &'static str,
  data: Msg,
}

async fn write(
  token: String,
  state: Arc<AppState>,
  mut ws_out: SplitSink<WebSocket, Message>,
) {
  let mut receiver = state.sender.subscribe();

  loop {
    let msg = receiver.recv().await;

    if let Err(err) = msg {
      error!("{err}");
      continue;
    }

    let msg = msg.unwrap();

    if msg.token == token {
      continue;
    }

    ws_out
      .send(Message::Text(
        serde_json::to_string(&MsgForward {
          r#type: "Recv",
          data: msg.msg,
        }).unwrap()
      )).await.unwrap();
  }
}
