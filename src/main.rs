mod auth;
mod cleaner;
mod db;
mod handlers;
mod logging;
mod responses;
mod types;

use std::{env, path, sync::Arc, time::Duration};

use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{any, delete, get, patch, post},
};
use dashmap::DashMap;
use fred::{
    interfaces::{ClientLike, EventInterface},
    types::{
        Builder,
        config::{Config, TcpConfig},
    },
};
use tower_http::{cors::CorsLayer, services::ServeDir};
use uuid::Uuid;

use crate::types::PrivilegeT;
use auth::endpoints::*;
use db::PsInterface;
use handlers::{delete::*, get::*, patch::*, post::*, ws::*};
use types::{AppState, OBJ_PTH_STR, TEMP_PTH_STR};

use crate::{db::LiteInterface, types::User};

async fn ensure_master_user(state: AppState) {
    let inf = state.ps_interface;
    let (usr_nm, pswrd) = (
        env::var("M_USERNAME").expect("the variable for the master user's username is expected"),
        env::var("M_PASSWORD").expect("the variable for the master user's password is expected"),
    );

    let user_s = inf
        .select::<String, User>(
            Some((
                &["username", "password"],
                &vec![usr_nm.clone(), pswrd.clone()],
            )),
            None,
        )
        .await
        .expect("master user querying is expected to succeed");

    if user_s.len() < 1 {
        inf.insert::<User>(
            &[User {
                id: Uuid::new_v4(),
                nickname: "master".to_string(),
                prv: PrivilegeT::Proprietor,
                username: usr_nm,
                password: pswrd,
                pfp: None,
            }],
            false,
        )
        .await
        .expect("master user inserting is expected to succeed");
    }
}

async fn setup() -> AppState {
    if !path::Path::new(TEMP_PTH_STR).exists() || !path::Path::new(OBJ_PTH_STR).exists() {
        panic!("couldnt find resolve file paths");
    }
    let conf =
        Config::from_url("redis://localhost:6379/1").expect("redis binding is expected to succeed");
    let client = Builder::from_config(conf)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(1);
            config.tcp = TcpConfig {
                nodelay: Some(true),
                ..Default::default()
            }
        })
        .build()
        .expect("client builder is expected to succeed");

    client
        .init()
        .await
        .expect("client init is expected to succeed");

    client.on_error(|(error, server)| async move {
        println!("{:?}: Connection error: {:?}", server, error);
        Ok(())
    });

    let state = AppState {
        jwt_secret: env::var("JWT_SECRET")
            .expect("the jwt secret is expected to exist")
            .into_bytes()
            .into(),
        ps_interface: PsInterface::new()
            .await
            .expect("failed to create the postgres db connection"),
        lite_interface: LiteInterface::new()
            .await
            .expect("failed to create the sqlite db connection"),
        redis_client: client,
        pending_calls: Arc::new(DashMap::new()),
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

    let dm: Router<AppState> = Router::new()
        .route("/member/{member_id}", post(add_dm))
        .route("/invite/inital", get(get_inital_dm_invites))
        .route("/invite/{cursor}", get(get_dm_invites_from))
        .route("/invite/{invite_id}", patch(manage_dm_invite))
        .route("/{dm_id}", delete(delete_dm))
        .route("/{dm_id}/message", post())
        .route("/{dm_id}/message/{message_id}", patch())
        .route("/{dm_id}/message/{message_id}", delete())
        .route("/{dm_id}/message/inital", get(get_messages))
        .route("/{dm_id}/message/{cursor}", get(get_messages));

    let channel: Router<AppState> = Router::new()
        .route("/inital", get(get_inital_channels))
        .route("/{cursor}", get(get_channels_from))
        .route("/{channel_id}/message", post())
        .route("/{channel_id}/message/{message_id}", patch())
        .route("/{channel_id}/message/{message_id}", delete())
        .route("/{channel_id}/message/inital", get(get_messages))
        .route("/{channel_id}/message/{cursor}", get(get_messages))
        .route("/", post(add_guild_channel))
        .route("/{channel_id}", patch(update_channel))
        .route("/{channel_id}", delete(delete_channel));

    let guild: Router<AppState> = Router::new()
        .route("/", post(add_guild))
        .route("/inital", get(get_inital_guilds))
        .route("/{cursor}", get(get_guilds_from))
        .route("/invite/inital", get(get_inital_guild_invites))
        .route("/invite/{cursor}", get(get_guild_invites_from))
        .route("/{guild_id}/member/inital", get(get_inital_guild_members))
        .route("/{guild_id}/member/{cursor}", get(get_guild_members_from))
        .route("/invite/{invite_id}", patch(manage_guild_invite))
        .route("/{guild_id}/invite", post(invite_guild_member))
        .route("/{guild_id}", delete(delete_guild))
        .route("/{guild_id}", patch(update_guild))
        .route(
            "/{guild_id}/member/{member_id}",
            delete(delete_guild_member),
        )
        .nest("{guild_id}/channel", channel);

    let relations: Router<AppState> = Router::new()
        .route("/friend/inital", get(get_inital_friends))
        .route("/friend/{cursor}", get(get_friends_from))
        .route("/invite/inital", get(get_inital_friend_invites))
        .route("/invite/{cursor}", get(get_friend_invites_from))
        .route("/{user_id}/invite", post(create_friend_invite))
        .route("/invite/{user_id}", patch(manage_friend_invite))
        .route("/{user_id}", patch(update_relation))
        .route("/{user_id}", delete(delete_relation));

    let users: Router<AppState> = Router::new()
        .route("/me", delete(delete_self))
        .route("/me", patch(update_self));

    let objects: Router<AppState> = Router::new()
        .route("/upload", post(upload))
        .layer(DefaultBodyLimit::max(10000000))
        .route("/pull/{object_id}", get(pull))
        .layer(DefaultBodyLimit::max(10000000));

    let sockets: Router<AppState> = Router::new().route("/upgrade", any(ws_upgrade));

    let api = Router::new()
        /* ===== Invites ===== */
        .route("/invite/new_user", post(create_invite))
        /* ===== Auth ===== */
        .nest("/auth", auth)
        /* ===== Dms ===== */
        .nest("/dm", dm)
        /* ===== Guilds ===== */
        .nest("/guild", guild)
        /* ===== Relations ===== */
        .nest("/relation", relations)
        /* ===== Users ===== */
        .nest("/user", users)
        /* ===== Objects ===== */
        .nest("/object", objects)
        /* ===== WebSockets ===== */
        .nest("/ws", sockets)
        /* ===== Generics ===== */
        .layer(CorsLayer::very_permissive())
        .with_state(state);

    let interface: Router<()> = Router::new()
        .nest("/apiV1", api)
        .fallback_service(ServeDir::new("res"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("port 8000 is required to be free");

    axum::serve(listener, interface)
        .await
        .expect("axum serve needs to start succesfully");
}
