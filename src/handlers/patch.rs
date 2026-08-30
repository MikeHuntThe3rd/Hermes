use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use uuid::Uuid;

use crate::handlers::{StrippedGroup, fetch_group_member};
use crate::{auth::extractor::AuthUser, types::*, responses::error_t::{InternalError, AuthError, GenericErr}};

pub async fn update_user(_auth: AuthUser, inf: State<AppState>, Json(data): Json<User>) -> Result<Res<User>, InternalError> {
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

pub async fn update_message(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Message>) -> Result<Res<Message>, InternalError> {
    return match inf.ps_interface.update(data).await {
        Ok(msg) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(msg) }),
        Err(_e) => Err(InternalError::DbError),
    }
}