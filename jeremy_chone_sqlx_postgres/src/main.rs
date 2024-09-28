use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, Row};

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    // 1) Create a connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/postgres")
        .await?;

    // 2a) Create a database if not exist
    // Check if the database exists
    let db_exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = 'sqlx_postgres_ticket')",
    )
    .fetch_one(&pool)
    .await?;

    // If it does not exist, create it
    if !db_exists.0 {
        sqlx::query("CREATE DATABASE dbname").execute(&pool).await?;
        println!("Database created!");
    } else {
        println!("Database already exists.");
    }

    // 2b) Connect to database
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/sqlx_postgres_ticket")
        .await?;

    // 2) Create a table if not exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ticket (
            id bigserial,
            name text
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // 3) Insert a new ticket
    sqlx::query_as("INSERT INTO ticket (name) values ($1) returning id")
        .bind("a new ticket")
        .fetch_one(&pool)
        .await?;

    // 4a) Select all tickets
    let rows = sqlx::query("SELECT * FROM ticket").fetch_all(&pool).await?;

    for row in &rows {
        let id: i64 = row.get("id");
        let name: String = row.get("name");
        println!("ID: {}, Name: {}", id, name);
    }

    // 4b) Select all tickets and concatenate the values
    let str_result = &rows
        .iter()
        .map(|r| format!("{} - {}", r.get::<i64, _>("id"), r.get::<String, _>("name")))
        .collect::<Vec<String>>()
        .join(", ");
    println!("Result: {}", str_result);

    // 5) Select query with map() (build the ticket manually)

    // 6) Select query_as (using derive FromRow)

    Ok(())
}
