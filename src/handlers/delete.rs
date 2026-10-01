use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::handlers::fetch_guild_member;
use crate::responses::error_t::{AuthError, GenericErr, InternalError};
use crate::{auth::extractor::AuthUser, types::*};

pub async fn delete_self(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Res<()>, InternalError> {
    let query = sqlx::query_as!(
        Guild,
        "SELECT guilds.* FROM guilds
        JOIN guild_members gm ON gm.guild_id = guilds.id
        WHERE gm.member_id = $1
        AND gm.rank = $2
        AND (SELECT 1 FROM guild_members gm2
            WHERE gm2.guild_id = guilds.id AND gm2.rank = $2
        ) = 1;",
        auth.user_id,
        RankT::Owner as RankT
    );

    let orphan_guilds = state.ps_interface.generic_fetch(query).await?;

    if !orphan_guilds.is_empty() {
        return Err(InternalError::DetachingOperation);
    }

    let deletes = state
        .ps_interface
        .delete::<Uuid, User>(&vec![auth.user_id])
        .await?;

    if deletes.is_empty() {
        return Ok(Res {
            status: StatusCode::OK,
            success: true,
            msg: String::new(),
            data: None,
        });
    } else {
        return Err(InternalError::NoMatches);
    }
}

pub async fn delete_guild(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(guild_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    let rank: Guild_Member = fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if rank.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let delete = state
        .ps_interface
        .delete::<Uuid, Guild>(&vec![guild_id])
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if !delete.is_empty() {
        return Ok(Res {
            status: StatusCode::OK,
            success: true,
            msg: String::new(),
            data: None,
        });
    } else {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }
}

pub async fn delete_dm(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(dm_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    let dm: Dm = state
        .ps_interface
        .select(Some((&["id"], &[dm_id])), None)
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    if Some(auth.user_id) != dm.user_a && Some(auth.user_id) != dm.user_b {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let delete = state
        .ps_interface
        .delete::<Uuid, Dm>(&vec![dm_id])
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if !delete.is_empty() {
        return Ok(Res {
            status: StatusCode::OK,
            success: true,
            msg: String::new(),
            data: None,
        });
    } else {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }
}

pub async fn delete_guild_member(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
    Path(member_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    let caller = fetch_guild_member(state.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;
    let recipient = fetch_guild_member(state.clone(), &group_id, &member_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if recipient.rank >= caller.rank {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes: Vec<Guild_Member> = state
        .ps_interface
        .delete(&vec![group_id, member_id])
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    if deletes.first().is_some() {
        return Ok(Res {
            status: StatusCode::OK,
            success: true,
            msg: String::new(),
            data: None,
        });
    } else {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }
}

pub async fn delete_relation(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }
    let (caller_to_user, user_to_caller): (Vec<Relation>, Vec<Relation>) = (
        state
            .ps_interface
            .delete::<Uuid, Relation>(&[auth.user_id, user_id])
            .await
            .map_err(|_| GenericErr::Internal(InternalError::DbError))?,
        state
            .ps_interface
            .delete::<Uuid, Relation>(&[user_id, auth.user_id])
            .await
            .map_err(|_| GenericErr::Internal(InternalError::DbError))?,
    );

    if caller_to_user.first().is_none() || user_to_caller.first().is_none() {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

//TODO: new dele ep for message del/updates
pub async fn delete_message(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((group_id, message_id)): Path<(Uuid, i64)>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_guild_member(state.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if member.rank == RankT::User {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes = state
        .ps_interface
        .delete::<i64, Dm_Message>(&vec![message_id])
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    if !deletes.is_empty() {
        return Ok(Res {
            status: StatusCode::OK,
            success: true,
            msg: String::new(),
            data: None,
        });
    } else {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }
}

pub async fn delete_channel(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((group_id, channel_id)): Path<(Uuid, Uuid)>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_guild_member(state.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    state
        .ps_interface
        .select::<Uuid, Channel>(Some((&["id"], &[channel_id])), None)
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    if member.rank < RankT::Admin {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    state
        .ps_interface
        .delete::<Uuid, Channel>(&[channel_id])
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}
