
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
    NoMatches,
}

pub enum GenericErr {
    Auth(AuthError),
    Internal(InternalError),
}