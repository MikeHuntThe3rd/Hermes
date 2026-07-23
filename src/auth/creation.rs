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
        TokenType::Access => current_t + Duration::minutes(10),
        TokenType::Refresh => current_t + Duration::days(30),
    };

    let claims = Claims {
        user_id: user_id,
        issued_t: current_t.unix_timestamp() as usize,
        expr_t: expr_t.unix_timestamp() as usize,
        tkn_type: tkn_type,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
}