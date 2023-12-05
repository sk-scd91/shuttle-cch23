use axum::{
    extract::Json,
    http::StatusCode,
    routing::post,
    Router,
};

use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
#[allow(dead_code, unused_variables)]
struct ReindeerStrength {
    name: String,
    strength: u64,
    #[serde(default)]
    speed: f64,
    #[serde(default)]
    height: u64,
    #[serde(default)]
    antler_width: u64,
    #[serde(default)]
    snow_magic_power: u64,
    #[serde(default)]
    favorite_food: String,
    #[serde(rename="cAnD13s_3ATeN-yesT3rdAy", default)]
    candies_eaten_yesterday: u64
}

#[derive(Serialize)]
struct ContestResults {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String
}

async fn strength(Json(reindeer): Json<Vec<ReindeerStrength>>) -> String {
    reindeer.iter()
        .map(|r| r.strength)
        .sum::<u64>()
        .to_string()
}

async fn contest(Json(reindeer): Json<Vec<ReindeerStrength>>)
    -> Result<Json<ContestResults>, StatusCode> {
    if reindeer.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let fastest = reindeer.iter()
        .max_by(|a, b| a.speed.partial_cmp(&b.speed)
            .unwrap_or_else(|| b.speed.is_nan().cmp(&a.speed.is_nan()))) // Propagate numeric values.
        .unwrap();
    let tallest = reindeer.iter().max_by_key(|r| r.height).unwrap();
    let magician = reindeer.iter().max_by_key(|r| r.snow_magic_power).unwrap();
    let most_candies = reindeer.iter().max_by_key(|r| r.candies_eaten_yesterday).unwrap();

    if most_candies.favorite_food.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Json(ContestResults {
        fastest: format!("Speeding past the finish line with a strength of {} is {}", fastest.speed, &fastest.name),
        tallest: format!("{} is standing tall with his {} cm wide antlers", &tallest.name, tallest.height),
        magician: format!("{} could blast you away with a snow magic power of {}", &magician.name, magician.snow_magic_power),
        consumer: format!("{} ate lots of candies, but also some {}", &most_candies.name, &most_candies.favorite_food)
    }))
}

pub fn serdeer_router() -> Router {
    Router::new().route("/strength", post(strength))
        .route("/contest", post(contest))
}