use axum::Json;
use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn add_user(Json(data): Json<UninitializedUser>) -> Result<Json<User>, axum::Error> {
    let inf = get_db_interface().await;

    return match inf.insert(data).await {
        Ok(()) => Json(),
        Err(e) => 
    }
}