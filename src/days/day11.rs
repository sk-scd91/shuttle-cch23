use axum::{
    //routing::get,
    Router
};

use tower_http::services::ServeFile;

pub fn ornament_router() -> Router {
    Router::new().nest_service("/assets/decoration.png", ServeFile::new("assets/decoration.png"))
}