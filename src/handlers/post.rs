use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    auth::{creation::create_priv_jwt, extractor::AuthUser},
    handlers::{Msg, fetch_guild_member},
    responses::error_t::{AuthError, GenericErr, InternalError},
    types::*,
};

#[derive(Serialize)]
pub struct InvJwt {
    pub jwt: String,
}

#[derive(Serialize, Deserialize)]
pub struct ObjectIds {
    pub ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct PrivLevel {
    pub prv_level: PrivilegeT,
}

#[derive(Serialize, Deserialize)]
pub struct GrpInv {
    pub user_id: Uuid,
    pub rank: RankT,
}

#[derive(Serialize, Deserialize)]
pub struct Grp {
    pub name: String,
    pub gp: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct Chnl {
    pub name: String,
    pub category: ChannelT,
}

pub async fn add_guild(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Json(data): Json<Grp>,
) -> Result<Res<()>, InternalError> {
    let guild = state
        .pg_interface
        .insert_one(
            Guild {
                id: PLACE_HOLDER_UUID,
                name: data.name,
                gp: data.gp,
            },
            false,
        )
        .await?;

    let member = state
        .pg_interface
        .insert_one(
            Guild_Member {
                guild_id: guild.id,
                member_id: auth.user_id,
                rank: RankT::Owner,
            },
            true,
        )
        .await;

    let channel = state
        .pg_interface
        .insert_one(
            Channel {
                id: PLACE_HOLDER_UUID,
                guild_id: guild.id,
                name: "main".to_string(),
                category: ChannelT::Text,
            },
            false,
        )
        .await;

    if member.is_err() || channel.is_err() {
        state
            .pg_interface
            .delete_many::<Uuid, Guild>(&[guild.id])
            .await?;
        return Err(InternalError::DbError);
    }

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn add_dm(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(member_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    if auth.user_id == member_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    state
        .pg_interface
        .select_one::<Uuid, User>((&["id"], &[member_id]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let existing_dm_query = sqlx::query_as!(
        Dm,
        r#"SELECT * FROM dms 
        WHERE (dms.user_a, dms.user_b) = ($1, $2) OR (dms.user_a, dms.user_b) = ($2, $1)
        LIMIT 1"#,
        auth.user_id,
        member_id
    );

    let existing_invite_query = sqlx::query_as!(
        Dm_Invite,
        r#"SELECT i.* FROM dm_invites i JOIN dms d ON d.id = i.dm_id
        WHERE (i.recipient = $1 AND d.user_a = $2) OR (i.recipient = $2 AND d.user_a = $1) 
        LIMIT 1"#,
        member_id,
        auth.user_id
    );

    let existing_invite = state
        .pg_interface
        .generic_fetch(existing_invite_query)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let existing_dm = state
        .pg_interface
        .generic_fetch(existing_dm_query)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if !existing_dm.is_empty() || !existing_invite.is_empty() {
        return Err(GenericErr::Internal(InternalError::DuplicateData));
    }

    let relation = state
        .pg_interface
        .select_one::<Uuid, Relation>((
            &["relating_user", "related_user"],
            &[member_id, auth.user_id],
        ))
        .await;

    match relation {
        Ok(rel_ok) => {
            if rel_ok.state == RelationT::Blocked {
                return Err(GenericErr::Auth(AuthError::Blocked));
            }
        }
        Err(rel_err) => match rel_err {
            InternalError::NoMatches => {}
            _ => {
                return Err(GenericErr::Internal(rel_err));
            }
        },
    }

    let dm: Dm = state
        .pg_interface
        .insert_one(
            Dm {
                id: PLACE_HOLDER_UUID,
                user_a: Some(auth.user_id),
                user_b: None,
            },
            false,
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let invite = state
        .pg_interface
        .insert_one(
            Dm_Invite {
                id: PLACE_HOLDER_UUID,
                dm_id: dm.id,
                recipient: member_id,
            },
            false,
        )
        .await;

    if invite.is_err() {
        state
            .pg_interface
            .delete_many::<Uuid, Dm>(&[dm.id])
            .await
            .map_err(|e| GenericErr::Internal(e))?;
        return Err(GenericErr::Internal(InternalError::DbError));
    }

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn add_dm_message(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(dm_id): Path<Uuid>,
    Json(data): Json<Msg>,
) -> Result<Res<()>, GenericErr> {
    if data.file_ids.is_empty() && data.message.is_none() {
        return Err(GenericErr::Internal(InternalError::EmptyMessage));
    }

    let dm: Dm = state
        .pg_interface
        .select_one((&["id"], &[dm_id]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if Some(auth.user_id) != dm.user_a && Some(auth.user_id) != dm.user_b {
        return Err(GenericErr::Auth(AuthError::NonOwner));
    }

    let msg: Dm_Message = state
        .pg_interface
        .insert_one(
            Dm_Message {
                id: 0,
                dm_id: dm.id,
                message: data.message,
                user_id: Some(auth.user_id),
            },
            false,
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let msg_objs: Vec<Dm_Message_Object> = data
        .file_ids
        .into_iter()
        .map(|file_id| Dm_Message_Object {
            message_id: msg.id,
            object_id: file_id,
        })
        .collect();

    let msg_objs_insert = state.pg_interface.insert_many(&msg_objs, true).await;

    if msg_objs_insert.is_err() {
        state
            .pg_interface
            .delete_many::<i64, Dm_Message>(&[msg.id])
            .await
            .map_err(|e| GenericErr::Internal(e))?;

        return Err(GenericErr::Internal(InternalError::DbError));
    }

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn add_guild_channel_message(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
    Json(data): Json<Msg>,
) -> Result<Res<()>, InternalError> {
    if data.file_ids.is_empty() && data.message.is_none() {
        return Err(InternalError::EmptyMessage);
    }

    fetch_guild_member(state.clone(), &guild_id, &auth.user_id).await?;

    state
        .pg_interface
        .select_one::<Uuid, Channel>((&["id", "guild_id"], &[channel_id, guild_id]))
        .await?;

    let msg = state
        .pg_interface
        .insert_one(
            Guild_Message {
                id: 0,
                message: data.message,
                channel_id: channel_id,
                user_id: Some(auth.user_id),
            },
            false,
        )
        .await?;

    let msg_objs: Vec<Guild_Message_Object> = data
        .file_ids
        .into_iter()
        .map(|file_id| Guild_Message_Object {
            message_id: msg.id,
            object_id: file_id,
        })
        .collect();

    let msg_objs_insert = state.pg_interface.insert_many(&msg_objs, true).await;

    if msg_objs_insert.is_err() {
        state
            .pg_interface
            .delete_many::<i64, Guild_Message>(&[msg.id])
            .await?;

        return Err(InternalError::DbError);
    }

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn invite_guild_member(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(guild_id): Path<Uuid>,
    Json(data): Json<GrpInv>,
) -> Result<Res<()>, GenericErr> {
    if auth.user_id == data.user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    let caller = fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let recipient = state
        .pg_interface
        .select_many::<Uuid, Guild_Member>(
            Some((&["guild_id", "member_id"], &[guild_id, data.user_id])),
            Some(1),
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next();

    if recipient.is_some() {
        return Err(GenericErr::Internal(InternalError::DuplicateData));
    }

    let relation = state
        .pg_interface
        .select_one::<Uuid, Relation>((
            &["relating_user", "related_user"],
            &[data.user_id, auth.user_id],
        ))
        .await;

    match relation {
        Ok(rel_ok) => {
            if rel_ok.state == RelationT::Blocked {
                return Err(GenericErr::Auth(AuthError::Blocked));
            }
        }
        Err(rel_err) => match rel_err {
            InternalError::NoMatches => {}
            _ => {
                return Err(GenericErr::Internal(rel_err));
            }
        },
    }

    let invite = state
        .pg_interface
        .select_one::<Uuid, Guild_Invite>((&["guild_id", "recipient"], &[guild_id, data.user_id]))
        .await
        .map_err(|e| GenericErr::Internal(e));

    if invite.is_ok() {
        return Err(GenericErr::Internal(InternalError::DuplicateData));
    }

    state
        .pg_interface
        .select_one::<Uuid, User>((&["id"], &[data.user_id]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if caller.rank < data.rank {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    state
        .pg_interface
        .insert_one(
            Guild_Invite {
                id: PLACE_HOLDER_UUID,
                guild_id: guild_id,
                recipient: data.user_id,
                rank: data.rank,
            },
            true,
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn create_friend_invite(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    state
        .pg_interface
        .select_one::<Uuid, User>((&["id"], &[user_id]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let peer_to_caller_relation: Vec<Relation> = state
        .pg_interface
        .select_many(
            Some((&["relating_user", "related_user"], &[user_id, auth.user_id])),
            Some(1),
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let caller_to_peer_relation: Vec<Relation> = state
        .pg_interface
        .select_many(
            Some((&["relating_user", "related_user"], &[auth.user_id, user_id])),
            Some(1),
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if let Some(relation) = peer_to_caller_relation.into_iter().next() {
        match relation.state {
            RelationT::Blocked => {
                return Err(GenericErr::Auth(AuthError::UserUnreachable));
            }
            _ => {
                return Err(GenericErr::Internal(InternalError::DuplicateData));
            }
        }
    }

    if let Some(relation) = caller_to_peer_relation.into_iter().next() {
        match relation.state {
            RelationT::Blocked => {
                return Err(GenericErr::Auth(AuthError::UserUnreachable));
            }
            _ => {
                return Err(GenericErr::Internal(InternalError::DuplicateData));
            }
        }
    }

    state
        .pg_interface
        .insert_one(
            Relation {
                relating_user: auth.user_id,
                related_user: user_id,
                state: RelationT::Pending,
            },
            true,
        )
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn create_invite(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Json(data): Json<PrivLevel>,
) -> Result<Res<InvJwt>, GenericErr> {
    let user: User = state
        .pg_interface
        .select_one((&["id"], &vec![auth.user_id]))
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    if user.prv != PrivilegeT::Proprietor {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let jwt_str = create_priv_jwt(data.prv_level, &state.jwt_secret)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let res: Res<InvJwt> = Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: Some(InvJwt { jwt: jwt_str }),
    };

    return Ok(res);
}

pub async fn add_guild_channel(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(guild_id): Path<Uuid>,
    Json(data): Json<Chnl>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if member.rank < RankT::Admin {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    state
        .pg_interface
        .insert_one(
            Channel {
                id: PLACE_HOLDER_UUID,
                guild_id: guild_id,
                name: data.name,
                category: data.category,
            },
            false,
        )
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn upload(
    _auth: AuthUser,
    State(mut state): State<AppState>,
    mut data: Multipart,
) -> Result<Res<ObjectIds>, InternalError> {
    let match_err = |status: StatusCode, rm: Option<String>| -> InternalError {
        if let Some(rm_path) = rm {
            tokio::spawn(tokio::fs::remove_file(rm_path));
        }

        return match status {
            StatusCode::BAD_REQUEST => InternalError::BadRequest,
            StatusCode::PAYLOAD_TOO_LARGE => InternalError::BodyTooLarge,
            _ => InternalError::DecodeEncodeErr,
        };
    };

    let cleanup_err = |path: String, error: InternalError| -> InternalError {
        tokio::spawn(tokio::fs::remove_file(path));
        error
    };

    let mut objs = ObjectIds { ids: vec![] };

    while let Some(mut field) = data
        .next_field()
        .await
        .map_err(|e| match_err(e.status(), None))?
    {
        let uuid_name = uuid::Uuid::new_v4().to_string();
        let temp_path = TEMP_PTH_STR.to_string() + &uuid_name;
        let mut temp_file = tokio::fs::File::create_new(&temp_path)
            .await
            .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

        let mut hasher = Sha256::new();

        /* ===== LOAD TEMP FILE ===== */
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| match_err(e.status(), Some(temp_path.clone())))?
        {
            hasher.update(&chunk);
            temp_file
                .write_all(&chunk)
                .await
                .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;
        }

        temp_file
            .flush()
            .await
            .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;
        temp_file
            .sync_all()
            .await
            .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

        let hash = hex::encode(hasher.finalize());

        let matches: Option<Object> = state
            .pg_interface
            .select_many::<&str, Object>(Some((&["hash"], &[&hash])), Some(1))
            .await
            .map_err(|_| cleanup_err(temp_path.clone(), InternalError::DbError))?
            .into_iter()
            .next();

        /* ===== CONSUME TEMP FILE ===== */
        let mime = if let Some(typ) = infer::get_from_path(&temp_path)
            .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?
        {
            typ
        } else {
            return Err(cleanup_err(temp_path.clone(), InternalError::UnknownType));
        };

        let (new_obj_path, size) = if let Some(dupe) = &matches {
            tokio::spawn(tokio::fs::remove_file(temp_path.clone()));
            (dupe.rel_path.clone(), dupe.size_bytes)
        } else {
            let size = temp_file
                .metadata()
                .await
                .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?
                .len() as i64;

            let dir_path = OBJ_PTH_STR.to_string() + &hash;
            let full_path = OBJ_PTH_STR.to_string() + &hash + "/" + &uuid_name;

            tokio::fs::create_dir_all(&dir_path)
                .await
                .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

            tokio::fs::rename(&temp_path, full_path)
                .await
                .map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

            (hash.clone() + "/" + &uuid_name, size)
        };

        let new_obj = Object {
            id: PLACE_HOLDER_UUID,
            hash: hash,
            rel_path: new_obj_path.clone(),
            mime_type: mime.mime_type().to_string(),
            size_bytes: size,
            creation_timestamp: OffsetDateTime::now_utc().unix_timestamp() as i64,
        };

        let obj: Object = state
            .pg_interface
            .insert_one(new_obj, false)
            .await
            .map_err(|_| {
                if matches.is_none() {
                    tokio::spawn(tokio::fs::remove_file(
                        OBJ_PTH_STR.to_string() + &new_obj_path,
                    ));
                }
                InternalError::DbError
            })?;

        objs.ids.push(obj.id);
    }

    return Ok(Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: Some(objs),
    });
}
