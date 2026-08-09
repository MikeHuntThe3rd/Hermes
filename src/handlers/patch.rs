use axum::extract::State;
use axum::{Json, http::StatusCode};

use crate::{auth::extractor::AuthUser, types::*, errors::error_t::InternalError};

pub async fn update_user(_auth: AuthUser, inf: State<AppState>, Json(data): Json<User>) -> Result<Res<User>, InternalError> {
    return match inf.db_interface.update(data).await {
        Ok(usr) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(usr) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn update_group(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group>) -> Result<Res<Group>, InternalError> {
    return match inf.db_interface.update(data).await {
        Ok(grp) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(grp) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn update_message(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Message>) -> Result<Res<Message>, InternalError> {
    return match inf.db_interface.update(data).await {
        Ok(msg) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(msg) }),
        Err(_e) => Err(InternalError::DbError),
    }
}