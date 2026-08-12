use axum::http::StatusCode;
use derive_macros::Bindable;
use fred::clients::Client;
use serde::{Serialize, Deserialize};
use sqlx::{Postgres, postgres::PgArguments, query::QueryAs};
use crate::db::*;
use uuid::Uuid;
/* ===== CONSTS ===== */

pub const TEMP_PTH_STR: &'static str = "/var/lib/hermes_objs/temp/";
pub const OBJ_PTH_STR: &'static str = "/var/lib/hermes_objs/objs/";
pub const LOGS_PTH_STR: &'static str = "/var/lib/hermes_objs/logs/log.db";

/* ===== TRAITS ===== */

pub trait Bindable {
    fn table_name() -> &'static str;

    fn columns() -> &'static [&'static str];
    fn id_columns() -> &'static [&'static str];
    fn base_columns() -> &'static [&'static str];

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, Self, PgArguments>,
        bind_val: BindVal)
        -> QueryAs<'lftm, Postgres, Self, PgArguments>
        where Self: Sized;
}

/* ===== ENUMS ===== */

#[derive(PartialEq)]
pub enum BindVal {
    ID,
    BASE,
    ALL,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "relation_t")]
pub enum RelationT {
    Friends, 
    Pending, 
    Blocked
}

/* ===== STRUCTS ===== */

pub struct Res<T>
where T: Serialize
{
    pub status: StatusCode,
    pub success: bool,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable, Clone)]
#[ids = "id"]
pub struct User {
    pub id: Option<Uuid>,
    pub nicname: String,
    pub username: String,
    pub password: String,
    pub pfp: Option<Uuid>,
}

#[derive(Serialize, Deserialize)]
pub struct StrippedUser {
    pub id: Option<Uuid>,
    pub nicname: String,
    pub pfp: Option<Uuid>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "id"]
pub struct Group {
    pub id: Option<Uuid>,
    pub name: String,
    pub is_dm: bool,
    pub gp: Option<Uuid>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "id"]
pub struct Message {
    pub id: Option<i32>,
    pub message: Option<String>,
    pub files: Option<Vec<Vec<u8>>>,
    pub group_id: Uuid,
    pub user_id: Uuid,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "id"]
pub struct Object {
  pub id: Option<Uuid>,
  pub hash : String,
  pub rel_path: String,
  pub mime_type: String,
  pub size_bytes: i64,
  pub creation_timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct ObjectIds {
    pub ids: Vec<Uuid>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "group_id;member_id"]
#[include_ids(true)]
pub struct Group_Member {
    pub group_id: Uuid,
    pub member_id: Uuid,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "message_id;object_id"]
#[include_ids(true)]
pub struct Message_Object {
    pub message_id: i32,
    pub object_id: Uuid,
}

pub struct Relation {
    relating_user: Uuid,
    related_user: Uuid,
    state: RelationT,
}

#[derive(sqlx::FromRow, Serialize, Bindable)]
#[ids="id"]
pub struct Log {
    pub id: Option<i32>,
    pub request: String,
    pub response: String,
}

#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: Vec<u8>,
    pub ps_interface: PsInterface,
    pub lite_interface: LiteInterface,
    pub redis_client: Client,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub jti: Uuid,
    pub iat: usize,
    pub exp: usize,
    pub tkn_type: TokenType,
}

#[derive(Serialize, Deserialize)]
pub struct TokenPair {
    pub access_tkn: String,
    pub refresh_tkn: String,
}

#[derive(Deserialize)]
pub struct RefreshBody {
    pub refresh_tkn: String,
}

