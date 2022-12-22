// TODO: handle messages from client

use futures::{stream::{StreamExt, SplitSink, SplitStream}, SinkExt};
use axum::{extract::ws::{WebSocketUpgrade, WebSocket, Message}, response::Response};

pub async fn ws(ws: WebSocketUpgrade) -> Response {
  ws.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket) {
  let (mut sender, mut receiver) = socket.split();

  tokio::spawn(write(sender));
  tokio::spawn(read(receiver));
}

async fn read(receiver: SplitStream<WebSocket>) {
  receiver
    .for_each(|msg| async {
      let msg = msg.unwrap();
      let msg = msg.to_text().unwrap();
      println!("Received message: {}", msg);
    }).await;
}

async fn write(mut sender: SplitSink<WebSocket, Message>) {
  loop {
    sender.send(Message::Text(String::from("Hello!"))).await.unwrap();
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
  }
}
