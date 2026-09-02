use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::handlers::fetch_group_member;
use crate::responses::error_t::{GenericErr, InternalError, AuthError};
use crate::{auth::extractor::AuthUser, types::*};

pub async fn delete_self(auth: AuthUser, State(inf): State<AppState>) -> Result<Res<()>, InternalError> {
    let deletes = inf.ps_interface.delete::<Uuid, User>(&vec![auth.user_id])
    .await.map_err(|_| InternalError::DbError)?;
    
    if deletes.len() >= 1 {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(InternalError::NoMatches);
    }
}

pub async fn delete_group(auth: AuthUser, State(inf): State<AppState>, Path(group_id): Path<Uuid>) -> Result<Res<()>, GenericErr> {
    let rank: Group_Member = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
    .await.map_err(|e| GenericErr::Internal(e))?;

    if rank.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes = inf.ps_interface.delete::<Uuid, Group>(&vec![group_id])
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;
    
    if deletes.len() >= 1 {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }
}

pub async fn delete_group_member(auth: AuthUser, State(inf): State<AppState>, Path(group_id): Path<Uuid>, Path(member_id): Path<Uuid>) -> Result<Res<()>, GenericErr> {
    let caller = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
    .await.map_err(|e|GenericErr::Internal(e))?;
    let recipient = fetch_group_member(inf.clone(), &group_id, &member_id)
    .await.map_err(|e|GenericErr::Internal(e))?;

    if recipient.rank >= caller.rank {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let deletes: Vec<Group_Member> = inf.ps_interface.delete(&vec![group_id, member_id])
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;
    
    if deletes.first().is_some() {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }
}

pub async fn delete_relation(auth: AuthUser, State(inf): State<AppState>, Path(user_id): Path<Uuid>) -> Result<Res<()>, GenericErr> {
    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }
    let (caller_to_user, user_to_caller): (Vec<Relation>, Vec<Relation>) = (
        inf.ps_interface.delete::<Uuid, Relation>(&[auth.user_id, user_id]).await.map_err(|_| GenericErr::Internal(InternalError::DbError))?,
        inf.ps_interface.delete::<Uuid, Relation>(&[user_id, auth.user_id]).await.map_err(|_| GenericErr::Internal(InternalError::DbError))?
    );

    if caller_to_user.first().is_none() || user_to_caller.first().is_none() {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}
pub async fn delete_message(_auth: AuthUser, State(inf): State<AppState>, Path(message_id): Path<i32>) -> Result<Res<()>, InternalError> {
    let deletes = inf.ps_interface.delete::<i32, Message>(&vec![message_id])
    .await.map_err(|_| InternalError::DbError)?;
    
    if deletes.len() >= 1 {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(InternalError::NoMatches);
    }
}