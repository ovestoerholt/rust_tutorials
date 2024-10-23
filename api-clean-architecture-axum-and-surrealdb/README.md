# Creating an Api with Rust, Clean Architecture, CQRS, Axum and SurrealDB

Source:
- https://blog.devgenius.io/creating-an-api-with-rust-clean-architecture-axum-and-surrealdb-2a95b1b72e0f


## Initial project setup

### Add dependencies

```sh
cargo add axum
```

```sh
cargo add chrono --features serde
```

```sh
cargo add serde --features derive
```

```sh
cargo add serde_json
```

```sh
cargo add tokio --features full
```

```sh
cargo add tower-http --features cors
```

You should end up with a `cargo.toml` dependencies section looking something like this:

```toml
[dependencies]
axum = "0.7.7"
chrono = { version = "0.4.38", features = ["serde"] }
serde = { version = "1.0.210", features = ["derive"] }
serde_json = "1.0.132"
tokio = { version = "1.40.0", features = ["full"] }
tower-http = { version = "0.6.1", features = ["cors"] }
```

### Modify main.rs

Modify your main function so that it look like this:

```rust
use axum::http::{HeaderValue, Method};
use axum::http::header::{AUTHORIZATION, ACCEPT, CONTENT_TYPE};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router().layer(cors);

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```


## Set up a clean architecture project structure

In the same folder as your `main.rs`, generate a `lib.rs` file with the following content:

```rust
pub mod api;
pub mod domain;
pub mod application;
pub mod infrastructure;
```

Then create the folder structure. In the folder where main.rs and lib.rs files are present, create the following folders:
- api (route, routes, handlers)
- domain (models, entities)
- application (logic for commands and queries)
- infrastructure (data layer, repositories)

Inside each of these folders create a file `mod.rs`.


## The router and first route

Add the following content to the `api/mod.rs` file:

```rust
use axum::{Json, response::IntoResponse};

pub mod router;

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Working fine, thanks!";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}
```

In the `api` folder create the `router` module as a file `router.rs` with the following content:

```rust
use axum::{
    routing::get,
    Router,
};

use super::health_checker_handler;

pub fn create_router() -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
}
```

After open your `main.rs` and import your `create_router` function.

Now run your program and test connecting to `http://localhost:3000/api/healthchecker`.


## The Domain Model

Navigate to `domain/mod.rs` and declare the models module. Create the module in `models/mod.rs`, then declare and craft our Todo model as follows:

```rust
// domain/mod.rs
pub mod models;
```

```rust
// domain/models/mod.rs
pub mod todo;
```

```rust
// domain/models/todo.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Todo {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    pub completed: Option<bool>,
    pub createdAt: Option<DateTime<Local>>,
    pub updatedAt: Option<DateTime<Local>>,
}
```


## Commands vs Queries

`CQRS`, or `Command Query Responsibility Segregation`, is a software architectural pattern that simplifies the complexity of managing data by dividing, the application into two distinct parts: `commands` and `queries`.

In simpler terms, it separates the responsibility of updating data (commands) from the responsibility of retrieving data (queries).
This segregation allows for tailored optimization of each part independently, enhancing scalability and performance.
With CQRS your can design your system to handle write operations differenctly from read operations. providing flexibility and efficiency in managing and retrieving data based on specific use cases.

### The first command

In the `application` layer module (application/mod.rs) declare modules for commands and queries.

```rust
pub mod commands;
pub mod queries;
```

Establish these modules as subfolders to the `application` folder.

Declare our first command `create_todo_command`: 

```rust
// application/commands/create_todo_command.rs
use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::Local;

use crate::domain::models::todo::Todo;

pub async fn create_todo_command(
    Json(mut body): Json<Todo>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let datetime = Local::now();

    body.id = Some("some-id".to_string());
    body.completed = Some(false);
    body.createdAt = Some(datetime);
    body.updatedAt = Some(datetime);

    let todo = body.to_owned();

    let json_response = serde_json::json!({
        "status": "success".to_string(),
        "data": todo,
    });

    Ok((StatusCode::CREATED, Json(json_response)))
}
```

In the module `commands/mod.rs` declare the new command:

```rust
// application/commands/mod.rs
pub module create_todo_command;
```

Then modify your `routes.rs` to include a route to the new command:

```rust
use axum::{
    routing::{get, post},
    Router,
};

use crate::application::commands::create_todo_command::create_todo_command;

use super::health_checker_handler;

pub fn create_router() -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/todos", post(create_todo_command))
}
```

Run your program and test adding a todo by creating a `POST` request to `http://localhost/api/todos`

Add the following body to your request:

```json
{
    "title" : "Do this",
    "content" : "Do that"
}
```

The response will be something like:

```json
{
    "data": {
        "completed": false,
        "content": "Do that",
        "createdAt": "2024-10-21T09:41:12.213102+02:00",
        "id": "some-id",
        "title": "Do this",
        "updatedAt": "2024-10-21T09:41:12.213102+02:00"
    },
    "status": "success"
}
```


## Implementing SurrealDB on the Project

### Run SurrealDB as a Docker instance

This project uses a SurrealDB in-memory database as storage.


Add a `docker-compose.yml` file with the following content to the root of your project:

```yml
services:
  surrealdb:
    env_file:
      - .env
    entrypoint: 
      - /surreal 
      - start 
      - --user
      - $DB_USER
      - --pass
      - $DB_PASSWORD
    image: surrealdb/surrealdb:latest
    ports:
      - 8000:8000
```

Also add a `.env` file with the following content:

```text
DB_USER=root
DB_PASSWORD=root
```

### Add SurrealDB dependency

```sh
cargo add surrealdb
```



## The infrastructure layer

### Infrastructure layer structure

In the `infrastructure/mod.rs` file declare the following modules:

```rust
// infrastructure/mod.rs

pub mod db_context;
pub mod repositories;
```

Create the modules as folders:
- db_context/mod.rs
- repositoroes/mod.rs


## The DB Context

### Add crate `once_cell` to project
We need to add one more depdendency before we can continue:

```sh
cargo add once_cell
```

From (crates.io)[https://crates.io/crates/once_cell]:
`once_cell` provides two new cell-like types:
- unsync::OnceCell 
- sync::OnceCell 

OnceCell might store arbitrary non-Copy types, can be assigned to at most once and provide direct access to the stored contents.


### Create db context for SurrealDB

In `db_context/mod.rs` declare module `surreal_context`.

```rust
// db_context/mod.rs
pub mod surreal_context;
```

Create module `surreal_context` as a file `surreal_context.rs` and add the following content:

```rust
use once_cell::sync::Lazy;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Result, Surreal,
};

pub static DB: Lazy<Surreal<Client>> = Lazy::new(Surreal::init);

pub async fn connect_db() -> Result<()> {
    let _ = DB.connect::<Ws>("localhost:8000").await?;
    let _ = DB
        .signin(Root {
            username: "root",
            password: "root",
        })
        .await;
    let _ = DB.use_ns("todo").use_db("todo").await?;
    Ok(())
}
```

Also call the `connect_db` function from `main.rs`:

```rust
    ...
    let app = create_router().layer(cors);

    connect_db().await.unwrap();

    println!("🚀 Server started successfully");
    ...
```

Make sure the SurrealDb Docker image is running and start your server program. If the server starts without panicing all is ok!


## The repository

In the `repositories/mod.rs` declare the module `todo_repository` 

```rust
// repositories/mod.rs

pub mod todo_repository;
```
as a file `todo_repository.rs` with the following content:

```rust
// todo_repository.rs

pub struct TodoRepository {
    table: String,
}

impl TodoRepository {
    pub fn new() -> Self {
        TodoRepository {
            table: String::from("todo"),
        }
    }
}
```


### Add methods

#### get_all

Add the following function to the `todo_repository.rs` file as part of the `impl TodoRepository`block.

```rust
pub async fn get_all(&self) -> Result<Vec<Todo>, Error> {
    let records = DB.select(&self.table).await?;
    Ok(records)
}
```

You will be asked to import (use) the following:
- crate::domain::models::todo::Todo, 
- crate::infrastructure::db_context::surreal_context::DB
- surrealdb::Error;


#### get_by_id

Add the following function to the `todo_repository.rs` file as part of the `impl TodoRepository`block.

```rust
pub async fn get_by_id(&self, id: String) -> Result<Todo, Error> {
    if let Some(record) = DB.select((&self.table, id.clone())).await? {
        return Ok(record);
    }

    let error = Error::Db(
        Thrown(
            format!("Todo with id {} not found", id)
        )
    );
    Err(error)
}
```

#### create_todo

```rust
    pub async fn create_todo(&self, content: Todo) -> Result<Option<Todo>, Error> {
        let record = DB.create(&self.table).content(content).await?;
        Ok(record)
    }
```


Also, modify your `create_todo_command.rs` to this:

```rust
use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::Local;

use crate::{domain::models::todo::Todo, infrastructure::repositories::todo_repository::TodoRepository};

pub async fn create_todo_command(
    Json(mut body): Json<Todo>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let repository = TodoRepository::new();
    
    if let Ok(todo) = repository.get_by_title(body.title.clone()).await {
        let json_response = serde_json::json!({
            "status": "error",
            "message": "Todo already exists",
            "data": todo,
        });

        return Err((StatusCode::BAD_REQUEST, Json(json_response)));
    }

    let datetime = Local::now();
    body.completed = Some(false);
    body.createdAt = Some(datetime);
    body.updatedAt = Some(datetime);

    let todo = body.to_owned();

    let todo = repository.create_todo(todo.clone()).await;

    match todo {
        Ok(_) => {
            let json_response = serde_json::json!({
                "status": "success",
                "data": todo,
            });
        
            Ok((StatusCode::CREATED, Json(json_response)))
        },
        Err(err) => {
            let json_response = serde_json::json!({
                "status": "error",
                "data": err,
            });
        
            Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(json_response)))
        },
    }
}
```


Restart the app/server and try to make a `post` request against `http://localhost:8080/api/todos` with the following content:

```json
{
    "_id" : "do_this_or_die_in_the_attempt",
    "title" : "Do this, or die in the attempt",
    "content" : "Do that",
    "completed" : false
}
```

Server is not working!

TODO: Need to verify the server methods more after having learned about SurrealDB