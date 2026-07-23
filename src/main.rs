mod handlers;
mod auth;
mod types;
mod db;

use axum::{Router, routing::{delete, get, patch, post}};
use tower_http::services::{ServeDir, ServeFile};
use tokio::sync::OnceCell;

use db::DbInterface;
use types::AppState;
use handlers::{post::*
    , delete::*
    , get::*
    , patch::*};

pub static APPSTATE: OnceCell<AppState> = OnceCell::const_new();

pub async fn get_app_state() -> &'static AppState {
    return APPSTATE.get_or_init(|| async {
        AppState {
            jwt_secret: vec![],
            db_interface: DbInterface::new().await.expect("failed to create the db connection"),
        }
    }).await;
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("a .env file is expected");
    
    let api = Router::new()
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
    .route("/update_message", patch(update_message))
    .with_state(get_app_state().await.clone());

    let interface = Router::<()>::new()
    .nest("/apiV1", api)
    .fallback_service(ServeDir::new("res"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
    .await
    .expect("port 8000 is required to be free");

    axum::serve(listener, interface).await.expect("axum serve needs to start succesfully");
}
