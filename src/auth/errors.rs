use axum::{http::StatusCode, response::{IntoResponse}, Json};
use crate::types::Response;

#[derive(Clone)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    WrongTokenType,
    ExpiredToken,
    RedisError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "given jwt is invalid"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "jwt token required but not found"),
            AuthError::WrongTokenType => (StatusCode::UNAUTHORIZED, "jwt type doesn't match the expected token type of the request"),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "given jwt is listed as expired"),
            AuthError::RedisError => (StatusCode::UNAUTHORIZED, "redis service is down. try again"),
        };

        let res: Response<()> = Response { success: false, msg: msg.to_string(), data: None }; 

        (code, Json(res)).into_response()
    }
}

