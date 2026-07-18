use axum::extract::Path;
use axum::{Json, http::StatusCode};
use uuid::Uuid;

use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn delete_user(Path(user_id): Path<Uuid>) -> (StatusCode, Json<Response<User>>) {
    let inf = get_db_interface().await;

    return match inf.delete::<Uuid, User>(&vec![user_id]).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response { success: true, msg: String::new(), data: Some(usr) })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: e.to_string(), data: None })),
    }
}