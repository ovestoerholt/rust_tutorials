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


