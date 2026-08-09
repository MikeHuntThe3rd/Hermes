use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::errors::error_t::InternalError;
use crate::{auth::extractor::AuthUser, types::*};

pub async fn delete_self(auth: AuthUser, State(inf): State<AppState>) -> Result<Res<()>, InternalError> {
    let deletes = inf.db_interface.delete::<Uuid, User>(&vec![auth.user_id])
    .await.map_err(|_| InternalError::DbError)?;
    
    if deletes.len() >= 1 {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(InternalError::NoMatches);
    }
}

pub async fn delete_group(_auth: AuthUser, State(inf): State<AppState>, Path(group_id): Path<Uuid>) -> Result<Res<()>, InternalError> {
    let deletes = inf.db_interface.delete::<Uuid, Group>(&vec![group_id])
    .await.map_err(|_| InternalError::DbError)?;
    
    if deletes.len() >= 1 {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(InternalError::NoMatches);
    }
}

pub async fn delete_group_member(_auth: AuthUser, State(inf): State<AppState>, Path(group_member_id): Path<Uuid>) -> Result<Res<()>, InternalError> {
    let deletes = inf.db_interface.delete::<Uuid, Group_Member>(&vec![group_member_id])
    .await.map_err(|_| InternalError::DbError)?;
    
    if deletes.len() >= 1 {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(InternalError::NoMatches);
    }
}

pub async fn delete_message(_auth: AuthUser, State(inf): State<AppState>, Path(message_id): Path<i32>) -> Result<Res<()>, InternalError> {
    let deletes = inf.db_interface.delete::<i32, Message>(&vec![message_id])
    .await.map_err(|_| InternalError::DbError)?;
    
    if deletes.len() >= 1 {
        return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
    }
    else {
        return Err(InternalError::NoMatches);
    }
}