use std::collections::HashMap;
use axum::{
    extract::Json,
    http::{
        header::HeaderMap,
        StatusCode
    },
    routing::get,
    Router,
};
use axum_extra::extract::cookie::Cookie;
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{ Serialize, Deserialize };
use serde_json::Value;


#[derive(Deserialize)]
struct RecipeInput {
    recipe: HashMap<String, u64>,
    pantry: HashMap<String, u64>
}

#[derive(Serialize)]
struct RecipeOutput {
    cookies: u64,
    pantry: HashMap<String, u64>
}

fn decode_recipe_cookie(headers: &HeaderMap) -> Result<Vec<u8>, (StatusCode, String)> {
    let cookie_val = headers.get_all("cookie")
        .iter()
        .filter_map(|v| match Cookie::parse(v.to_str().unwrap()) {
            Ok(cookie) => if cookie.name() == "recipe" { Some(Ok(cookie)) } else { None },
            Err(e) => Some(Err(e))
        }).next().ok_or((StatusCode::BAD_REQUEST, "Could not find cookies in header".into()))?;
    
    let cookie = cookie_val.map_err(|e| (StatusCode::BAD_REQUEST, format!("Unable to parse cookie: {}", e)))?;

    BASE64_STANDARD.decode(cookie.value())
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Unable to decode string: {}", e)))
}

async fn decode(headers: HeaderMap) -> Result<Json<Value>, (StatusCode, String)> {
    let encoded_cookie = decode_recipe_cookie(&headers)?;
    let decoded_recipe = serde_json::from_slice(&encoded_cookie)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Unable to convert to json: {}", e)))?;
    Ok(Json(decoded_recipe))
}

async fn bake(headers: HeaderMap) -> Result<Json<RecipeOutput>, (StatusCode, String)> {
    let encoded_cookie = decode_recipe_cookie(&headers)?;
    let decoded_input: RecipeInput = serde_json::from_slice(&encoded_cookie)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Unable to convert to json: {}", e)))?;

    let cookies = decoded_input.recipe.iter()
        .filter(|(_, &count)| count > 0)
        .filter_map(|(ingredient, &count)| match decoded_input.pantry.get(ingredient) {
            Some(p) => p.checked_div(count), // Ignore if ingredient has 0 count.
            None => Some(0) // Can't make cookies without ingredient
        }).min().ok_or((StatusCode::BAD_REQUEST, "Could not find all ingredients in pantry".into()))?;

    let mut pantry = decoded_input.pantry.clone();
    if cookies > 0 {
        for (ingredient, &count) in decoded_input.recipe.iter() {
            if let Some(pantry_entry) = pantry.get_mut(ingredient) {
                *pantry_entry -= cookies * count;
            }
        }
    }

    Ok(Json(RecipeOutput { cookies, pantry }))
}

pub fn cookie_router() -> Router {
    Router::new().route("/decode", get(decode))
        .route("/bake", get(bake))
}