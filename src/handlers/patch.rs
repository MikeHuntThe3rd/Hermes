use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, query};
use sqlx::postgres::PgArguments;
use sqlx::query::Query;
use std::collections::HashSet;
use uuid::Uuid;

use crate::handlers::{Msg, StrippedGroup, fetch_group_member};
use crate::{
    auth::extractor::AuthUser,
    responses::error_t::{AuthError, GenericErr, InternalError},
    types::*,
};

#[derive(Serialize, Deserialize)]
pub struct InvMng {
    pub accept: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StatChange {
    pub state: RelationT,
}

#[derive(Serialize, Deserialize)]
pub struct UserUpdate {
    pub nickname: String,
    pub username: String,
    pub password: String,
    pub pfp: Option<Uuid>,
}

pub async fn update_self(
    auth: AuthUser,
    State(inf): State<AppState>,
    Json(data): Json<UserUpdate>,
) -> Result<Res<User>, GenericErr> {
    if data.nickname.trim().len() < 3 {
        return Err(GenericErr::Auth(AuthError::InvalidNickname));
    }

    if data.username.trim().len() < 3 || data.password.trim().len() < 3 {
        return Err(GenericErr::Internal(InternalError::InvalidAccountCredentials));
    }

    let old_user: User = inf.ps_interface.select(Some((&["id"], &[auth.user_id])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?
    .into_iter().next().ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    inf.ps_interface.update(User{
        id: auth.user_id,
        nickname: data.nickname,
        prv: old_user.prv,
        username: data.username,
        password: data.password,
        pfp: data.pfp
    })
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}

pub async fn update_group(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(group_id): Path<Uuid>,
    Json(data): Json<StrippedGroup>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if member.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    inf.ps_interface
        .update(Group {
            id: group_id,
            name: data.name,
            is_dm: data.is_dm,
            gp: data.gp,
        })
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn update_relation(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<StatChange>,
) -> Result<Res<()>, GenericErr> {
    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    let rels: Vec<Relation> = inf
        .ps_interface
        .select(Some((
            &["relating_user", "related_user"],
            &[auth.user_id, user_id],
        )))
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let mut rel = rels
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    rel.state = data.state;

    inf.ps_interface
        .update(rel)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn manage_group_invite(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(invite_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let invs: Vec<Group_Invite> = inf
        .ps_interface
        .select(Some((&["id"], &[invite_id])))
        .await
        .map_err(|_| InternalError::DbError)?;

    let inv: Group_Invite = invs.into_iter().next().ok_or(InternalError::NoMatches)?;

    if inv.user_id != auth.user_id {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        inf.ps_interface
            .insert::<Group_Member>(
                &[Group_Member {
                    group_id: inv.group_id,
                    member_id: inv.user_id,
                    rank: inv.rank,
                }],
                true,
            )
            .await
            .map_err(|_| InternalError::DbError)?;
    }

    inf.ps_interface
        .delete::<Uuid, Group_Invite>(&[inv.id])
        .await
        .map_err(|_| InternalError::DbError)?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn manage_friend_invite(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let invs: Vec<Relation> = inf
        .ps_interface
        .select(Some((
            &["relating_user", "related_user"],
            &[user_id, auth.user_id],
        )))
        .await
        .map_err(|_| InternalError::DbError)?;

    let mut inv: Relation = invs.into_iter().next().ok_or(InternalError::NoMatches)?;

    if inv.state != RelationT::Pending {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        inv.state = RelationT::Friends;
        inf.ps_interface
            .update(inv)
            .await
            .map_err(|_| InternalError::DbError)?;
        inf.ps_interface
            .insert(
                &[Relation {
                    relating_user: auth.user_id,
                    related_user: user_id,
                    state: RelationT::Friends,
                }],
                true,
            )
            .await
            .map_err(|_| InternalError::DbError)?;
    } else {
        inf.ps_interface
            .delete::<Uuid, Relation>(&[auth.user_id, user_id])
            .await
            .map_err(|_| InternalError::DbError)?;
    }

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn update_message(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(group_id): Path<Uuid>,
    Path(message_id): Path<i32>,
    Json(data): Json<Msg>,
) -> Result<Res<()>, GenericErr> {
    fetch_group_member(inf.clone(), &group_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let msg: Message = inf
        .ps_interface
        .select::<i32, Message>(Some((&["id"], &[message_id])))
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    match msg.user_id {
        Some(msg_id) => {
            if msg_id != auth.user_id {
                return Err(GenericErr::Auth(AuthError::NonOwner));
            }
        }
        None => {
            return Err(GenericErr::Internal(InternalError::DetachedOwner));
        }
    }

    inf.ps_interface
        .update::<Message>(Message {
            id: 0,
            message: data.message,
            group_id: group_id,
            user_id: Some(auth.user_id),
        })
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let old_objs_hash: HashSet<Uuid> = inf
        .ps_interface
        .select(Some((&["message_id"], &[msg.id])))
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?
        .into_iter()
        .map(|m: Message_Object| m.object_id)
        .collect();

    let new_objs_hash: HashSet<Uuid> = data.file_ids.into_iter().collect();

    let insert_objs: Vec<Message_Object> = new_objs_hash.difference(&old_objs_hash)
    .map(|obj_id| Message_Object { message_id: msg.id, object_id: *obj_id })
    .collect();

    let delete_objs: Vec<&Uuid> = old_objs_hash.difference(&new_objs_hash).collect();

    inf.ps_interface.insert(&insert_objs,true,)
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let sql: &'static str = "DELETE FROM message_objects WHERE message_id = $1 AND object_id = ANY($2);";

    let query: Query<'_, Postgres, PgArguments> = query(sql).bind(msg.id).bind(delete_objs);

    inf.ps_interface.generic_exec(query).await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}
