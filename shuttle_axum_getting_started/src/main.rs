use axum::{routing::get, Json, Router};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(hello_world));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> Json<Hello> {
    let msg = Hello {
        message: "Hello World!".to_owned(),
    };
    Json(msg)
}

#[derive(Serialize, Deserialize, Debug)]

struct Hello {
    message: String,
}
