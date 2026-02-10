use crate::handlers;
use axum::{Router, routing::get, routing::post};
use sqlx::{Pool, Postgres};
use tower_http::cors::{Any, CorsLayer};

pub fn create_router(pool: Pool<Postgres>) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_headers(Any);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/thoughts", get(handlers::get_thoughts))
        .route("/thought-submission", post(handlers::submit_thought))
        .layer(cors)
        .with_state(pool)
}
