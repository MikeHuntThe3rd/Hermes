pub mod delete;
pub mod get;
pub mod patch;
pub mod post;
pub mod ws;

use crate::AppState;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::{
    responses::error_t::InternalError,
    types::{Guild_Member, RankT},
};

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

#[derive(Serialize, Deserialize)]
pub struct Msg {
    pub file_ids: Vec<Uuid>,
    pub message: Option<String>,
}

pub async fn fetch_guild_member(
    state: AppState,
    guild_id: &Uuid,
    member_id: &Uuid,
) -> Result<Guild_Member, InternalError> {
    let groups: Vec<Guild_Member> = state
        .ps_interface
        .select(
            Some((&["guild_id", "member_id"], &[guild_id, member_id])),
            None,
        )
        .await?;

    let first = groups.into_iter().next().ok_or(InternalError::NoMatches)?;

    return Ok(first);
}
