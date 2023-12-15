use axum::{
    extract::Json,
    http::StatusCode,
    routing::post,
    Router,
};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct PasswordInput {
    input: String,
}

#[derive(Serialize)]
struct PasswordOutput {
    result: String,
}

async fn match_nice_password(
    Json(PasswordInput { input }): Json<PasswordInput>,
) -> (StatusCode, Json<PasswordOutput>) {
    let three_vowels_regex = Regex::new("^.*?(?:[aeiouy].*?){3}.*$").unwrap();
    let forbidden_letters_regex = Regex::new("ab|cd|pq|xy").unwrap();

    // The regex crate does not support \g, so we match two ascii letters and compare them as a workaround.
    let has_double = input.as_bytes().windows(2)
        .filter(|w| (w[0] as char).is_ascii_alphabetic())
        .any(|w| w[0] == w[1]);

    if three_vowels_regex.is_match(&input)
        && has_double 
        && !forbidden_letters_regex.is_match(&input) {
            (StatusCode::OK, Json(PasswordOutput { result: "nice".into() }))
        } else {
            (StatusCode::BAD_REQUEST, Json(PasswordOutput { result: "naughty".into() }))
        }
}

pub fn nice_password_router() -> Router {
    Router::new().route("/nice", post(match_nice_password))
}