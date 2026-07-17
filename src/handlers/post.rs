use axum::{Json, http::StatusCode};

use crate::types::*;
use crate::handlers::get_db_interface;

#[axum::debug_handler]
pub async fn add_user(Json(data): Json<User>) -> (StatusCode, Json<Response<User>>) {
    let inf = get_db_interface().await;

    return match inf.insert(data).await {
        Ok(usr) => (StatusCode::CREATED, Json(Response{success: true, data: Some(usr)})),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Response {success: false, data: None})),
    }
}





