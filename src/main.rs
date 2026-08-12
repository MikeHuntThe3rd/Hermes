mod handlers;
mod auth;
mod types;
mod cleaner;
mod logging;
mod responses;
mod db;

use std::{path, time::Duration};

use axum::{Router, routing::{delete, get, patch, post}, extract::DefaultBodyLimit};
use tower_http::services::{ServeDir};
use fred::{interfaces::{ClientLike, EventInterface}, types::{Builder, config::{Config, TcpConfig}}};

use db::PsInterface;
use types::{AppState, TEMP_PTH_STR, OBJ_PTH_STR};
use auth::endpoints::*;
use handlers::{post::*
    ,delete::*
    ,get::*
    ,patch::*};

use crate::db::LiteInterface;

pub async fn get_app_state() -> AppState {
    if !path::Path::new(TEMP_PTH_STR).exists() ||
    !path::Path::new(OBJ_PTH_STR).exists()
    {
        panic!("couldnt find resolve file paths");
    }
    let conf = Config::from_url("redis://localhost:6379/1").expect("redis binding is expected to succeed");
    let client = Builder::from_config(conf)
    .with_connection_config(|config| {
        config.connection_timeout = Duration::from_secs(1);
        config.tcp = TcpConfig {
            nodelay: Some(true),
            ..Default::default()
        }
    }).build().expect("client builder is expected to succeed");

    client.init().await.expect("client init is expected to succeed");

    client.on_error(|(error, server)| async move {
        println!("{:?}: Connection error: {:?}", server, error);
        Ok(())
    });
    
    return AppState {
        jwt_secret: vec![],
        ps_interface: PsInterface::new().await.expect("failed to create the postgres db connection"),
        lite_interface: LiteInterface::new().await.expect("failed to create the sqlite db connection"),
        redis_client: client,
    };
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("a .env file is expected");
    let state = get_app_state().await;
    tokio::spawn(cleaner::cleaner_subprocess(state.clone()));

    let auth: Router<AppState> = Router::new()
    .route("/login", post(login))
    .route("/refresh", post(refresh))
    .route("/sign_up", post(sign_up));
    
    let api = Router::new()
    .nest("/auth", auth)
    /* ===== POST ===== */
    .route("/add_group", post(add_group))
    .route("/add_group_member", post(add_group_member))
    .route("/add_message", post(add_message))
    /* ===== GET ===== */
    .route("/get_friends", get(get_friends))
    .route("/get_groups", get(get_groups))
    .route("/get_group_members/{group_id}", get(get_group_members))
    .route("/get_messages/{group_id}", get(get_messages))
    /* ===== DELETE ===== */
    .route("/delete_self", delete(delete_self))
    .route("/delete_group/{group_id}", delete(delete_group))
    .route("/delete_group_member/{group_member_id}", delete(delete_group_member))
    .route("/delete_message/{message_id}", delete(delete_message))
    /* ===== PATCH ===== */
    .route("/update_user", patch(update_user))
    .route("/update_group", patch(update_group))
    .route("/update_message", patch(update_message))
    /* ===== OBJECTS ===== */
    .route("/upload", post(upload)).layer(DefaultBodyLimit::max(10000000))
    .route("/pull/{object_id}", get(pull)).layer(DefaultBodyLimit::max(10000000))
    .with_state(state);

    let interface: Router<()> = Router::new()
    .nest("/apiV1", api)
    .fallback_service(ServeDir::new("res"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
    .await
    .expect("port 8000 is required to be free");

    axum::serve(listener, interface).await.expect("axum serve needs to start succesfully");
}
