mod handlers;
mod routes;
use anyhow::{Context, Result};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::env;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").context("Cannot find db url")?;
    let frontend_domain = env::var("FRONTEND_DOMAIN").context("Cannot find frontend domain")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .context("Failed to create db connection")?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    tokio::spawn(cleanup(pool.clone()));

    let router = routes::create_router(frontend_domain, pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("server error")?;
    Ok(())
}

async fn cleanup(pool: Pool<Postgres>) {
    loop {
        sleep(Duration::from_secs(24 * 60 * 60)).await;
        println!("Cleaning up old thoughts...");

        let result =
            sqlx::query("DELETE FROM positions WHERE submitted_at < NOW() - INTERVAL '7 days'")
                .execute(&pool)
                .await;

        match result {
            Ok(result) => println!(
                "Successfully cleaned up old thoughts. Rows deleted: {}",
                result.rows_affected()
            ),
            Err(e) => eprintln!("Cleanup error: {}", e),
        };
    }
}
