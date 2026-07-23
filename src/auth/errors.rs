use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Clone)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    WrongTokenType,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (code, msg) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "given jwt is invalid"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "jwt token required but not found"),
            AuthError::WrongTokenType => (StatusCode::UNAUTHORIZED, "jwt type doesn't match the expected token type of the request"),
        };

        (code, Json(json!({"error_msg": msg}))).into_response()
    }
}

