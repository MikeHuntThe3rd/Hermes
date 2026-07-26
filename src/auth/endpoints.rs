use axum::{Json, http::StatusCode, extract::State};
use fred::interfaces::KeysInterface;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::types::*;
use time::OffsetDateTime;
use crate::auth::{creation::create_jwt, errors::AuthError};

pub async fn login(State(inf): State<AppState>, Json(data): Json<User>) -> (StatusCode, Json<Response<TokenPair>>) {
    let rows: Result<Vec<User>, sqlx::Error> = inf.db_interface
    .select(Some((&vec!["username", "password"], &vec![&data.username, &data.password]))).await;

    return match rows {
        Ok(val) => {
            if val.len() == 1 && 
            let Some(frst) = val.first() && 
            frst.username == data.username && 
            frst.password == data.password &&
            let Ok(access) = create_jwt(frst.id.expect("uuid has to be set in the id field"), TokenType::Access, &inf.jwt_secret).await &&
            let Ok(refresh) = create_jwt(frst.id.expect("uuid has to be set in the id field"), TokenType::Refresh, &inf.jwt_secret).await 
            {
                (StatusCode::OK, Json(Response { success: true, msg: String::new(), data: Some(
                TokenPair { 
                    access_tkn: access, 
                    refresh_tkn: refresh 
                }) }))
            }
            else {
                (StatusCode::UNAUTHORIZED
            , Json(Response { success: false, msg: "password or username is incorrect".to_string(), data: None }))
            }
        },
        Err(error) => (StatusCode::INTERNAL_SERVER_ERROR
            , Json(Response { success: false, msg: error.to_string(), data: None })),
    };
}

pub async fn refresh(State(inf): State<AppState>, Json(data): Json<RefreshBody>) -> Result<(StatusCode, Json<Response<TokenPair>>), AuthError> {
    let tkn_data = decode::<Claims>(
        &data.refresh_tkn, 
        &DecodingKey::from_secret(&inf.jwt_secret), 
        &Validation::default()).map_err(|_| AuthError::InvalidToken)?;
    
    if tkn_data.claims.tkn_type != TokenType::Refresh {
        return Err(AuthError::WrongTokenType);
    }

    let existence_key = format!("jwt:blacklist:{}", tkn_data.claims.jti);
    let is_revoked: i64 = inf.redis_client.exists(existence_key).await.map_err(|_| AuthError::RedisError)?;
    if is_revoked == 1 {
        return Err(AuthError::ExpiredToken);
    }

    let access = create_jwt(tkn_data.claims.sub, TokenType::Access, &inf.jwt_secret).await.map_err(|_| AuthError::MissingToken)?;
    let refresh = create_jwt(tkn_data.claims.sub, TokenType::Refresh, &inf.jwt_secret).await.map_err(|_| AuthError::MissingToken)?;

    let set_key = format!("jwt:blacklist:{}", tkn_data.claims.jti);
    let ttl = tkn_data.claims.exp as i64 - OffsetDateTime::now_utc().unix_timestamp();

    if ttl <= 0 {
        return Err(AuthError::InvalidToken);
    }
    
    inf.redis_client
    .set::<(), _, _>(set_key, 1, Some(fred::types::Expiration::EX(ttl)), None, false)
    .await.map_err(|_| AuthError::InvalidToken)?;

    let res = TokenPair{ access_tkn: access, refresh_tkn: refresh };

    return Ok((StatusCode::OK, Json(Response { success: true, msg: String::new(), data: Some(res) })));
}

