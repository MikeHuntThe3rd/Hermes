
#[derive(Clone)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    InvalidCredentials,
    WrongTokenType,
    ExpiredToken,
    MismatchedPriviligeLevels,
}

pub enum InternalError {
    DbError,
    RedisError,
    UncleanRedisError,
    DecodeEncodeErr,
    OperationsError,
    NoMatches,
    BadRequest,
    BodyTooLarge,
    UnknownType,
}

pub enum GenericErr {
    Auth(AuthError),
    Internal(InternalError),
}