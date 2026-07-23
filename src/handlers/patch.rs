use axum::{Json, http::StatusCode};

use crate::types::*;
use crate::get_app_state;

pub async fn update_user(Json(data): Json<User>) ->(StatusCode, Json<Response<User>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.update(data).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(usr)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn update_group(Json(data): Json<Group>) ->(StatusCode, Json<Response<Group>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.update(data).await {
        Ok(grp) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(grp)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn update_message(Json(data): Json<Message>) ->(StatusCode, Json<Response<Message>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.update(data).await {
        Ok(msg) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(msg)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}