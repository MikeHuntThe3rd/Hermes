use std::time::Duration;

use axum::{
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use uuid::Uuid;

use tokio::{
    sync::{mpsc, oneshot},
    time::timeout,
};

use crate::types::*;
use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc::channel;

use crate::types::AppState;

pub async fn ws_upgrade(ws: WebSocketUpgrade, State(inf): State<AppState>) -> Response {
    return ws.on_upgrade(|socket| test_handler(socket, State(inf), Uuid::new_v4()));
}

async fn test_handler(mut socket: WebSocket, State(inf): State<AppState>, call_id: Uuid) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    let (mpsc_sender, mut mpsc_receiver) = channel::<Bytes>(64);

    let peer_sender: mpsc::Sender<Bytes> = match inf.pending_calls.remove(&call_id) {
        Some((_, senders)) => {
            senders.caller.send(mpsc_sender);
            senders.peer
        }
        None => {
            let (oneshot_sender, oneshot_receiver) = oneshot::channel();
            inf.pending_calls.insert(
                call_id,
                Senders {
                    peer: mpsc_sender,
                    caller: oneshot_sender,
                },
            );

            match timeout(Duration::from_secs(60), oneshot_receiver).await {
                Ok(Ok(peer_sender)) => peer_sender,
                _ => {
                    inf.pending_calls.remove(&call_id);
                    return;
                }
            }
        }
    };

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(chunk)) = ws_receiver.next().await {
            if let Message::Binary(chunk_bytes) = chunk {
                if peer_sender.send(chunk_bytes).await.is_err() {
                    break;
                }
            }
        }
    });
    let mut send_task = tokio::spawn(async move {
        while let Some(chunk) = mpsc_receiver.recv().await {
            ws_sender.send(Message::Binary(chunk)).await;
        }
    });

    tokio::select! {
        _ = &mut recv_task => recv_task.abort(),
        _ = &mut send_task => send_task.abort(),
    }
}
