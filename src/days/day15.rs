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

#[derive(Serialize)]
struct GameOutput {
    result: &'static str,
    reason: &'static str,
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

async fn nice_password_game(
    Json(PasswordInput { input }): Json<PasswordInput>,
) -> (StatusCode, Json<GameOutput>) {
    // Helper function to create output for naughty passwords.
    fn validate_nice(is_nice: bool, reason: &'static str) -> Result<(), GameOutput> {
        if !is_nice {
            return Err(GameOutput { result: "naughty", reason });
        }
        Ok(())
    }

    // Rule 1: At least 8 chars.
    if let Err(o) = validate_nice(input.len() >= 8, "8 chars") {
        return (StatusCode::BAD_REQUEST, Json(o));
    }

    // Rule 2: Contain digits, uppercase and lowercase letters.
    {
        let has_all_types = input.chars().any(|c| c.is_ascii_digit())
            && input.chars().any(|c| c.is_ascii_uppercase())
            && input.chars().any(|c| c.is_ascii_lowercase());
        if let Err(o) = validate_nice(has_all_types, "more types of chars") {
            return (StatusCode::BAD_REQUEST, Json(o));
        }
    }

    // Rule 3: At least 5 digits.
    {
        let digit_count = input.chars()
            .filter(|c| c.is_ascii_digit())
            .count();
        if let Err(o) = validate_nice(digit_count >= 5, "55555") {
            return (StatusCode::BAD_REQUEST, Json(o));
        }
    }

    // Rule 4: All digits add up to 2023.
    {
        let digits_regex = Regex::new("[0-9]+").unwrap();
        // Thankfully, find_iter is not overlapping.
        let sum = digits_regex.find_iter(&input)
            .map(|m| m.as_str().parse::<u64>().unwrap())
            .sum::<u64>();
        if let Err(o) = validate_nice(sum == 2023, "math is hard") {
            return (StatusCode::BAD_REQUEST, Json(o));
        }
    }

    // Rule 5: 'j','o','y' in no other order.
    {
        let joy_regex = Regex::new("^[^oy]*?j[^jy]*?o[^jo]*?y[^joy]*$")
            .unwrap();
        if let Err(o) = validate_nice(joy_regex.is_match(&input), "not joyful enough") {
            return (StatusCode::NOT_ACCEPTABLE, Json(o));
        }
    }

    // Rule 6: Two repeats with other letter in middle.
    {
        let is_aba = input.as_bytes().windows(3)
            .filter(
                |w| w[0].is_ascii_alphabetic() && w[1].is_ascii_alphabetic()
            ).any(|w| w[0] == w[2] && w[0] != w[1]);

        if let Err(o) = validate_nice(is_aba, "illegal: no sandwich") {
            return (StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS, Json(o));
        }
    }

    // Rule 7: At least one Unicode code point in range [U+2980, U+2BFF].
    if let Err(o) = validate_nice(
        Regex::new("[\\u2980-\\u2bff]").unwrap().is_match(&input),
        "outranged"
    ) {
        return (StatusCode::RANGE_NOT_SATISFIABLE, Json(o));
    }

    // Rule 8: At least one Emoji.
    if let Err(o) = validate_nice(
        Regex::new("[\\p{Emoji_Presentation}]").unwrap().is_match(&input),
        "😳"
    ) {
        return (StatusCode::UPGRADE_REQUIRED, Json(o));
    }

    //Rule 9: SHA256 ends in 'a'.
    if let Err(o) = validate_nice(
        sha256::digest(&input).ends_with('a'),
        "not a coffee brewer"
    ) {
        return (StatusCode::IM_A_TEAPOT, Json(o))
    }

    (StatusCode::OK, Json(GameOutput { result: "nice", reason: "that's a nice password" }))
}

pub fn nice_password_router() -> Router {
    Router::new().route("/nice", post(match_nice_password))
        .route("/game", post(nice_password_game))
}