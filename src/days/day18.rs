use axum::{
    extract::{ Json, State },
    http::StatusCode,
    routing::{ get, post },
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Executor, FromRow, PgPool, QueryBuilder};

use super::day13::OrderDb;

#[derive(Deserialize, FromRow, Serialize)]
struct Region {
    id: i32,
    name: String
}

#[derive(Deserialize, FromRow, Serialize)]
struct TotalByRegion {
    region: String,
    total: i64,
}

const RESET_QUERY: &'static str = r"
    DROP TABLE IF EXISTS regions;
    DROP TABLE IF EXISTS orders;

    CREATE TABLE regions (
        id INT PRIMARY KEY,
        name VARCHAR(50)
    );

    CREATE TABLE orders (
        id INT PRIMARY KEY,
        region_id INT,
        gift_name VARCHAR(50),
        quantity INT
    );
";

async fn reset_order_table(State(order_db): State<OrderDb>) -> StatusCode {
    // Use pool directly to execute multiple statements.
    order_db.pool.execute(RESET_QUERY)
        .await
        .and(Ok(StatusCode::OK))
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn insert_region(
    State(order_db): State<OrderDb>,
    Json(regions): Json<Vec<Region>>,
 ) -> StatusCode {
    // Use a QueryBuilder to add multiple tuple values.
    let mut builder: QueryBuilder<sqlx::Postgres> =
        QueryBuilder::new("INSERT INTO regions (id, name) ");
    builder.push_values(&regions, |mut row, region| {
        row.push_bind(region.id)
            .push_bind(&region.name);
    });
    let result = builder.build()
        .execute(&order_db.pool)
        .await;
    result.and(Ok(StatusCode::OK))
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

async fn get_total_orders_by_region(
    State(order_db): State<OrderDb>,
) -> Result<Json<Vec<TotalByRegion>>, (StatusCode, String)> {
    let result: Vec<TotalByRegion> = sqlx::query_as(
        r"SELECT rs.name AS region, SUM(os.quantity) AS total
            FROM orders os
            JOIN regions as rs ON os.region_id = rs.id
            GROUP BY rs.id
            HAVING SUM(os.quantity) > 0
            ORDER BY rs.name;"
    )
        .fetch_all(&order_db.pool)
        .await
        .map_err(
            |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e))
        )?;
    Ok(Json(result))
}

pub fn gift_order_router2(pg_pool: PgPool) -> Router {
    let order_db = OrderDb { pool: pg_pool };
    Router::new()
        .route("/reset", post(reset_order_table))
        .route("/orders", post(super::day13::insert_order))
        .route("/regions", post(insert_region))
        .route("/regions/total", get(get_total_orders_by_region))
        .with_state(order_db)
}