use axum::{
    extract::{ Json, State },
    http::StatusCode,
    routing::{ get, post },
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, PgPool, QueryBuilder};

#[derive(Clone)]
pub(super) struct OrderDb {
    pub(super) pool: PgPool,
}

#[derive(FromRow, Serialize, Deserialize)]
pub(super) struct Order {
    pub(super) id: i32,
    pub(super) region_id: i32,
    pub(super) gift_name: String,
    pub(super) quantity: i32,
}

#[derive(Serialize)]
struct Total {
    total: i64,
}

#[derive(Serialize)]
struct Popular {
    popular: Option<String>,
}

const RESET_QUERY: &'static str = r"
    DROP TABLE IF EXISTS orders;
    CREATE TABLE orders (
        id INT PRIMARY KEY,
        region_id INT,
        gift_name VARCHAR(50),
        quantity INT
    );
";

const MOST_POPULAR_QUERY: &'static str = r"
    SELECT gift_name
    FROM (SELECT gift_name, SUM(quantity) AS total
        FROM orders
        GROUP BY gift_name)
    ORDER BY total DESC
    LIMIT 1;
";

async fn test_sql(State(order_db): State<OrderDb>) -> Result<String, (StatusCode, String)> {
    sqlx::query_scalar("SELECT 20231213")
        .fetch_one(&order_db.pool)
        .await
        .map(|x| i32::to_string(&x))
        .map_err(
            |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e))
        )
}

async fn reset_order_table(State(order_db): State<OrderDb>) -> StatusCode {
    // Use pool directly to execute multiple statements.
    order_db.pool.execute(RESET_QUERY)
        .await
        .and(Ok(StatusCode::OK))
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

pub(super) async fn insert_order(
    State(order_db): State<OrderDb>,
    Json(orders): Json<Vec<Order>>,
 ) -> StatusCode {

    // Skip if orders is empty.
    if orders.is_empty() {
        return StatusCode::OK;
    }

    // Use a QueryBuilder to add multiple tuple values.
    let mut builder: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new("INSERT INTO orders (id, region_id, gift_name, quantity) ");
    builder.push_values(&orders, |mut row, order| {
        row.push_bind(order.id)
            .push_bind(order.region_id)
            .push_bind(&order.gift_name)
            .push_bind(order.quantity);
    });
    let result = builder.build()
        .execute(&order_db.pool)
        .await;
    result.and(Ok(StatusCode::OK))
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_total_orders(
    State(order_db): State<OrderDb>,
) -> Result<Json<Total>, (StatusCode, String)> {
    let result: i64 = sqlx::query_scalar("SELECT COALESCE(SUM(quantity), 0) FROM orders;")
        .fetch_one(&order_db.pool)
        .await
        .map_err(
            |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e))
        )?;
    Ok(Json(Total { total: result }))
}

async fn get_popular_gift(
    State(order_db): State<OrderDb>,
) -> Result<Json<Popular>, (StatusCode, String)> {
    let result: Option<String> = sqlx::query_scalar(MOST_POPULAR_QUERY)
        .fetch_optional(&order_db.pool)
        .await
        .map_err(
            |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e))
        )?;
    Ok(Json(Popular { popular: result }))
}

pub fn gift_order_router(pg_pool: PgPool) -> Router {
    let order_db = OrderDb { pool: pg_pool };
    Router::new()
        .route("/sql", get(test_sql))
        .route("/reset", post(reset_order_table))
        .route("/orders", post(insert_order))
        .route("/orders/total", get(get_total_orders))
        .route("/orders/popular", get(get_popular_gift))
        .with_state(order_db)
}