use axum::Json;
use crate::types::*;

pub async fn send_msg(Json(message): Json<msg>) -> Json<msg> {
    return Json(msg { text: message.text});
} 