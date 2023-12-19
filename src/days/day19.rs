use axum::{
    extract::{
        WebSocketUpgrade,
        ws::{Message, WebSocket}
    },
    response::Response,
    routing::get,
    Router,
};

async fn ws_upgrade_ping(
    ws: WebSocketUpgrade
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

pub fn ws_games_router() -> Router {
    Router::new().route("/ws/ping", get(ws_upgrade_ping))
}