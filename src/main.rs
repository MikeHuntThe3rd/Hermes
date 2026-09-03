mod handlers;
mod auth;
mod types;
mod cleaner;
mod logging;
mod responses;
mod db;

use std::{env, path, time::Duration};

use axum::{Router, routing::{delete, get, patch, post}, extract::DefaultBodyLimit};
use tower_http::{cors::CorsLayer, services::ServeDir};
use fred::{interfaces::{ClientLike, EventInterface}, types::{Builder, config::{Config, TcpConfig}}};
use uuid::Uuid;

use crate::types::PrivilegeT;
use db::PsInterface;
use types::{AppState, TEMP_PTH_STR, OBJ_PTH_STR};
use auth::endpoints::*;
use handlers::{post::*
    ,delete::*
    ,get::*
    ,patch::*};

use crate::{db::LiteInterface, types::User};

async fn place_holder_fn(){}

async fn ensure_master_user(state: AppState) {
    let inf = state.ps_interface;
    let (usr_nm, pswrd) = (env::var("M_USERNAME").expect("the variable for the master user's username is expected")
    , env::var("M_PASSWORD").expect("the variable for the master user's password is expected"));

    let user_s = inf.select::<String, User>(Some((&["username", "password"], &vec![usr_nm.clone(), pswrd.clone()])))
    .await.expect("master user querying is expected to succeed");

    if user_s.len() < 1 {
        inf.insert::<User>(&[User { 
            id: Uuid::new_v4(), 
            nickname: "master".to_string(), 
            prv: PrivilegeT::Proprietor, 
            username: usr_nm, 
            password: pswrd, 
            pfp: None 
        }], false)
        .await.expect("master user inserting is expected to succeed");
    }
}

async fn setup() -> AppState {
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

    let state = AppState {
        jwt_secret: env::var("JWT_SECRET").expect("the jwt secret is expected to exist").into_bytes(),
        ps_interface: PsInterface::new().await.expect("failed to create the postgres db connection"),
        lite_interface: LiteInterface::new().await.expect("failed to create the sqlite db connection"),
        redis_client: client,
    };

    ensure_master_user(state.clone()).await;
    
    return state;
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("a .env file is expected");
    let state = setup().await;
    tokio::spawn(cleaner::cleaner_subprocess(state.clone()));

    let auth: Router<AppState> = Router::new()
    .route("/login", post(login))
    .route("/refresh", post(refresh))
    .route("/sign_up", post(sign_up));

    let groups: Router<AppState> = Router::new()
    /* ===== Groups ===== */
    .route("/", post(add_group))
    .route("/all", get(get_groups))
    .route("/invites", get(get_group_invites))
    .route("/{group_id}/members", get(get_group_members))
    .route("/{group_id}", delete(delete_group))
    .route("/{group_id}", patch(update_group))
    .route("/{group_id}/member/{member_id}", delete(delete_group_member))
    .route("/{group_id}/invite", post(invite_group_member))
    .route("/invite/{invite_id}", patch(manage_group_invite))
    .route("/{group_id}/message", post(add_message))
    .route("/{group_id}/message/all", get(get_messages))
    .route("/{group_id}/message/{message_id}", patch(update_message))
    .route("/{group_id}/message/{message_id}", delete(delete_message));

    let relations: Router<AppState> = Router::new()
    .route("/friends", get(get_friends))
    .route("/invites", get(get_friend_invites))
    .route("/{user_id}/invite", post(create_friend_invite))
    .route("/invite/{user_id}", patch(manage_friend_invite))
    .route("/{user_id}", patch(update_relation))
    .route("/{user_id}", delete(delete_relation));

    let users: Router<AppState> = Router::new()
    .route("/me", delete(delete_self))
    .route("/me", patch(update_user));

    let objects: Router<AppState> = Router::new()
    .route("/upload", post(upload)).layer(DefaultBodyLimit::max(10000000))
    .route("/pull/{object_id}", get(pull)).layer(DefaultBodyLimit::max(10000000));

    let api = Router::new()
    /* ===== Invites ===== */
    .route("invite/new_user/{prv_level}", post(create_invite))
    /* ===== Auth ===== */
    .nest("/auth", auth)
    /* ===== Groups ===== */
    .nest("/group", groups)
    /* ===== Relations ===== */
    .nest("/relation", relations)
    /* ===== Users ===== */
    .nest("/user", users)
    /* ===== Objects ===== */
    .nest("/object", objects)
    /* ===== Generics ===== */
    .layer(CorsLayer::very_permissive())
    .with_state(state);

    let interface: Router<()> = Router::new()
    .nest("/apiV1", api)
    .fallback_service(ServeDir::new("res"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
    .await
    .expect("port 8000 is required to be free");

    axum::serve(listener, interface).await.expect("axum serve needs to start succesfully");
}
