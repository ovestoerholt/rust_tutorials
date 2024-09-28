use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/list", get(list_of_json_data));

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


async fn list_of_json_data() -> ApiResponse {
    let msg0 = Hello{ message: "Hei og hei".to_string(),};
    let msg1 = Hello{ message: "Hoj og hoj".to_string(),};
    let v = vec![msg0, msg1];
    ApiResponse::JsonData(v)
}


enum ApiResponse {
    Ok,
    Created,
    JsonData(Vec<Hello>),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            ApiResponse::Ok => (StatusCode::OK).into_response(),
            ApiResponse::Created => (StatusCode::CREATED).into_response(),
            ApiResponse::JsonData(data) => (StatusCode::OK, Json(data)).into_response(),
        }
    }
}
