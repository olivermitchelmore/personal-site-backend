use axum::http::{HeaderMap, StatusCode};
use axum::{Form, Json, extract::ConnectInfo, extract::State, response::Redirect};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::net::SocketAddr;

#[derive(Deserialize, sqlx::FromRow)]
pub struct ThoughtJson {
    x: i32,
    y: i32,
    thought: String,
}

#[derive(Serialize)]
pub struct Thought {
    x: i32,
    y: i32,
    #[serde(rename = "z")]
    id: i32,
    thought: String,
}

#[derive(Deserialize, sqlx::FromRow)]
pub struct ContactSubmission {
    email: String,
    name: String,
    message: String,
}

pub async fn get_thoughts(
    State(pool): State<PgPool>,
) -> Result<Json<Vec<Thought>>, (StatusCode, String)> {
    let thoughts = sqlx::query_as!(Thought, "SELECT x, y, id, thought FROM positions")
        .fetch_all(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(thoughts))
}

pub async fn submit_thought(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(submission): Json<ThoughtJson>,
) -> Result<StatusCode, (StatusCode, String)> {
    let x = submission.x.clamp(0, 1000);
    let y = submission.y.clamp(0, 1000);

    let user_ip = headers
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| addr.ip().to_string());

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

    if thought.chars().count() > 50 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Thought must be below 50 characters".to_string(),
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
        println!("Database Error: {}", e);
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

pub async fn contact_submission(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Form(submission): Form<ContactSubmission>,
) -> Result<Redirect, (StatusCode, String)> {
    let user_ip = headers
        .get("cf-connecting-ip")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| addr.ip().to_string());

    // Prevent XSS for future admin panel
    let email = ammonia::clean(&submission.email);
    let name = ammonia::clean(&submission.name);
    let message = ammonia::clean(&submission.message);

    let count = sqlx::query!(
        "SELECT COUNT(*) as count
        FROM contact
        WHERE user_ip = $1
        AND submitted_at > NOW() - INTERVAL '24 hours'",
        user_ip
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let contact_count = count.count.unwrap_or(0);

    // todo: redirect to proper error pages
    if contact_count >= 15 {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            "You can only post 15 contact forms every 24 hours".to_string(),
        ));
    }

    if message.chars().count() > 20000 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Contact message must be below 20000 characters".to_string(),
        ));
    }

    sqlx::query!(
        "INSERT INTO contact (email, name, message, user_ip) VALUES ($1, $2, $3, $4)",
        email,
        name,
        message,
        user_ip,
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
    Ok(Redirect::to("https://olliemitchelmore.com/thank-you"))
}
