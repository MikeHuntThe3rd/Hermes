mod handlers;
mod auth;
mod types;
mod db;
mod res_impls;

use std::{path, time::Duration};

use axum::{Router, routing::{delete, get, patch, post}};
use tower_http::services::{ServeDir};
use fred::{interfaces::{ClientLike, EventInterface}, types::{Builder, config::{Config, TcpConfig}}};

use db::DbInterface;
use types::AppState;
use auth::endpoints::*;
use handlers::{post::*
    ,delete::*
    ,get::*
    ,patch::*};

pub async fn get_app_state() -> AppState {
    if !path::Path::new("/var/lib/hermes_objs").exists() {
        panic!("couldnt find /var/lib/hermes_objs directory which is expected to exist");
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
        db_interface: DbInterface::new().await.expect("failed to create the db connection"),
        redis_client: client,
    };
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("a .env file is expected");
    let state = get_app_state().await.clone();

    let auth: Router<AppState> = Router::new()
    .route("/login", post(login))
    .route("/refresh", post(refresh));
    
    let api = Router::new()
    .nest("/auth", auth)
    .route("/add_user", post(add_user))
    .route("/add_group", post(add_group))
    .route("/add_group_member", post(add_group_member))
    .route("/add_message", post(add_message))
    .route("/get_user", get(get_user))
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
    .with_state(state);

    let interface: Router<()> = Router::new()
    .nest("/apiV1", api)
    .fallback_service(ServeDir::new("res"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
    .await
    .expect("port 8000 is required to be free");

    axum::serve(listener, interface).await.expect("axum serve needs to start succesfully");
}
