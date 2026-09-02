use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::handlers::{StrippedGroup, fetch_group_member};
use crate::{auth::extractor::AuthUser, types::*, responses::error_t::{InternalError, AuthError, GenericErr}};

#[derive(Serialize, Deserialize)]
pub struct InvMng {
    pub accept: bool,
}

#[derive(Serialize, Deserialize)]
pub struct StatChange {
    pub state: RelationT,
}

pub async fn update_user(_auth: AuthUser, State(inf): State<AppState>, Json(data): Json<User>) -> Result<Res<User>, InternalError> {
    return match inf.ps_interface.update(data).await {
        Ok(usr) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(usr) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn update_group(auth: AuthUser, State(inf): State<AppState>, Path(group_id): Path<Uuid>, Json(data): Json<StrippedGroup>) -> Result<Res<()>, GenericErr> {
    let member = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
    .await.map_err(|e| GenericErr::Internal(e))?;

    if member.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    inf.ps_interface.update(Group {
        id: group_id,
        name: data.name,
        is_dm: data.is_dm,
        gp: data.gp
    }).await.map_err(|e| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}

pub async fn update_relation(auth: AuthUser, State(inf): State<AppState>, Path(user_id): Path<Uuid>, Json(data): Json<StatChange>) -> Result<Res<()>, GenericErr> {
    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    let rels: Vec<Relation> = inf.ps_interface.select(Some((&["relating_user", "related_user"], &[auth.user_id, user_id])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let mut rel = rels.into_iter().next().ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    rel.state = data.state;

    inf.ps_interface.update(rel).await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}

pub async fn manage_group_invite(auth: AuthUser, State(inf): State<AppState>, Path(invite_id): Path<Uuid>, Json(data): Json<InvMng>) -> Result<Res<()>, InternalError> {
    let invs: Vec<Group_Invite> = inf.ps_interface.select(Some((&["id"], &[invite_id])))
    .await.map_err(|_| InternalError::DbError)?;

    let inv: Group_Invite = invs.into_iter().next().ok_or(InternalError::NoMatches)?;
    
    if inv.user_id != auth.user_id {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        inf.ps_interface.insert::<Group_Member>(Group_Member {
            group_id: inv.group_id, 
            member_id: inv.user_id, 
            rank: inv.rank }, true)
        .await.map_err(|_| InternalError::DbError)?;
    }

    inf.ps_interface.delete::<Uuid, Group_Invite>(&[inv.id]).await.map_err(|_| InternalError::DbError)?;

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}

pub async fn manage_friend_invite(auth: AuthUser, State(inf): State<AppState>, Path(user_id): Path<Uuid>, Json(data): Json<InvMng>) -> Result<Res<()>, InternalError>{
    let invs: Vec<Relation> = inf.ps_interface.select(Some((&["relating_user", "related_user"], &[user_id, auth.user_id])))
    .await.map_err(|_| InternalError::DbError)?;

    let mut inv: Relation = invs.into_iter().next().ok_or(InternalError::NoMatches)?;

    if inv.state != RelationT::Pending {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        inv.state = RelationT::Friends;
        inf.ps_interface.update(inv).await.map_err(|_| InternalError::DbError)?;
        inf.ps_interface.insert(Relation {
            relating_user: auth.user_id,
            related_user: user_id,
            state: RelationT::Friends
        }, true).await.map_err(|_| InternalError::DbError)?;
    }
    else {
        inf.ps_interface.delete::<Uuid, Relation>(&[auth.user_id, user_id]).await.map_err(|_| InternalError::DbError)?;
    }

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}

pub async fn update_message(_auth: AuthUser, State(inf): State<AppState>, Json(data): Json<Message>) -> Result<Res<Message>, InternalError> {
    return match inf.ps_interface.update(data).await {
        Ok(msg) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(msg) }),
        Err(_e) => Err(InternalError::DbError),
    }
}