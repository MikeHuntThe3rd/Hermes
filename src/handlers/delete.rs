use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;
use sqlx::{Postgres, query_as};
use uuid::Uuid;

use crate::handlers::fetch_group_member;
use crate::responses::error_t::{AuthError, GenericErr, InternalError};
use crate::{auth::extractor::AuthUser, types::*};

pub async fn delete_self(
    auth: AuthUser,
    State(inf): State<AppState>,
) -> Result<Res<()>, InternalError> {
    let sql: &'static str = "SELECT * FROM groups
    JOIN group_members gm ON gm.group_id = groups.id
    WHERE gm.member_id = $1
    AND gm.rank = $2
    AND (SELECT COUNT(*) FROM group_members gm2
        WHERE gm2.group_id = groups.id AND gm2.rank = $2
    ) = 1;";

    let query: QueryAs<'_, Postgres, Group, PgArguments> =
        query_as(sql).bind(auth.user_id).bind(RankT::Owner);

    let orphan_groups = inf
        .ps_interface
        .generic_fetch(query)
        .await
        .map_err(|_| InternalError::DbError)?;

    if !orphan_groups.is_empty() {
        return Err(InternalError::DetachingOperation);
    }

    let deletes = inf
        .ps_interface
        .delete::<Uuid, User>(&vec![auth.user_id])
        .await
        .map_err(|_| InternalError::DbError)?;

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
    State(inf): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    let rank: Guild_Member = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if rank.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes = inf
        .ps_interface
        .delete::<Uuid, Group>(&vec![group_id])
        .await
        .map_err(|e| GenericErr::Internal(e))?;

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

pub async fn delete_dm(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    let dm: Dm = inf
        .ps_interface
        .select(Some((&["group_id"], &[group_id])))
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    if Some(auth.user_id) != dm.user_a && Some(auth.user_id) != dm.user_b {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes = inf
        .ps_interface
        .delete::<Uuid, Group>(&vec![group_id])
        .await
        .map_err(|e| GenericErr::Internal(e))?;

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

pub async fn delete_guild_member(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(group_id): Path<Uuid>,
    Path(member_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    let caller = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;
    let recipient = fetch_group_member(inf.clone(), &group_id, &member_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if recipient.rank >= caller.rank {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes: Vec<Guild_Member> = inf
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
    State(inf): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }
    let (caller_to_user, user_to_caller): (Vec<Relation>, Vec<Relation>) = (
        inf.ps_interface
            .delete::<Uuid, Relation>(&[auth.user_id, user_id])
            .await
            .map_err(|_| GenericErr::Internal(InternalError::DbError))?,
        inf.ps_interface
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

pub async fn delete_message(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path((group_id, message_id)): Path<(Uuid, i32)>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if member.rank == RankT::User {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes = inf
        .ps_interface
        .delete::<i32, Message>(&vec![message_id])
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
    _auth: AuthUser,
    State(inf): State<AppState>,
    Path((group_id, user_id, channel_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_group_member(inf.clone(), &group_id, &user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    inf.ps_interface
        .select::<Uuid, Channel>(Some((&["id"], &[channel_id])))
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    if member.rank < RankT::Admin {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    inf.ps_interface
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
