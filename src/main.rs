use axum::{routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, Santa!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_world));

    Ok(router.into())
}
