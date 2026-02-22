use crate::handlers;
use axum::{Router, http::header, routing::{get, post}};
use sqlx::{Pool, Postgres};
use tower_http::cors::{CorsLayer, AllowOrigin};
use axum::http::Method;

pub fn create_router(frontend_domain: String, pool: Pool<Postgres>) -> Router {

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(frontend_domain.parse().expect("failed to parse frontend domain")))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE])
        .max_age(std::time::Duration::from_secs(86400));


    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/thoughts", get(handlers::get_thoughts))
        .route("/thought-submission", post(handlers::submit_thought))
        .route("/contact-submission", post(handlers::contact_submission))
        .layer(cors)
        .with_state(pool)
}
