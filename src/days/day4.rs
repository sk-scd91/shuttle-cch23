use axum::{
    extract::Json,
    routing::post,
    Router
};

use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code, unused_variables)]
struct ReindeerStrength {
    name: String,
    strength: u64
}

async fn strength(Json(reindeer): Json<Vec<ReindeerStrength>>) -> String {
    reindeer.iter()
        .map(|r| r.strength)
        .sum::<u64>()
        .to_string()
}

pub fn serdeer_router() -> Router {
    Router::new().route("/strength", post(strength))
}