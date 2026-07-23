use derive_macros::Bindable;
use serde::{Serialize, Deserialize};
use sqlx::{Postgres, postgres::PgArguments, query::QueryAs};
use crate::db::*;
use uuid::Uuid;

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

#[derive(Serialize)]
pub struct Response<T>
{
    pub success: bool,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "id"]
pub struct User {
    pub id: Option<Uuid>,
    pub username: String,
    pub password: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "id"]
pub struct Group {
    pub id: Option<Uuid>,
    pub name: String,
    pub is_dm: bool,
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
#[ids = "group_id;member_id"]
#[include_ids(true)]
pub struct Group_Member {
    pub group_id: Uuid,
    pub member_id: Uuid,
}

#[derive(Clone)]
pub struct AppState {
    pub jwt_secret: Vec<u8>,
    pub db_interface: DbInterface,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub user_id: Uuid,
    pub issued_t: usize,
    pub expr_t: usize,
    pub tkn_type: TokenType,
}