use axum::Json;
use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn add_user(Json(_data): Json<user>) {
    let inf = get_db_interface().await;
    inf.generic_query("ligma", "balls").await;
}