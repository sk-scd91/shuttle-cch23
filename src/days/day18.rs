use axum::{
    extract::{ Json, Path, State },
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

#[derive(Deserialize, FromRow, Serialize)]
struct TopGiftsByRegion {
    region: String,
    top_gifts: Vec<String>,
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

    // Return Ok status code when empty.
    if regions.is_empty() {
        return StatusCode::OK;
    }

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
            ORDER BY rs.name;"
    )
        .fetch_all(&order_db.pool)
        .await
        .map_err(
            |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e))
        )?;
    Ok(Json(result))
}

async fn get_top_gifts_per_region(
    State(order_db): State<OrderDb>,
    Path(limit): Path<i64>
) -> Result<Json<Vec<TopGiftsByRegion>>, (StatusCode, String)> {
    let results: Vec<TopGiftsByRegion> = sqlx::query_as(
        r"WITH gifts AS
                (SELECT os.region_id, os.gift_name, SUM(os.quantity) AS sum
                FROM orders os
                GROUP BY os.region_id, os.gift_name
                ORDER BY SUM(os.quantity) DESC, os.gift_name ASC)
            SELECT
                rs.name AS region,
                COALESCE(
                    ARRAY_AGG(ga.gift_name ORDER BY ga.sum DESC, ga.gift_name ASC)
                        FILTER(WHERE ga.gift_name IS NOT NULL),
                    '{}') AS top_gifts
            FROM regions rs
            LEFT JOIN LATERAL (SELECT g.region_id, g.gift_name, g.sum
                FROM gifts g
                WHERE rs.id = g.region_id
                LIMIT $1 ) ga ON rs.id = ga.region_id
            GROUP BY rs.name
            ORDER BY rs.name;"
    )
        .bind(limit)
        .fetch_all(&order_db.pool)
        .await
        .map_err(
            |e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB Error: {}", e))
        )?;
    Ok(Json(results))
}

pub fn gift_order_router2(pg_pool: PgPool) -> Router {
    let order_db = OrderDb { pool: pg_pool };
    Router::new()
        .route("/reset", post(reset_order_table))
        .route("/orders", post(super::day13::insert_order))
        .route("/regions", post(insert_region))
        .route("/regions/total", get(get_total_orders_by_region))
        .route("/regions/top_list/:limit", get(get_top_gifts_per_region))
        .with_state(order_db)
}