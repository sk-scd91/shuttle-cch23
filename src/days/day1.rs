use axum::{
    extract::Path,
    http::status::StatusCode,
    routing::get,
    Router
};
use std::str::FromStr;

async fn xor_cube(Path((a, b)): Path<(String, String)>) -> Result<String, StatusCode> {
    let a = i64::from_str(&a).or(Result::Err(StatusCode::BAD_REQUEST))?;
    let b = i64::from_str(&b).or(Result::Err(StatusCode::BAD_REQUEST))?;
    let xor = a ^ b;
    let r = xor.checked_pow(3)
        .map(|x| x.to_string())
        .ok_or(StatusCode::BAD_REQUEST);
    r
}

pub fn xor_cube_router() -> Router {
    Router::new().route("/:a/:b", get(xor_cube))
}