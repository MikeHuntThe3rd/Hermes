use axum::{Json, http::StatusCode, extract::State};
use fred::interfaces::KeysInterface;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::types::*;
use crate::{auth::{creation::create_jwt}, errors::error_t::*};

#[derive(Serialize, Deserialize)]
pub struct UserTokenObj {
    pub user_data: User,
    pub token_data: TokenPair,
}

pub async fn login(State(inf): State<AppState>, Json(data): Json<User>) -> Result<Res<UserTokenObj>, GenericErr> {
    let rows: Vec<User> = inf.db_interface
    .select(Some((&vec!["username", "password"], &vec![&data.username, &data.password])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    if rows.len() == 1 && 
    let Some(frst) = rows.first() && 
    frst.username == data.username && 
    frst.password == data.password
    {
        let access = create_jwt(frst.id.expect("uuid has to be set in the id field"), TokenType::Access, &inf.jwt_secret)
        .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
        let refresh = create_jwt(frst.id.expect("uuid has to be set in the id field"), TokenType::Refresh, &inf.jwt_secret)
        .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

        let dta = UserTokenObj {
            user_data: frst.clone(),
            token_data: TokenPair { access_tkn: access, refresh_tkn: refresh }
        };

        return Ok(
            Res { 
                status: StatusCode::OK, 
                success: true, 
                msg: String::new(), 
                data: Some(dta) 
            }
        );
    }
    else {
        return Err(GenericErr::Auth(AuthError::InvalidCredentials));
    }
}

pub async fn refresh(State(inf): State<AppState>, Json(data): Json<RefreshBody>) -> Result<Res<TokenPair>, GenericErr> {
    let tkn_data = decode::<Claims>(
        &data.refresh_tkn, 
        &DecodingKey::from_secret(&inf.jwt_secret), 
        &Validation::default()).map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
    
    if tkn_data.claims.tkn_type != TokenType::Refresh {
        return Err(GenericErr::Auth(AuthError::WrongTokenType));
    }

    let existence_key = format!("jwt:blacklist:{}", tkn_data.claims.jti);
    let is_revoked: i64 = inf.redis_client.exists(existence_key)
    .await.map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    if is_revoked == 1 {
        return Err(GenericErr::Auth(AuthError::ExpiredToken));
    }

    let access = create_jwt(tkn_data.claims.sub, TokenType::Access, &inf.jwt_secret)
    .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
    let refresh = create_jwt(tkn_data.claims.sub, TokenType::Refresh, &inf.jwt_secret)
    .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let set_key = format!("jwt:blacklist:{}", tkn_data.claims.jti);
    let ttl = tkn_data.claims.exp as i64 - OffsetDateTime::now_utc().unix_timestamp();

    if ttl <= 0 {
        return Err(GenericErr::Auth(AuthError::InvalidToken));
    }
    
    inf.redis_client
    .set::<(), _, _>(set_key, 1, Some(fred::types::Expiration::EX(ttl)), None, false)
    .await.map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    let res = TokenPair{ access_tkn: access, refresh_tkn: refresh };

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(res) });
}

pub async fn sign_up(inf: State<AppState>, Json(data): Json<User>) -> Result<Res<UserTokenObj>, GenericErr> {
    let usr = inf.db_interface.insert::<User>(data)
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let id = if let Some(id_val) = usr.id {
        id_val
    }
    else {
        return Err(GenericErr::Internal(InternalError::DbError));
    };

    let access = create_jwt(id, TokenType::Access, &inf.jwt_secret)
    .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
    let refresh = create_jwt(id, TokenType::Refresh, &inf.jwt_secret)
    .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
    
    let dta = UserTokenObj {
        user_data: usr,
        token_data: TokenPair { access_tkn: access, refresh_tkn: refresh }
    };

    Ok(
        Res{ 
            status: StatusCode::OK, 
            success: true, 
            msg: String::new(), 
            data: Some(dta)
        }
    )
}