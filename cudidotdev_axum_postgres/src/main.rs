use dotenvy::dotenv;
use std::{env, error::Error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    println!("DATABASE_URL: {}", database_url);

    // Create database connections pool
    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Run existing database migrations
    sqlx::migrate!("./migrations").run(&pool)
        .await
        .expect("Failed to run migrations...");

    Ok(())
}