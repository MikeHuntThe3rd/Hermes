use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::types::*;
use crate::get_app_state;

pub async fn get_user(auth: AuthUser, State(inf): State<AppState>) -> (StatusCode, Json<Response<Vec<User>>>) {
    return match inf.db_interface.select::<Uuid, User>(Some((&User::id_columns(), &vec![auth.user_id]))).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(usr)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn get_group(Path(group_id) : Path<Uuid>) -> (StatusCode, Json<Response<Vec<Group>>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.select::<Uuid, Group>(Some((&Group::id_columns(), &vec![group_id]))).await {
        Ok(grp) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(grp)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn get_group_member(Path(group_id) : Path<Uuid>, Path(member_id) : Path<Uuid>) -> (StatusCode, Json<Response<Vec<Group_Member>>>) {
    let inf = get_app_state().await;

    println!("{}", group_id);
    println!("{}", member_id);

    return match inf.db_interface.select::<Uuid, Group_Member>(Some((&Group_Member::id_columns(), &vec![group_id, member_id]))).await {
        Ok(grp_mem) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(grp_mem)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn get_message(Path(message_id) : Path<i32>) -> (StatusCode, Json<Response<Vec<Message>>>) {
    let inf = get_app_state().await;

    return match inf.db_interface.select::<i32, Message>(Some((&Message::id_columns(), &vec![message_id]))).await {
        Ok(message) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(message)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}