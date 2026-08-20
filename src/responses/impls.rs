use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

use crate::types::Res;
use super::error_t::*;

#[derive(Serialize)]
struct ResBody<T> {
    success: bool,
    msg: String,
    data: Option<T>,
}

impl<T> IntoResponse for Res<T>
where T: Serialize
{
    fn into_response(self) -> axum::response::Response {
        let body = ResBody { 
            success: self.success, 
            msg: self.msg, 
            data: self.data
        };

        let mut resp = Json(body).into_response();

        *resp.status_mut() = self.status;

        return resp;
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "given jwt is invalid"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "password or username is incorrect"),
            AuthError::MissingToken => (StatusCode::UNAUTHORIZED, "jwt token required but not found"),
            AuthError::WrongTokenType => (StatusCode::UNAUTHORIZED, "jwt type doesn't match the expected token type of the request"),
            AuthError::ExpiredToken => (StatusCode::UNAUTHORIZED, "given jwt is listed as expired"),
            AuthError::InvalidPrivilige => (StatusCode::UNAUTHORIZED, "provided privilige level isnt high enough"),
            AuthError::InvalidNickname => (StatusCode::UNAUTHORIZED, "provided username does not meet the required 3 character limit"),
        };

        let res: Res<()> = Res { status: code, success: false, msg: msg.to_string(), data: None }; 

        return res.into_response();
    }
}

impl IntoResponse for InternalError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            InternalError::DbError => (StatusCode::INTERNAL_SERVER_ERROR, "given data resulted in a faulty db request"),
            InternalError::RedisError => (StatusCode::INTERNAL_SERVER_ERROR, "redis service is down. try again"),
            InternalError::UncleanRedisError => (StatusCode::INTERNAL_SERVER_ERROR, "redis service failed in an unsaved state"),
            InternalError::DecodeEncodeErr => (StatusCode::INTERNAL_SERVER_ERROR, "an error occured while encoding/decoding data"),
            InternalError::OperationsError => (StatusCode::INTERNAL_SERVER_ERROR, "internal operations in the server failed"),
            InternalError::NoMatches => (StatusCode::NOT_FOUND, "no row matched the given data"),
            InternalError::BadRequest => (StatusCode::BAD_REQUEST, "the request body was incorrectly formatted"),
            InternalError::BodyTooLarge => (StatusCode::PAYLOAD_TOO_LARGE, "the request body exceeded the maximum size allowed"),
            InternalError::UnknownType => (StatusCode::NOT_FOUND, "type of the file could not be infered"),
        };

        let res: Res<()> = Res { status: code, success: false, msg: msg.to_string(), data: None }; 

        return res.into_response();
    }
}

impl IntoResponse for GenericErr {
    fn into_response(self) -> axum::response::Response {
        return match self {
            GenericErr::Auth(au) => au.into_response(),
            GenericErr::Internal(int) => int.into_response(),
        };
    }
}

