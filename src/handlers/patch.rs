use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::handlers::{Msg, StrippedGroup, fetch_guild_member};
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
pub struct StatChange {
    pub state: RelationT,
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
    State(state): State<AppState>,
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
        .ps_interface
        .select(Some((&["id"], &[auth.user_id])), None)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    state
        .ps_interface
        .update(User {
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
    State(state): State<AppState>,
    Path(guild_id): Path<Uuid>,
    Json(data): Json<StrippedGroup>,
) -> Result<Res<()>, GenericErr> {
    let member = fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    if member.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    state
        .ps_interface
        .update(Guild {
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
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<StatChange>,
) -> Result<Res<()>, GenericErr> {
    if auth.user_id == user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    let rels: Vec<Relation> = state
        .ps_interface
        .select(
            Some((&["relating_user", "related_user"], &[auth.user_id, user_id])),
            None,
        )
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let mut rel = rels
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    rel.state = data.state;

    state
        .ps_interface
        .update(rel)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: None,
    });
}

pub async fn manage_guild_invite(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(invite_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let invs: Vec<Guild_Invite> = state
        .ps_interface
        .select(Some((&["id"], &[invite_id])), None)
        .await?;

    let inv: Guild_Invite = invs.into_iter().next().ok_or(InternalError::NoMatches)?;

    if inv.recipient != auth.user_id {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        state
            .ps_interface
            .insert::<Guild_Member>(
                &[Guild_Member {
                    guild_id: inv.guild_id,
                    member_id: inv.recipient,
                    rank: inv.rank,
                }],
                true,
            )
            .await?;
    }

    state
        .ps_interface
        .delete::<Uuid, Guild_Invite>(&[inv.id])
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
    State(state): State<AppState>,
    Path(invite_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let inv: Dm_Invite = state
        .ps_interface
        .select(Some((&["id"], &[invite_id])), None)
        .await?
        .into_iter()
        .next()
        .ok_or(InternalError::NoMatches)?;

    if inv.recipient != auth.user_id {
        return Err(InternalError::NoMatches);
    }

    let dm: Dm = state
        .ps_interface
        .select(Some((&["id"], &[inv.dm_id])), Some(1))
        .await?
        .into_iter()
        .next()
        .ok_or(InternalError::NoMatches)?;

    let existing_user = if let Some(user) = dm.user_a {
        user
    } else if let Some(user) = dm.user_b {
        user
    } else {
        return Err(InternalError::NoMatches);
    };

    if data.accept {
        state
            .ps_interface
            .update(Dm {
                id: dm.id,
                user_a: Some(existing_user),
                user_b: Some(inv.recipient),
            })
            .await?;
    }

    state
        .ps_interface
        .delete::<Uuid, Dm_Invite>(&[inv.id])
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
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(data): Json<InvMng>,
) -> Result<Res<()>, InternalError> {
    let invs: Vec<Relation> = state
        .ps_interface
        .select(
            Some((&["relating_user", "related_user"], &[user_id, auth.user_id])),
            None,
        )
        .await
        .map_err(|_| InternalError::DbError)?;

    let mut inv: Relation = invs.into_iter().next().ok_or(InternalError::NoMatches)?;

    if inv.state != RelationT::Pending {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        inv.state = RelationT::Friends;
        state
            .ps_interface
            .update(inv)
            .await
            .map_err(|_| InternalError::DbError)?;
        state
            .ps_interface
            .insert(
                &[Relation {
                    relating_user: auth.user_id,
                    related_user: user_id,
                    state: RelationT::Friends,
                }],
                true,
            )
            .await
            .map_err(|_| InternalError::DbError)?;
    } else {
        state
            .ps_interface
            .delete::<Uuid, Relation>(&[auth.user_id, user_id])
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
    State(state): State<AppState>,
    Path((guild_id, channel_id, message_id)): Path<(Uuid, Uuid, i64)>,
    Json(data): Json<Msg>,
) -> Result<Res<()>, GenericErr> {
    fetch_guild_member(state.clone(), &guild_id, &auth.user_id)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let messages_query = sqlx::query_as!(Guild_Message, 
        "SELECT * FROM guild_messages WHERE guild_messages.id = $1 AND guild_messages.channel_id = $2 LIMIT 1",
        message_id,
        channel_id
    );

    let messages: Guild_Message = state
        .ps_interface
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
        .ps_interface
        .update::<Guild_Message>(Guild_Message {
            id: messages.id,
            channel_id: messages.channel_id,
            message: data.message,
            user_id: messages.user_id,
        })
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let old_objs_hash: HashSet<Uuid> = state
        .ps_interface
        .select(Some((&["message_id"], &[messages.id])), None)
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
        .ps_interface
        .insert(&insert_objs, true)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let delete_query = sqlx::query!( 
        r#"DELETE FROM guild_message_objects WHERE message_id = $1 AND object_id = ANY($2);"#,
        message_id,
        delete_objs.as_slice()
    );

    state
        .ps_interface
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
    State(state): State<AppState>,
    Path((dm_id, message_id)): Path<(Uuid, i64)>,
    Json(data): Json<Msg>,
) -> Result<Res<()>, GenericErr> {
    state.ps_interface.select::<Uuid, Dm>(Some((&["id"], &[dm_id])), Some(1))
    .await.map_err(|e| GenericErr::Internal(e))?
    .into_iter()
    .next()
    .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    let messages_query = sqlx::query_as!(Dm_Message, 
        "SELECT * FROM dm_messages WHERE id = $1 AND dm_id = $2 LIMIT 1",
        message_id,
        dm_id
    );

    let messages: Dm_Message = state
        .ps_interface
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
        .ps_interface
        .update::<Dm_Message>(Dm_Message {
            id: messages.id,
            dm_id: messages.dm_id,
            message: data.message,
            user_id: messages.user_id,
        })
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let old_objs_hash: HashSet<Uuid> = state
        .ps_interface
        .select(Some((&["message_id"], &[messages.id])), None)
        .await
        .map_err(|_| GenericErr::Internal(InternalError::DbError))?
        .into_iter()
        .map(|m: Dm_Message_Object| m.object_id)
        .collect();

    let new_objs_hash: HashSet<Uuid> = data.file_ids.into_iter().collect();

    let insert_objs: Vec<Dm_Message_Object> = new_objs_hash
        .difference(&old_objs_hash)
        .map(|obj_id| Dm_Message_Object {
            message_id: messages.id,
            object_id: *obj_id,
        })
        .collect();

    let delete_objs: Vec<Uuid> = old_objs_hash.difference(&new_objs_hash).copied().collect();

    state
        .ps_interface
        .insert(&insert_objs, true)
        .await
        .map_err(|e| GenericErr::Internal(e))?;

    let delete_query = sqlx::query!("DELETE FROM dm_message_objects WHERE message_id = $1 AND object_id = ANY($2);",
        messages.id,
        delete_objs.as_slice()
    );

    state
        .ps_interface
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

pub async fn update_channel(
    auth: AuthUser,
    State(state): State<AppState>,
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
        .ps_interface
        .select(Some((&["id"], &[channel_id])), None)
        .await
        .map_err(|e| GenericErr::Internal(e))?
        .into_iter()
        .next()
        .ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    state
        .ps_interface
        .update(Channel {
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
