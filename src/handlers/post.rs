use axum::{Json, http::StatusCode};

use crate::types::*;
use crate::get_app_state;

pub async fn add_user(Json(data): Json<User>) -> (StatusCode, Json<Response<User>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.insert::<User>(data).await {
        Ok(usr) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(usr)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn add_group(Json(data): Json<Group>) -> (StatusCode, Json<Response<Group>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.insert::<Group>(data).await {
        Ok(grp) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(grp)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn add_group_member(Json(data): Json<Group_Member>) -> (StatusCode, Json<Response<Group_Member>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.insert::<Group_Member>(data).await {
        Ok(grp_mem) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(grp_mem)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn add_message(Json(data): Json<Message>) -> (StatusCode, Json<Response<Message>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.insert::<Message>(data).await {
        Ok(msg) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(msg)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}