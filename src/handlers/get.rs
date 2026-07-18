use axum::extract::Path;
use axum::{Json, http::StatusCode};
use uuid::Uuid;

use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn get_user(Path(user_id) : Path<Uuid>) -> (StatusCode, Json<Response<Vec<User>>>) {
    let inf = get_db_interface().await;

    return match inf.select::<Uuid, User>(Some((&User::id_columns(), &vec![user_id]))).await {
        Ok(usr) => (StatusCode::OK
            , Json(Response{success: true, msg: String::new(), data: Some(usr)})),
        Err(e) => (StatusCode::NOT_FOUND
            , Json(Response{success: false, msg: e.to_string(), data: None})),
    }
}

