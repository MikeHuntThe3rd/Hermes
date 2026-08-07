
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
    UnknownType,
}

pub enum GenericErr {
    Auth(AuthError),
    Internal(InternalError),
}