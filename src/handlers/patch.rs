use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::handlers::post::Grp;
use crate::handlers::{Msg, fetch_guild_member};
use crate::{
    auth::extractor::AuthUser,
    responses::error_t::{AuthError, GenericErr, InternalError},
    types::*,
};

#[derive(Serialize, Deserialize)]
pub struct InvMng {
    pub accept: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ChannelUpdate {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserUpdate {
    pub nickname: String,
    pub username: String,
    pub password: String,
    pub pfp: Option<Uuid>,
}

pub async fn update_self(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Json(data): Json<UserUpdate>,
) -> Result<Res<User>, GenericErr> {
    if data.nickname.trim().len() < 3 {
        return Err(GenericErr::Auth(AuthError::InvalidNickname));
    }

    if data.username.trim().len() < 3 || data.password.trim().len() < 3 {
        return Err(GenericErr::Internal(
            InternalError::InvalidAccountCredentials,
        ));
    }

    let old_user: User = state
        .pg_interface
        .select_one((&["id"], &[auth.user_id]))
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    state
        .pg_interface
        .update_one(User {
            id: auth.user_id,
            nickname: data.nickname,
            prv: old_user.prv,
            username: data.username,
            password: data.password,
            pfp: data.pfp,
        })
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn update_guild(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(guild_id): Path<Uuid>,
    Json(data): Json<Grp>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if member.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    state
        .pg_interface
        .update_one(Guild {
            id: guild_id,
            name: data.name,
            gp: data.gp,
        })
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn update_relation(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Res<()>, GenericErr> {
    const SUCCESS_RESPONSE: Result<Res<()>, GenericErr> = Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });

    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    let current_relation: Result<Relation, InternalError> = state
        .pg_interface
        .select_one((&["relating_user", "related_user"], &[auth.user_id, user_id]))
        .await;
    let mut active_relation: Relation;

    match current_relation {
        Err(e) if e == InternalError::NoMatches => {
            state
                .pg_interface
                .insert_one(
                    Relation {
                        relating_user: auth.user_id,
                        related_user: user_id,
                        state: RelationT::Blocked,
                    },
                    true,
                )
                .await
                .map_err(|e| GenericErr::Internal(e))?;
            return SUCCESS_RESPONSE;
        }
        Err(e) => {
            return Err(GenericErr::Internal(e));
        }

        Ok(k) => active_relation = k,
    }

    active_relation.state = match active_relation.state {
        RelationT::Blocked => {
            let reverse_relation: Result<Relation, InternalError> = state
                .pg_interface
                .select_one((&["relating_user", "related_user"], &[user_id, auth.user_id]))
                .await;
            match reverse_relation {
                Err(e) if e == InternalError::NoMatches => {
                    state
                        .pg_interface
                        .delete_many::<Uuid, Relation>(&[auth.user_id, user_id])
                        .await
                        .map_err(|e| GenericErr::Internal(e))?;
                    return SUCCESS_RESPONSE;
                }

                Err(e) => {
                    return Err(GenericErr::Internal(e));
                }

                Ok(k) if k.state == RelationT::Pending || k.state == RelationT::Blocked => {
                    return Err(GenericErr::Auth(AuthError::NonOwner));
                }

                _ => {}
            }
            RelationT::Friends
        }
        _ => RelationT::Blocked,
    };

    state
        .pg_interface
        .update_one(active_relation)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return SUCCESS_RESPONSE;
}

pub async fn manage_guild_invite(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(invite_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let invite: Guild_Invite = state
        .pg_interface
        .select_one((&["id"], &[invite_id]))
        .await?;

    if invite.recipient != auth.user_id {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        state
            .pg_interface
            .insert_one::<Guild_Member>(
                Guild_Member {
                    guild_id: invite.guild_id,
                    member_id: invite.recipient,
                    rank: invite.rank,
                },
                true,
            )
            .await?;
    }

    state
        .pg_interface
        .delete_many::<Uuid, Guild_Invite>(&[invite.id])
        .await?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn manage_dm_invite(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(invite_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let invite: Dm_Invite = state
        .pg_interface
        .select_one((&["id"], &[invite_id]))
        .await?;

    if invite.recipient != auth.user_id {
        return Err(InternalError::NoMatches);
    }

    let dm: Dm = state
        .pg_interface
        .select_one((&["id"], &[invite.dm_id]))
        .await?;

    let existing_user = if let Some(user) = dm.user_a {
        user
    } else if let Some(user) = dm.user_b {
        user
    } else {
        return Err(InternalError::NoMatches);
    };

    let (user_a, user_b) = if existing_user < invite.recipient {
        (Some(existing_user), Some(invite.recipient))
    } else {
        (Some(invite.recipient), Some(existing_user))
    };

    if data.accept {
        state
            .pg_interface
            .update_one(Dm {
                id: dm.id,
                user_a: user_a,
                user_b: user_b,
            })
            .await?;
    }

    state
        .pg_interface
        .delete_many::<Uuid, Dm_Invite>(&[invite.id])
        .await?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn manage_friend_invite(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let mut inv: Relation = state
        .pg_interface
        .select_one((&["relating_user", "related_user"], &[user_id, auth.user_id]))
        .await?;

    if inv.state != RelationT::Pending {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        let mut tr = state.pg_interface.begin_transaction().await?;
        inv.state = RelationT::Friends;

        tr
            .update_one(inv)
            .await
            .map_err(|_| InternalError::DbError)?;

        tr
            .insert_one(
                Relation {
                    relating_user: auth.user_id,
                    related_user: user_id,
                    state: RelationT::Friends,
                },
                true,
            )
            .await
            .map_err(|_| InternalError::DbError)?;

        tr.commit().await?;
    } else {
        state
            .pg_interface
            .delete_many::<Uuid, Relation>(&[user_id, auth.user_id])
            .await
            .map_err(|_| InternalError::DbError)?;
    }

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn update_guild_channel_message(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path((guild_id, channel_id, message_id)): Path<(Uuid, Uuid, i64)>,
    Json(data): Json<Msg>,
) -> Result<Res<()>, GenericErr> {
    if data.message.is_none() && data.file_ids.is_empty() {
        return Err(GenericErr::Internal(InternalError::EmptyMessage));
    }

    fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let channel: Channel = state
        .pg_interface
        .select_one((&["id", "guild_id"], &[channel_id, guild_id]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let messages_query = sqlx::query_as!(
        Guild_Message,
        "SELECT * FROM guild_messages WHERE guild_messages.id = $1 AND guild_messages.channel_id = $2 LIMIT 1",
        message_id,
        &channel.id
    );

    let messages: Guild_Message = state
        .pg_interface
        .generic_fetch(messages_query)
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    match messages.user_id {
        Some(msg_id) => {
            if msg_id != auth.user_id {
                return Err(GenericErr::Auth(AuthError::NonOwner));
            }
        }
        None => {
            return Err(GenericErr::Internal(InternalError::DetachedOwner));
        }
    }

    state
        .pg_interface
        .update_one::<Guild_Message>(Guild_Message {
            id: messages.id,
            channel_id: messages.channel_id,
            message: data.message,
            user_id: messages.user_id,
        })
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let old_objs_hash: HashSet<Uuid> = state
        .pg_interface
        .select_many(Some((&["message_id"], &[messages.id])), None)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?
        .into_iter()
        .map(|m: Guild_Message_Object| m.object_id)
        .collect();

    let new_objs_hash: HashSet<Uuid> = data.file_ids.into_iter().collect();

    let insert_objs: Vec<Guild_Message_Object> = new_objs_hash
        .difference(&old_objs_hash)
        .map(|obj_id| Guild_Message_Object {
            message_id: messages.id,
            object_id: *obj_id,
        })
        .collect();

    let delete_objs: Vec<Uuid> = old_objs_hash.difference(&new_objs_hash).copied().collect();

    state
        .pg_interface
        .insert_many(&insert_objs, true)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let delete_query = sqlx::query!(
        r#"DELETE FROM guild_message_objects WHERE message_id = $1 AND object_id = ANY($2);"#,
        message_id,
        delete_objs.as_slice()
    );

    state
        .pg_interface
        .generic_exec(delete_query)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn update_dm_message(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path((dm_id, message_id)): Path<(Uuid, i64)>,
    Json(data): Json<Msg>,
) -> Result<Res<()>, GenericErr> {
    if data.message.is_none() && data.file_ids.is_empty() {
        return Err(GenericErr::Internal(InternalError::EmptyMessage));
    }

    state
        .pg_interface
        .select_one::<Uuid, Dm>((&["id"], &[dm_id]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let messages_query = sqlx::query_as!(
        Dm_Message,
        "SELECT * FROM dm_messages WHERE id = $1 AND dm_id = $2 LIMIT 1",
        message_id,
        dm_id
    );

    let message: Dm_Message = state
        .pg_interface
        .generic_fetch(messages_query)
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    match message.user_id {
        Some(msg_id) => {
            if msg_id != auth.user_id {
                return Err(GenericErr::Auth(AuthError::NonOwner));
            }
        }
        None => {
            return Err(GenericErr::Internal(InternalError::DetachedOwner));
        }
    }

    state
        .pg_interface
        .update_one::<Dm_Message>(Dm_Message {
            id: message.id,
            dm_id: message.dm_id,
            message: data.message,
            user_id: message.user_id,
        })
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let old_objs_hash: HashSet<Uuid> = state
        .pg_interface
        .select_many(Some((&["message_id"], &[message.id])), None)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?
        .into_iter()
        .map(|m: Dm_Message_Object| m.object_id)
        .collect();

    let new_objs_hash: HashSet<Uuid> = data.file_ids.into_iter().collect();

    let insert_objs: Vec<Dm_Message_Object> = new_objs_hash
        .difference(&old_objs_hash)
        .map(|obj_id| Dm_Message_Object {
            message_id: message.id,
            object_id: *obj_id,
        })
        .collect();

    let delete_objs: Vec<Uuid> = old_objs_hash.difference(&new_objs_hash).copied().collect();

    state
        .pg_interface
        .insert_many(&insert_objs, true)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let delete_query = sqlx::query!(
        "DELETE FROM dm_message_objects WHERE message_id = $1 AND object_id = ANY($2);",
        message.id,
        delete_objs.as_slice()
    );

    state
        .pg_interface
        .generic_exec(delete_query)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn update_channel(
    auth: AuthUser,
    State(mut state): State<AppState>,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
    Json(data): Json<ChannelUpdate>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if member.rank < RankT::Admin {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let channel: Channel = state
        .pg_interface
        .select_one((&["id", "guild_id"], &[channel_id, guild_id]))
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    state
        .pg_interface
        .update_one(Channel {
            id: channel.id,
            guild_id: guild_id,
            name: data.name,
            category: channel.category,
        })
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}
