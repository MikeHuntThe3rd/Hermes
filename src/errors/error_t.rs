
#[derive(Clone)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    InvalidCredentials,
    WrongTokenType,
    ExpiredToken,
}

pub enum InternalError {
    DbError,
    RedisError,
    DecodeEncodeErr,
    OperationsError,
    NoMatches,
    BadRequest,
    BodyTooLarge,
}

pub enum GenericErr {
    Auth(AuthError),
    Internal(InternalError),
}