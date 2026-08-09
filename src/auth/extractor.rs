use axum::{RequestPartsExt, extract::{FromRef, FromRequestParts}, http::request::Parts};

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use fred::interfaces::KeysInterface;
use jsonwebtoken::{decode, DecodingKey, Validation};
use uuid::Uuid;

use crate::{errors::error_t::*, types::{AppState, Claims, TokenType}};

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl<T> FromRequestParts<T> for AuthUser
where 
    T: Sync + Send,
    AppState: FromRef<T>
{
    type Rejection = GenericErr;
    async fn from_request_parts(
        parts: &mut Parts,
        state: &T,
    ) -> Result<Self, Self::Rejection>
    {
        let TypedHeader(Authorization(bearer)) = parts.extract::<TypedHeader<Authorization<Bearer>>>()
        .await.map_err(|_| GenericErr::Auth(AuthError::MissingToken))?;

        let app_state = AppState::from_ref(state);

        let tkn_data = decode::<Claims>(
            bearer.token(), 
            &DecodingKey::from_secret(&app_state.jwt_secret), 
            &Validation::default()).map_err(|_| GenericErr::Auth(AuthError::InvalidToken))?;
        
        if tkn_data.claims.tkn_type != TokenType::Access {
            return Err(GenericErr::Auth(AuthError::WrongTokenType));
        }
        
        let key = format!("jwt:blacklist:{}", tkn_data.claims.jti);
        let is_revoked: i64 = app_state.redis_client.exists(key).await.map_err(|_| GenericErr::Internal(InternalError::RedisError))?;
        if is_revoked == 1 {
            return Err(GenericErr::Auth(AuthError::ExpiredToken));
        }

        return Ok(AuthUser { user_id: tkn_data.claims.sub });
    }
}