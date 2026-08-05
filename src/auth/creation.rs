use jsonwebtoken::{encode, EncodingKey, Header};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use crate::types::{Claims, TokenType};

pub async fn create_jwt(
    user_id: Uuid,
    tkn_type: TokenType,
    secret: &[u8]) -> Result<String, jsonwebtoken::errors::Error>
{
    let current_t = OffsetDateTime::now_utc();
    let expr_t = match tkn_type {
        TokenType::Access => current_t + Duration::minutes(15),
        TokenType::Refresh => current_t + Duration::days(3),
    };

    let claims = Claims {
        sub: user_id,
        jti: Uuid::new_v4(),
        iat: current_t.unix_timestamp() as usize,
        exp: expr_t.unix_timestamp() as usize,
        tkn_type: tkn_type,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}