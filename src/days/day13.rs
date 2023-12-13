use axum::{
    extract::{ /* Json, Path, */ State },
    http::StatusCode,
    routing::{ get, /* post */ },
    Router,
};
use sqlx::PgPool;

#[derive(Clone)]
struct OrderDb {
    pool: PgPool,
}

async fn test_sql(State(order_db): State<OrderDb>) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("SELECT 20231213")
        .fetch_one(&order_db.pool)
        .await
        .map(|x| i32::to_string(&x))
        .map_err(
            |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e))
        )
}

pub fn gift_order_router(pg_pool: PgPool) -> Router {
    let order_db = OrderDb { pool: pg_pool };
    Router::new()
        .route("/sql", get(test_sql))
        .with_state(order_db)
}