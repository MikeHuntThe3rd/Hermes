mod handlers;
mod types;
mod db;

use axum::{Router, routing::{Route, delete, get, patch, post}};
use tower_http::services::{ServeDir, ServeFile};

use handlers::{post::*
    , delete::*
    , get::*
    , patch::*};


#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("a .env file is expected");
    
    let api = Router::<()>::new()
    .route("/add_user", post(add_user))
    .route("/add_group", post(add_group))
    .route("/add_group_member", post(add_group_member))
    .route("/add_message", post(add_message))
    .route("/get_user/{user_id}", get(get_user))
    .route("/get_group/{group_id}", get(get_group))
    .route("/get_group_member/{group_id, member_id}", get(get_group_member))
    .route("/get_message/{message_id}", get(get_message))
    .route("/delete_user/{user_id}", delete(delete_user))
    .route("/delete_group/{group_id}", delete(delete_group))
    .route("/delete_group_member/{group_member_id}", delete(delete_group_member))
    .route("/delete_message/{message_id}", delete(delete_message))
    .route("/update_user", patch(update_user))
    .route("/update_group", patch(update_group))
    .route("/update_message", patch(update_message));

    let app = Router::<()>::new()
    .nest_service("/main", ServeDir::new("res/main"))
    .nest_service("/signup", ServeDir::new("res/signup"))
    .nest_service("/login", ServeDir::new("res/login"))
    .nest_service("/mainCss", ServeFile::new("res/css/style.css"))
    .nest_service("/loginCss", ServeDir::new("res/css/login.css"));

    let interface = Router::<()>::new()
    .nest("/apiV1", api)
    .nest("/app", app)
    .fallback_service(ServeDir::new("res/welcome"));


    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
    .await
    .expect("port 8000 is required to be free");

    axum::serve(listener, interface).await.expect("axum serve needs to start succesfully");
}
