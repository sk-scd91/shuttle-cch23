use axum::{
    extract::Path,
    http::StatusCode,
    routing::get,
    Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
struct PokemonStat {
    weight: u64 // in hectograms.
}

async fn get_pokemon_stats(poke_id: u64) -> reqwest::Result<PokemonStat> {
    let poke_url = format!("https://pokeapi.co/api/v2/pokemon/{}/", poke_id);
    reqwest::get(poke_url)
        .await?
        .json::<PokemonStat>()
        .await
}

// Display Pokemon weight, in kilograms.
async fn weight(Path(poke_id): Path<u64>) -> Result<String, (StatusCode, String)> {
    get_pokemon_stats(poke_id).await
        .map(|poke_stat| (poke_stat.weight / 10).to_string())
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))
}

pub fn pokemon_router() -> Router {
    Router::new().route("/weight/:poke_id", get(weight))
}