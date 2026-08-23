use axum::{Json, http::StatusCode, extract::State};
use fred::interfaces::KeysInterface;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::auth::extractor::AuthInvite;
use crate::types::*;
use crate::{auth::{creation::create_jwt}, responses::error_t::*};

#[derive(Serialize, Deserialize)]
pub struct UserTokenObj {
    pub user_data: User,
    pub token_data: TokenPair,
}

#[derive(Deserialize)]
pub struct RefreshBody {
    pub refresh_tkn: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenPair {
    pub access_tkn: String,
    pub refresh_tkn: String,
}

#[derive(Deserialize)]
pub struct LoginCred {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignupCred {
    pub nickname: String,
    pub username: String,
    pub password: String,
}

pub async fn login(State(inf): State<AppState>, Json(data): Json<LoginCred>) -> Result<Res<UserTokenObj>, GenericErr> {
    let rows: Vec<User> = inf.ps_interface
    .select(Some((&vec!["username", "password"], &vec![&data.username, &data.password])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    if rows.len() == 1 && 
    let Some(frst) = rows.first() && 
    frst.username == data.username && 
    frst.password == data.password
    {
        let access = create_jwt(frst.id, TokenType::Access, &inf.jwt_secret)
        .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
        let refresh = create_jwt(frst.id, TokenType::Refresh, &inf.jwt_secret)
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
        &Validation::default()).map_err(|_| GenericErr::Auth(AuthError::InvalidToken))?;
    
    if tkn_data.claims.tkn_type != TokenType::Refresh {
        return Err(GenericErr::Auth(AuthError::WrongTokenType));
    }

    let existence_key = format!("{BLACKLIST_STR}{}", tkn_data.claims.jti);
    let is_revoked: i64 = inf.redis_client.exists(existence_key)
    .await.map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    if is_revoked == 1 {
        return Err(GenericErr::Auth(AuthError::ExpiredToken));
    }

    let access = create_jwt(tkn_data.claims.sub, TokenType::Access, &inf.jwt_secret)
    .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
    let refresh = create_jwt(tkn_data.claims.sub, TokenType::Refresh, &inf.jwt_secret)
    .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let set_key = format!("{BLACKLIST_STR}{}", tkn_data.claims.jti);
    let ttl = tkn_data.claims.exp as i64 - OffsetDateTime::now_utc().unix_timestamp() + 60;

    if ttl <= 0 {
        return Err(GenericErr::Auth(AuthError::InvalidToken));
    }
    
    inf.redis_client
    .set::<(), _, _>(set_key, 1, Some(fred::types::Expiration::EX(ttl)), None, false)
    .await.map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    let res = TokenPair{ access_tkn: access, refresh_tkn: refresh };

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(res) });
}

pub async fn sign_up(auth: AuthInvite, inf: State<AppState>, Json(data): Json<SignupCred>) -> Result<Res<UserTokenObj>, GenericErr> {
    let key = format!("{INVITES_BLACKLIST_STR}{}", auth.claims.jti);
    let ttl = auth.claims.exp as i64 - OffsetDateTime::now_utc().unix_timestamp() + 60;

    if ttl <= 0 {
        return Err(GenericErr::Auth(AuthError::InvalidToken));
    }

    if data.nickname.trim().len() < 3 {
        return Err(GenericErr::Auth(AuthError::InvalidNickname));
    }

    inf.redis_client
    .set::<(), _, _>(&key, 1, Some(fred::types::Expiration::EX(ttl)), None, false)
    .await.map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    let usr = inf.ps_interface.insert::<User>(User { 
        id: PLACE_HOLDER_UUID, 
        nickname: data.nickname, 
        prv: auth.claims.priv_level, 
        username: data.username, 
        password: data.password, 
        pfp: None }, false)
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let access: String = create_jwt(usr.id, TokenType::Access, &inf.jwt_secret).await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let refresh: String = create_jwt(usr.id, TokenType::Refresh, &inf.jwt_secret).await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let dta = UserTokenObj {
        user_data: usr,
        token_data: TokenPair { access_tkn: access, refresh_tkn: refresh }
    };

    inf.redis_client.del::<u8, &str>(&key).await.map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    Ok(
        Res{ 
            status: StatusCode::OK, 
            success: true, 
            msg: String::new(), 
            data: Some(dta)
        }
    )
}