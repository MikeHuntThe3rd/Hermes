
#[derive(Clone)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
    InvalidCredentials,
    WrongTokenType,
    ExpiredToken,
    InvalidNickname,
    InvalidPrivilige,
    SelfInvite,
    UserUnreachable,
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
    EmptyMessage,
    DuplicateData,
}

pub enum GenericErr {
    Auth(AuthError),
    Internal(InternalError),
}