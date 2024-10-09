use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> () {
    dotenv().expect("Unable to access .env file");
    // Read server address from .env/environment. Fallback to 127.0.0.1:3000
    let server_address = env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url =
        env::var("DATABASE_URL").expect("Unable to read DATABASE_URL environment variable");
    println!("DATABASE_URL: {}", database_url);

    // Create database connections pool
    let db_pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Run existing database migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations...");

    // Create Axum TCP listener
    let listener = TcpListener::bind(server_address)
        .await
        .expect("Could not create a TCP listener");

    println!("Listening on {}", listener.local_addr().unwrap());

    let app = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:task_id", patch(update_task).delete(delete_task))
        .with_state(db_pool);

    axum::serve(listener, app)
        .await
        .expect("Error serving application");
}

#[derive(Serialize)]
struct TaskRow {
    id: i32,
    name: String,
    priority: Option<i32>,
}

async fn get_tasks(
    State(pg_pool): State<PgPool>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let rows = sqlx::query_as!(TaskRow, "SELECT * FROM tasks ORDER BY id")
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "success" : false, "message" : e.to_string() }).to_string(),
            )
        })?;

    Ok((
        StatusCode::OK,
        json!({ "success" : true, "data" : rows }).to_string(),
    ))
}

#[derive(Deserialize)]
struct CreateTaskReq {
    name: String,
    priority: Option<i32>,
}

#[derive(Serialize)]
struct CreateTaskRow {
    id: i32,
}

async fn create_task(
    State(pg_pool): State<PgPool>,
    Json(task): Json<CreateTaskReq>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let row = sqlx::query_as!(
        CreateTaskRow,
        "INSERT INTO tasks (name, priority) VALUES ($1, $2) RETURNING id",
        task.name,
        task.priority
    )
    .fetch_one(&pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success" : false, "message" : e.to_string() }).to_string(),
        )
    })?;
    Ok((
        StatusCode::CREATED,
        json!({ "success" : true, "data" : row}).to_string(),
    ))
}

#[derive(Deserialize)]
struct UpdateTaskReq {
    name: Option<String>,
    priority: Option<i32>,
}

async fn update_task(
    State(pg_pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(task): Json<UpdateTaskReq>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    sqlx::query!(
        "
        UPDATE tasks SET 
        name = $2,
        priority = $3
        WHERE id = $1
        ",
        id,
        task.name,
        task.priority
    )
    .execute(&pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success" : false, "message" : e.to_string() }).to_string(),
        )
    })?;
    Ok((StatusCode::OK, json!({ "success": true, }).to_string()))
}


async fn delete_task(
    State(pg_pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    sqlx::query!("DELETE FROM tasks WHERE id = $1", id)
    .execute(&pg_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success" : false, "message" : e.to_string() }).to_string(),
        )
    })?;
    Ok((StatusCode::OK, json!({ "success": true, }).to_string()))
}
