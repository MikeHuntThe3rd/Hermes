pub mod delete;
pub mod patch;
pub mod post;
pub mod get;

use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use crate::AppState;
use uuid::Uuid;

use crate::{responses::error_t::InternalError, types::{Group, Group_Member, RankT}};

#[derive(Serialize, Deserialize, FromRow)]
pub struct StrippedMember {
    pub user_id: Uuid,
    pub nickname: String,
    pub pfp: Option<Uuid>,
    pub rank: RankT,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct StrippedUser {
    pub user_id: Uuid,
    pub nickname: String,
    pub pfp: Option<Uuid>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct StrippedGroup {
    pub name: String,
    pub is_dm: bool,
    pub gp: Option<Uuid>,
}


pub async fn fetch_group(inf: AppState, group_id: &Uuid) -> Result<Group, InternalError> {
    let groups: Vec<Group> = inf.ps_interface.select(Some((&["id"], &[group_id])))
    .await.map_err(|_| InternalError::DbError)?;

    let first = groups.into_iter().next().ok_or(InternalError::NoMatches)?;

    return Ok(first);
}

pub async fn fetch_group_member(inf: AppState, group_id: &Uuid, member_id: &Uuid) -> Result<Group_Member, InternalError> {
    let groups: Vec<Group_Member> = inf.ps_interface.select(Some((&["group_id", "member_id"], &[group_id, member_id])))
    .await.map_err(|_| InternalError::DbError)?;

    let first = groups.into_iter().next().ok_or(InternalError::NoMatches)?;

    return Ok(first);
}
