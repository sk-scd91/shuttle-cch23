use std::{
    collections::{
        HashMap,
        hash_map::Entry,
    },
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use axum::{
    extract::{
        Path,
        State,
        WebSocketUpgrade,
        ws::{Message, WebSocket}
    },
    response::Response,
    routing::{get, post},
    Router,
};
use futures_util::{
    sink::SinkExt,
    stream::StreamExt
};
use serde::{Deserialize, Serialize};
use tokio::sync::{
    broadcast::Sender,
    RwLock
};

#[derive(Deserialize)]
struct ChatInMessage {
    message: String,
}

#[derive(Serialize)]
struct ChatOutMessage {
    user: String,
    message: String,
}

#[derive(Default)]
struct ChatState {
    views: Arc<AtomicUsize>,
    channels: HashMap<i64, Sender<Message>>,
}

async fn ws_upgrade_ping(
    ws: WebSocketUpgrade,
) -> Response {
    tracing::info!("Starting ping websocket upgrade.");
    ws.on_upgrade(|ws| handle_ping(ws))
}

async fn handle_ping(mut ws: WebSocket) {
    tracing::info!("Starting ping session.");
    let mut ping_state = false;

    while let Some(ws_result) = ws.recv().await {
        match ws_result {
            Ok(msg) => {
                match msg {
                    Message::Text(s) if &s == "serve" => {
                        tracing::info!("Recieved message: serve");
                        ping_state = true;
                    },
                    Message::Text(s) if &s == "ping" => {
                        tracing::info!("Recieved message: ping");
                        if ping_state {
                            if let Err(e) = ws.send(Message::Text("pong".into())).await {
                                tracing::error!("Ping websocket recieved an error while sending: {e}");
                                break;
                            }
                        }
                    },
                    _ => { tracing::info!("Recieved other message"); },
                }
            },
            Err(e) => {
                tracing::error!("Ping websocket recieved an error: {e}");
                break;
            },
        }
        tracing::info!("Ending ping session.")
    }
}

async fn reset_chat(State(chat_state): State<Arc<RwLock<ChatState>>>) {
    tracing::info!("Starting '/reset'.");
    let s = chat_state.write().await;
    s.views.store(0usize, Ordering::Relaxed);
}

async fn get_chat_views(State(chat_state): State<Arc<RwLock<ChatState>>>) -> String {
    // Use a write lock to let other transactions through.
    tracing::info!("Starting '/views'.");
    let n = chat_state.read().await.views.load(Ordering::Relaxed);
    tracing::info!("Receiving {n}.");
    n.to_string()
}

async fn ws_upgrade_chat(
    State(chat_state): State<Arc<RwLock<ChatState>>>,
    Path((channel, user)): Path<(i64, String)>,
    ws: WebSocketUpgrade,
) -> Response {
    //tracing::info!("Starting chat websocket upgrade.");
    ws.on_upgrade(move |ws| handle_chat(ws, chat_state, channel, user))
}

async fn handle_chat(ws: WebSocket, chat_state: Arc<RwLock<ChatState>>, room: i64, user: String) {
    let (mut ws_sink, mut ws_stream) = ws.split();

    let mut send_ws = {
        tracing::info!("Beginning session for channel: {room} user: {user}.");

        let views = chat_state.read().await.views.clone(); // Clone the atomic counter before getting write lock.

        let mut receiver = {
            let mut s = chat_state.write().await;
            match s.channels.entry(room) {
                Entry::Vacant(v) => {
                    let (sender, receiver) = tokio::sync::broadcast::channel(128);
                    v.insert(sender);
                    receiver
                },
                Entry::Occupied(v) => {
                    let sender = v.get();
                    sender.subscribe()
                }
            }
        };

        let fut = tokio::spawn(async move {
            while let Ok(msg) = receiver.recv().await {
                if let Err(e) = ws_sink.send(msg).await {
                    tracing::info!("Disconnecting due to error: {e}");
                    return;
                } else {
                    //tracing::info!("Successfully sent message.");
                    views.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        fut
    };

    let mut recv_ws = tokio::spawn(async move {
        tracing::info!("Starting WebSocket reciever loop.");
        while let Some(msg) = ws_stream.next().await {
            tracing::info!("Recieved message.");
            match msg {
                Ok(Message::Text(msg)) => {
                    // First, parse the message.
                    let message = match serde_json::from_str(&msg) {
                        Ok(ChatInMessage { message }) => message.to_owned(),
                        Err(e) => {
                            tracing::error!("Recieved JSON error: {e}");
                            continue;
                        }
                    };

                    //Then, make sure it has the right amount of characters.
                    if message.len() > 128 {
                        tracing::error!("Message is too long.");
                        continue;
                    }

                    let chan = {
                        let s = chat_state.read().await;
                        s.channels.get(&room).unwrap().clone()
                    };
                    
                    tracing::info!("Broadcasting message.");
                    let out_message = ChatOutMessage {
                        user: user.clone(),
                        message: message.clone()
                    };
                    let out_text = Message::Text(serde_json::to_string(&out_message).unwrap());
                    let result = chan
                        .send(out_text);
                    if let Err(e) = result {
                        tracing::error!("Unable to send message: {e}");
                    }
                    
                    tracing::info!("Finished broadcasting messages");
                },
                Err(e) => {
                    tracing::error!("Recieved websocket message error: {e}");
                },
                _ => {
                    tracing::info!("Unknown message.")
                }
            }
        }
    });

    // Close the other JoinHandle after one finishes.
    tokio::select! {
        _ = (&mut send_ws) => recv_ws.abort(),
        _ = (&mut recv_ws) => send_ws.abort(),
    };
    tracing::info!("Closing websocket.");
}

pub fn ws_games_router() -> Router {
    let chat_state = Arc::new(RwLock::new(ChatState::default()));

    Router::new().route("/ws/ping", get(ws_upgrade_ping))
        .route("/reset", post(reset_chat))
        .route("/views", get(get_chat_views))
        .route("/ws/room/:channel/user/:user", get(ws_upgrade_chat))
        .with_state(chat_state)
}