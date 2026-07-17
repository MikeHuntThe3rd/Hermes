use axum::extract::Path;
use axum::{Json, http::StatusCode, extract::path};
use uuid::Uuid;

use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn delete_user(Path(user_id): Path<Uuid>) -> (StatusCode, Json<Response<User>>) {
    let inf = get_db_interface().await;

    return match inf.delete(&vec![user_id]).await {
        Ok(usr) => (StatusCode::OK, Json(Response { success: true, data: Some(usr) })),
        Err(_e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { success: false, data: None })),
    }
}