use axum::{
    http::status::StatusCode,
    routing::get,
    Router
};
use sqlx::PgPool;

mod days;
use days::*;

async fn hello_world() -> &'static str {
    "Hello, Santa!"
}

async fn internal_service_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(local_uri="postgres://{secrets.USERSPEC}@localhost:5432/cch23")]
    pool: PgPool
) -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world))
        .route("/-1/error", get(internal_service_error))
        .nest("/1", day1::xor_cube_router())
        .nest("/4", day4::serdeer_router())
        .nest("/6", day6::elf_router())
        .nest("/7", day7::cookie_router())
        .nest("/8", day8::pokemon_router())
        .nest("/11", day11::ornament_router())
        .nest("/12", day12::timekeeper_router())
        .nest("/13", day13::gift_order_router(pool.clone()))
        .nest("/14", day14::html_reindeer_route())
        .nest("/15", day15::nice_password_router());

    Ok(router.into())
}
