use axum::{Json, http::StatusCode};

use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn upd_test(Json(data): Json<User>) -> Result<Json<User>, StatusCode> {
    let inf = get_db_interface().await;

    return match inf.update(data).await {
        Ok(usr) => Ok(Json(usr)),
        Err(_e) => {
            println!("{}", _e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

