use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::types::*;

pub async fn get_user(auth: AuthUser, State(inf): State<AppState>) -> (StatusCode, Json<Response<Vec<User>>>) {
    return match inf.db_interface.select::<Uuid, User>(Some((&User::id_columns(), &vec![auth.user_id]))).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(usr)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn get_group(_auth: AuthUser, State(inf): State<AppState>, Path(group_id) : Path<Uuid>) -> (StatusCode, Json<Response<Vec<Group>>>) {
    return match inf.db_interface.select::<Uuid, Group>(Some((&Group::id_columns(), &vec![group_id]))).await {
        Ok(grp) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(grp)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn get_group_member(_auth: AuthUser, State(inf): State<AppState>, Path(group_id) : Path<Uuid>, Path(member_id) : Path<Uuid>) -> (StatusCode, Json<Response<Vec<Group_Member>>>) {
    println!("{}", group_id);
    println!("{}", member_id);

    return match inf.db_interface.select::<Uuid, Group_Member>(Some((&Group_Member::id_columns(), &vec![group_id, member_id]))).await {
        Ok(grp_mem) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(grp_mem)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn get_message(_auth: AuthUser, State(inf): State<AppState>, Path(message_id) : Path<i32>) -> (StatusCode, Json<Response<Vec<Message>>>) {
    return match inf.db_interface.select::<i32, Message>(Some((&Message::id_columns(), &vec![message_id]))).await {
        Ok(message) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(message)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}