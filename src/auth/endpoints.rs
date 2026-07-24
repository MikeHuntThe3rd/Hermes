use axum::{Json, http::StatusCode};

use crate::types::*;
use crate::auth::creation::create_jwt;
use crate::get_app_state;

pub async fn login(Json(data): Json<User>) -> (StatusCode, Json<Response<TokenPair>>) {
    let inf = get_app_state().await;

    let rows: Result<Vec<User>, sqlx::Error> = inf.db_interface
    .select(Some((&vec!["username", "password"], &vec![&data.username, &data.password]))).await;

    return match rows {
        Ok(val) => {
            if val.len() == 1 && 
            let Some(frst) = val.first() && 
            frst.username == data.username && 
            frst.password == data.password &&
            let Ok(access) = create_jwt(frst.id.expect("uuid has to be set in the id field"), TokenType::Access, &inf.jwt_secret).await &&
            let Ok(refresh) = create_jwt(frst.id.expect("uuid has to be set in the id field"), TokenType::Refresh, &inf.jwt_secret).await 
            {
                (StatusCode::OK, Json(Response { success: true, msg: String::new(), data: Some(
                TokenPair { 
                    access_tkn: access, 
                    refresh_tkn: refresh 
                }) }))
            }
            else {
                (StatusCode::UNAUTHORIZED
            , Json(Response { success: false, msg: "password or username is incorrect".to_string(), data: None }))
            }
        },
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: error.to_string(), data: None })),
    };
}