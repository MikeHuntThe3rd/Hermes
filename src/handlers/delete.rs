use axum::extract::Path;
use axum::{Json, http::StatusCode};
use uuid::Uuid;

use crate::types::*;
use crate::get_app_state;

pub async fn delete_user(Path(user_id): Path<Uuid>) -> (StatusCode, Json<Response<User>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.delete::<Uuid, User>(&vec![user_id]).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(usr) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}

pub async fn delete_group(Path(group_id): Path<Uuid>) -> (StatusCode, Json<Response<Group>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.delete::<Uuid, Group>(&vec![group_id]).await {
        Ok(grp) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(grp) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}

pub async fn delete_group_member(Path(group_member_id): Path<Uuid>) -> (StatusCode, Json<Response<Group_Member>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.delete::<Uuid, Group_Member>(&vec![group_member_id]).await {
        Ok(grp_mem) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(grp_mem) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}

pub async fn delete_message(Path(message_id): Path<i32>) -> (StatusCode, Json<Response<Message>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.delete::<i32, Message>(&vec![message_id]).await {
        Ok(message) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(message) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}