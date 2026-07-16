use axum::{Json, http::StatusCode};

use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn add_user(Json(data): Json<UninitializedUser>) -> Result<Json<User>, StatusCode> {
    let inf = get_db_interface().await;

    return match inf.insert(data).await {
        Ok(usr) => Ok(Json(usr)),
        Err(_e) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}