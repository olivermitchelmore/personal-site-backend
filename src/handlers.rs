use axum::http::StatusCode;
use axum::{Json, extract::ConnectInfo, extract::State};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(Deserialize, Serialize, sqlx::FromRow)]
pub struct Thought {
    x: i32,
    y: i32,
    thought: String,
}

pub async fn get_thoughts(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Thought>>, (StatusCode, String)> {
    let thoughts = sqlx::query_as!(Thought, "SELECT x, y, thought FROM positions")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(thoughts))
}

pub async fn submit_thought(
    State(pool): State<PgPool>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(submission): Json<Thought>,
) -> Result<StatusCode, (StatusCode, String)> {
    let x = submission.x.clamp(0, 700);
    let y = submission.y.clamp(0, 880);

    let user_ip = addr.ip().to_string();

    let thought = ammonia::clean(&submission.thought);

    let count = sqlx::query!(
        "SELECT COUNT(*) as count
        FROM positions
        WHERE user_ip = $1
        AND submitted_at > NOW() - INTERVAL '24 hours'",
        user_ip
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let thought_count = count.count.unwrap_or(0);
    if thought_count >= 3 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "You can only submit 3 thoughts in 24 hours".to_string(),
        ));
    }

    println!("chars = {}", thought.chars().count());
    if thought.chars().count() > 99 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Thought must be below 100 characters".to_string(),
        ));
    }

    sqlx::query!(
        "INSERT INTO positions (x, y, thought, user_ip) VALUES ($1, $2, $3, $4)",
        x,
        y,
        thought,
        user_ip
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        eprintln!("Database Error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    })?;

    Ok(StatusCode::CREATED)
}

pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
