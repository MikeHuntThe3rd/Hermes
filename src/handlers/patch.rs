use axum::extract::State;
use axum::{Json, http::StatusCode};

use crate::{auth::extractor::AuthUser, types::*};

pub async fn update_user(_auth: AuthUser, inf: State<AppState>, Json(data): Json<User>) ->(StatusCode, Json<Response<User>>) {
    return match inf.db_interface.update(data).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(usr)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn update_group(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group>) ->(StatusCode, Json<Response<Group>>) {
    return match inf.db_interface.update(data).await {
        Ok(grp) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(grp)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn update_message(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Message>) ->(StatusCode, Json<Response<Message>>) {
    return match inf.db_interface.update(data).await {
        Ok(msg) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(msg)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}