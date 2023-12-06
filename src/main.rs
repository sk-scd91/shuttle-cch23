use axum::{
    http::status::StatusCode,
    routing::get,
    Router
};
mod days;
use days::*;

async fn hello_world() -> &'static str {
    "Hello, Santa!"
}

async fn internal_service_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world))
        .route("/-1/error", get(internal_service_error))
        .nest("/1", day1::xor_cube_router())
        .nest("/4", days::day4::serdeer_router())
        .nest("/6", days::day6::elf_router());

    Ok(router.into())
}
