use axum::{Json, extract::State, http::StatusCode};

use crate::{auth::extractor::AuthUser, types::*, errors::error_t::InternalError};

pub async fn add_group(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group>) -> Result<Res<Group>, InternalError> {
    return match inf.db_interface.insert::<Group>(data).await {
        Ok(grp) => Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(grp) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn add_group_member(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group_Member>) -> Result<Res<Group_Member>, InternalError> {
    return match inf.db_interface.insert::<Group_Member>(data).await {
        Ok(grp_mem) => Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(grp_mem) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn add_message(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Message>) -> Result<Res<Message>, InternalError> {
    return match inf.db_interface.insert::<Message>(data).await {
        Ok(msg) => Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(msg) }),
        Err(_e) => Err(InternalError::DbError),
    }
}