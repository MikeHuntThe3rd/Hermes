use axum::{
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::Response,
};
use uuid::Uuid;

use tokio::sync::{mpsc, oneshot};

use crate::types::*;
use bytes::Bytes;
use futures::StreamExt;
use tokio::sync::mpsc::channel;

use crate::types::AppState;

pub async fn ws_upgrade(ws: WebSocketUpgrade, State(inf): State<AppState>) -> Response {
    return ws.on_upgrade(|socket| test_handler(socket, State(inf), Uuid::new_v4()));
}

async fn test_handler(mut socket: WebSocket, State(inf): State<AppState>, room_id: Uuid) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let (mut mpsc_sender, mut mpsc_receiver) = channel::<Bytes>(64);

    let peer_sender: mpsc::Sender<Bytes> = match inf.pending_calls.remove(&room_id) {
        Some((_, senders)) => {
            senders.peer_sender.send(mpsc_sender);
            senders.caller_sender
        }
        None => {
            let (oneshot_sender, oneshot_receiver) = oneshot::channel();
            inf.pending_calls.insert(
                room_id,
                Senders {
                    caller_sender: mpsc_sender,
                    peer_sender: oneshot_sender,
                },
            );

            match oneshot_receiver.await {
                Ok(peer_sender) => peer_sender,
                Err(_) => return,
            }
        }
    };

    let send_task = tokio::spawn(async move {});
    let recv_task = tokio::spawn(async move {});
}
