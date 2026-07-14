mod handlers;
mod types;

use axum::{routing::{get, post}, Router, Json};
use tower_http::services::{ServeDir, ServeFile};

use crate::handlers::*;




#[tokio::main]
async fn main() {
    let api = Router::<()>::new()
    .route("/send_msg", post(send_msg));

    let interface = Router::<()>::new()
    .nest("/apiV1", api)
    .fallback_service(ServeDir::new("res"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.expect("port 8000 is required to be free");
    axum::serve(listener, interface).await.expect("axum serve needs to start succesfully");
}
