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
    NonOwner,
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
    DetachedOwner,
}

pub enum GenericErr {
    Auth(AuthError),
    Internal(InternalError),
}
