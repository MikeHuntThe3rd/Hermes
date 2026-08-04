use axum::{Json, extract::State, http::StatusCode};

use crate::{auth::extractor::AuthUser, types::*};

pub async fn add_user(_auth: AuthUser, inf: State<AppState>, Json(data): Json<User>) -> (StatusCode, Json<Response<User>>) {
    return match inf.db_interface.insert::<User>(data).await {
        Ok(usr) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(usr)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn add_group(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group>) -> (StatusCode, Json<Response<Group>>) {
    return match inf.db_interface.insert::<Group>(data).await {
        Ok(grp) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(grp)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn add_group_member(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group_Member>) -> (StatusCode, Json<Response<Group_Member>>) {
    return match inf.db_interface.insert::<Group_Member>(data).await {
        Ok(grp_mem) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(grp_mem)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}

pub async fn add_message(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Message>) -> (StatusCode, Json<Response<Message>>) {
    return match inf.db_interface.insert::<Message>(data).await {
        Ok(msg) => (StatusCode::CREATED
            , Json(Response{success: true, msg: String::new(), data: Some(msg)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response {success: false, msg: e.to_string(), data: None})),
    }
}