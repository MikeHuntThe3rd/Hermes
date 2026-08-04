use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, types::*};

pub async fn delete_user(_auth: AuthUser, State(inf): State<AppState>, Path(user_id): Path<Uuid>) -> (StatusCode, Json<Response<User>>) {
    return match inf.db_interface.delete::<Uuid, User>(&vec![user_id]).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(usr) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}

pub async fn delete_group(_auth: AuthUser, State(inf): State<AppState>, Path(group_id): Path<Uuid>) -> (StatusCode, Json<Response<Group>>) {
    return match inf.db_interface.delete::<Uuid, Group>(&vec![group_id]).await {
        Ok(grp) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(grp) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}

pub async fn delete_group_member(_auth: AuthUser, State(inf): State<AppState>, Path(group_member_id): Path<Uuid>) -> (StatusCode, Json<Response<Group_Member>>) {
    return match inf.db_interface.delete::<Uuid, Group_Member>(&vec![group_member_id]).await {
        Ok(grp_mem) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(grp_mem) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}

pub async fn delete_message(_auth: AuthUser, State(inf): State<AppState>, Path(message_id): Path<i32>) -> (StatusCode, Json<Response<Message>>) {
    return match inf.db_interface.delete::<i32, Message>(&vec![message_id]).await {
        Ok(message) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(message) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}