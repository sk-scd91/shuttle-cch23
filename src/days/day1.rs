use axum::{
    extract::Path,
    http::status::StatusCode,
    routing::get,
    Router
};
use std::{
    ops::BitXor,
    str::FromStr
};

async fn xor_cube(Path(nums): Path<String>) -> Result<String, StatusCode> {
    let nums: Vec<Result<i64, std::num::ParseIntError>> = nums.split('/')
        .map(i64::from_str)
        .collect();
    // Require 1-20 numbers
    if nums.is_empty() || nums.len() > 20 || nums.iter().any(Result::is_err) {
        return Result::Err(StatusCode::BAD_REQUEST)
    }

    let xor = nums.into_iter()
        .map(Result::unwrap)
        .fold(0i64, BitXor::bitxor);
    let r = xor.checked_pow(3)
        .map(|x| x.to_string())
        .ok_or(StatusCode::UNPROCESSABLE_ENTITY);
    r
}

pub fn xor_cube_router() -> Router {
    Router::new().route("/*num", get(xor_cube))
}