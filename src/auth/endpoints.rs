use axum::{Json, extract::State, http::StatusCode};
use fred::interfaces::KeysInterface;
use fred::types;
use jsonwebtoken::{DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::auth::extractor::AuthInvite;
use crate::types::*;
use crate::{auth::creation::create_jwt, responses::error_t::*};

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

pub async fn login(
    State(state): State<AppState>,
    Json(data): Json<LoginCred>,
) -> Result<Res<UserTokenObj>, GenericErr> {
    let user: User = state
        .ps_interface
        .select_one((
            &vec!["username", "password"],
            &vec![&data.username, &data.password],
        ))
        .await
        .map_err(|_| GenericErr::Auth(AuthError::InvalidCredentials))?;

    if user.username == data.username && user.password == data.password {
        let access = create_jwt(user.id, TokenType::Access, &state.jwt_secret)
            .await
            .map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
        let refresh = create_jwt(user.id, TokenType::Refresh, &state.jwt_secret)
            .await
            .map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

        let dta = UserTokenObj {
            user_data: user,
            token_data: TokenPair {
                access_tkn: access,
                refresh_tkn: refresh,
            },
        };

        return Ok(Res {
            status: StatusCode::OK,
            success: true,
            msg: String::new(),
            data: Some(dta),
        });
    } else {
        return Err(GenericErr::Auth(AuthError::InvalidCredentials));
    }
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(data): Json<RefreshBody>,
) -> Result<Res<TokenPair>, GenericErr> {
    let tkn_data = decode::<Claims>(
        &data.refresh_tkn,
        &DecodingKey::from_secret(&state.jwt_secret),
        &Validation::default(),
    )
    .map_err(|_| GenericErr::Auth(AuthError::InvalidToken))?;

    if tkn_data.claims.tkn_type != TokenType::Refresh {
        return Err(GenericErr::Auth(AuthError::WrongTokenType));
    }

    let existence_key = format!("{BLACKLIST_STR}{}", tkn_data.claims.jti);
    let is_revoked: i64 = state
        .redis_client
        .exists(existence_key)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    if is_revoked == 1 {
        return Err(GenericErr::Auth(AuthError::ExpiredToken));
    }

    state
        .ps_interface
        .select_one::<Uuid, User>((&["id"], &[tkn_data.claims.sub]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let access = create_jwt(tkn_data.claims.sub, TokenType::Access, &state.jwt_secret)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;
    let refresh = create_jwt(tkn_data.claims.sub, TokenType::Refresh, &state.jwt_secret)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let set_key = format!("{BLACKLIST_STR}{}", tkn_data.claims.jti);
    let ttl = tkn_data.claims.exp as i64 - OffsetDateTime::now_utc().unix_timestamp() + 60;

    if ttl <= 0 {
        return Err(GenericErr::Auth(AuthError::InvalidToken));
    }

    state
        .redis_client
        .set::<(), _, _>(
            set_key,
            1,
            Some(fred::types::Expiration::EX(ttl)),
            None,
            false,
        )
        .await
        .map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    let res = TokenPair {
        access_tkn: access,
        refresh_tkn: refresh,
    };

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: Some(res),
    });
}

pub async fn sign_up(
    auth: AuthInvite,
    state: State<AppState>,
    Json(data): Json<SignupCred>,
) -> Result<Res<UserTokenObj>, GenericErr> {
    let key = format!("{INVITES_BLACKLIST_STR}{}", auth.claims.jti);

    if data.nickname.trim().len() < 3 {
        return Err(GenericErr::Auth(AuthError::InvalidNickname));
    }

    if data.username.trim().len() < 3 || data.password.trim().len() < 3 {
        return Err(GenericErr::Internal(
            InternalError::InvalidAccountCredentials,
        ));
    }

    let user: User = state
        .ps_interface
        .insert_one(
            User {
                id: PLACE_HOLDER_UUID,
                nickname: data.nickname,
                prv: auth.claims.priv_level,
                username: data.username,
                password: data.password,
                pfp: None,
            },
            false,
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let access: String = create_jwt(user.id, TokenType::Access, &state.jwt_secret)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let refresh: String = create_jwt(user.id, TokenType::Refresh, &state.jwt_secret)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let res = UserTokenObj {
        user_data: user,
        token_data: TokenPair {
            access_tkn: access,
            refresh_tkn: refresh,
        },
    };

    let ttl = auth.claims.exp as i64 - OffsetDateTime::now_utc().unix_timestamp() + 60;

    if ttl <= 0 {
        return Err(GenericErr::Auth(AuthError::InvalidToken));
    }

    state
        .redis_client
        .set::<(), &String, i8>(
            &key,
            1,
            Some(fred::types::Expiration::EX(ttl)),
            Some(fred::types::SetOptions::NX),
            false,
        )
        .await
        .map_err(|_| GenericErr::Internal(InternalError::RedisError))?;

    Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: Some(res),
    })
}
