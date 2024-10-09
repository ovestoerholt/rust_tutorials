# Build a CRUD REST API with Rust Axum | Tutorial
Source: https://www.youtube.com/watch?v=NJsTgmayHZY


## Setup your environment

### Configure Postgres environment

#### Add docker environment

Add a `docker-compose.yml` file to the root of your project with the following content:

```yml
services:
  db:
    image: postgres:17.0-bookworm
    restart: always
    environment:
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
    ports:
      - "5432:5432"
    volumes:
      - db:/var/lib/postgresql/data
volumes:
  db:
    driver: local
```

Then start the docker environment (the database):

```sh
docker compose up
```

#### Create Postgres database and database owner (role)

Connect to the database server either using the docker console `psql`or by connecting from remote using fx. [Azure Data Studio](https://azure.microsoft.com/en-us/products/data-studio).

Run the following queries to create user and database:

```SQL
CREATE ROLE axum_postgres WITH LOGIN PASSWORD 'axum_postgres';
```

```SQL
CREATE DATABASE axum_postgres WITH OWNER = 'axum_postgres';
```

### Environment variables

#### Add the `dotenv` crate to your project

Add this to the `[dependencies]` section of `cargo.toml`

```toml
[dependencies]
dotenvy = "0.15.7"
```

or; add by command

```sh
cargo add dotenvy
```

#### Modify your `main.rs` file

```rust
use dotenvy::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
    println!("DATABASE_URL: {}", database_url);
}
```

#### Add a `.env` file to the root of your project

```Text
DATABASE_URL=postgres://axum_postgres:axum_postgres@localhost/axum_postgres
```

#### Run your program

```sh
cargo run
```

You should see the following output:

```Text
DATABASE_URL: postgres://axum_postgres:axum_postgres@localhost/axum_postgres
```

### Add more dependencies (Tokio, Axum, SQLx)

Add the following to `cargo.toml` dependencies section:

#### Tokio

Edit manually:

```toml
[dependencies]
tokio = { version = "1.40.0", features = ["full"] }
```

Or use the Cargo CLI:

```sh
cargo add tokio --features full
```

The `full` feature includes all the nuts and bolts for doing async I/O in Rust.

#### SQLX

Edit manually:

```toml
[dependencies]
sqlx = { version = "0.8.2", features = ["postgres", "runtime-tokio", "tls-native-tls", "macros"] }
```

Or use the Cargo CLI:

```sh
cargo add sqlx --features postgres,runtime-tokio,tls-native-tls,macros
```

The features listed will add support for the following:
- postgres: Add support for the Postgres database server
- runtime-tokio: Use the tokio runtime without enabling a TLS backend
- tls-native-tls: Use the native-tls TLS backend (OpenSSL on *nix, SChannel on Windows, Secure Transport on macOS)
- macros: Add support for the query*! macros, which allows compile-time checked queries


#### Axum

Edit manually:

```toml
[dependencies]
axum = "0.7.7"
```

Or use the Cargo CLI:

```sh
cargo add axum
```

#### Serde

Serde is a framework for serializing and deserializing Rust data structures efficiently and generically.

Edit manually:

```toml
[dependencies]
serde = { version = "1.0.210", features = ["derive"] }
```

Or use the Cargo CLI:

```sh
cargo add serde --features derive
```

The `derive` feature will add support for serialization/deserialization annotations to structs.


#### serde_json

Edit manually:

```toml
[dependencies]
serde_json = "1.0.128"
```

Or use the Cargo CLI:

```sh
cargo add serde_json
```





## Add `tasks` table (first SQLx migration)

### Install `sqlx-cli`

The SQLx CLI tool helps you operate database in context of your project.

To install:

```sh
cargo install sqlx-cli
```

After the command is available using the `sqlx` command from a terminal.

The sqlx CLI will read the .env file in your project to determine the database connection settings. By default, it looks for the DATABASE_URL environment variable in your .env file, which should contain the connection string for your database. If the DATABASE_URL is properly set, sqlx will use it to connect to your database.


We will add all database tables etc. using `SQLx migrations`.

### Create the `migrations` folder

With a terminal opened in the project root, use the following command:

```sh
mkdir migrations
```

### Create the migration files

Inside the `migrations`folder create a file called `0001_add_tasks_table.sql`

Edit the newly created file by adding the following SQL statement:

```sql
CREATE TABLE tasks (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    priority INT
);
```

For every `migration` there should be `down-migration`. 

Inside the `migrations`folder create a file called `0001_add_tasks_table.down.sql`

Edit the newly created file by adding the following SQL statement:

```sql
DROP TABLE tasks;
```

You can now run the `migrations` using the `sqlx` CLI.

Run all available migrations (from current level and up):
```sh
sqlx migrate run
```

And to revert (one by one):
```sh
sqlx migrate revert
```

Observe changes to the tables fx. in `Azure Data Studio`


### Run migrations when running your program

#### Add buildscript

```sh
sqlx migrate build-script
```

This command adds a file `build.rs` to your project folder. 

The build-script is for running migrations when you have only modified SQL files and not Rust code.


#### Add running migrations code to the `main.rs` file

```Rust
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
}```

If either connecting to the database or running the migrations fails the program will abort early. You can test this by fx. rename the `DATABASE_URL` variable in your code to something bogus. 

Running the program will then (for example) exit (panic) with the following error message:

```Text
Failed to connect to the database: Configuration(RelativeUrlWithoutBase)
```



## Serve up your Axum app (Hello world style)

To your `.env`file add information about server host address.

```Text
SERVER_ADDRESS=0.0.0.0:8080
```

Modify your main.rs so it looks like this:

```Rust
use axum::{routing::get, Router};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;
use std::env;

#[tokio::main]
async fn main() -> () {
    dotenv().expect("Unable to access .env file");
    // Read server address from .env/environment. Fallback to 127.0.0.1:3000
    let server_address =
        env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_owned());
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
    let listener = TcpListener::bind(server_address).await.expect("Could not create a TCP listener");

    println!("Listening on {}", listener.local_addr().unwrap());

    let app = Router::new().route("/", get(|| async {"Hello World!"}));

    axum::serve(listener, app).await.expect("Error serving application");
}
```

Now run your program. The console should show:

```Text
DATABASE_URL: postgres://axum_postgres:axum_postgres@localhost/axum_postgres
Listening on 0.0.0.0:8080
```

Now use `Postman` or a similar tool for testing the server trying a `GET` request against `0.0.0.0:8080` (or whatever IP and port your server listens to)



## Add `tasks`routes and route handlers

Add tasks routes and handlers to your `main.rs`.

```Rust

    // ... above code is unchanged ...

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
```

After making the edits, run your program and your should be able to create, get, update and delete tasks.