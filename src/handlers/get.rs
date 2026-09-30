use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use base64::Engine;
use sqlx::Postgres;
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, responses::error_t::InternalError};
use crate::{handlers::*, types::*};

#[derive(FromRow)]
pub struct FullMsg {
    pub id: i32,
    pub file_ids: Vec<Uuid>,
    pub message: Option<String>,
    pub group_id: Uuid,
    pub user_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct StrippedChannel {
    pub id: Uuid,
    pub name: String,
    pub category: ChannelT,
}

#[derive(Serialize)]
pub struct Page {
    pub file_ids: Vec<Uuid>,
    pub message: Option<String>,
    pub group_id: Uuid,
    pub user_id: Option<Uuid>,
}

#[derive(Serialize, FromRow)]
pub struct MsgResponse {
    pub cursor: String,
    pub pages: Vec<Page>,
}

#[derive(Serialize)]
pub struct DmInvitesResponse {
    pub cursor: Option<Uuid>,
    pub pages: Vec<Dm_Invite>,
}

#[derive(Serialize)]
pub struct ChannelsResponse {
    pub cursor: Option<Uuid>,
    pub pages: Vec<StrippedChannel>,
}

#[derive(Serialize)]
pub struct GuildsResponse {
    pub cursor: Option<Uuid>,
    pub pages: Vec<Guild>,
}

#[derive(Serialize)]
pub struct GuildInvitesResponse {
    pub cursor: Option<Uuid>,
    pub pages: Vec<Guild_Invite>,
}

#[derive(Serialize)]
pub struct GuildMembersResponse {
    pub cursor: Option<Uuid>,
    pub pages: Vec<StrippedMember>,
}

#[derive(Serialize)]
pub struct StrippedUserResponse {
    pub cursor: Option<Uuid>,
    pub pages: Vec<StrippedUser>,
}

pub type RelationInvitesResponse = StrippedUserResponse;
pub type RelationsResponse = StrippedUserResponse;

/* ===== Handlers ===== */

pub async fn get_inital_dm_invites(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Res<DmInvitesResponse>, InternalError> {
    return get_dm_invites(auth.user_id, None, state).await;
}

pub async fn get_dm_invites_from(
    auth: AuthUser,
    Path(cursor): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<DmInvitesResponse>, InternalError> {
    return get_dm_invites(auth.user_id, Some(cursor), state).await;
}

pub async fn get_inital_channels(
    _auth: AuthUser,
    Path(group_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<ChannelsResponse>, InternalError> {
    return get_channels(group_id, None, state).await;
}

pub async fn get_channels_from(
    _auth: AuthUser,
    Path((group_id, cursor)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Res<ChannelsResponse>, InternalError> {
    return get_channels(group_id, Some(cursor), state).await;
}

pub async fn get_inital_guilds(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Res<GuildsResponse>, InternalError> {
    return get_guilds(auth.user_id, None, state).await;
}

pub async fn get_guilds_from(
    auth: AuthUser,
    Path(cursor): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<GuildsResponse>, InternalError> {
    return get_guilds(auth.user_id, Some(cursor), state).await;
}

pub async fn get_inital_guild_invites(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Res<GuildInvitesResponse>, InternalError> {
    return get_guild_invites(auth.user_id, None, state).await;
}

pub async fn get_guild_invites_from(
    auth: AuthUser,
    Path(cursor): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<GuildInvitesResponse>, InternalError> {
    return get_guild_invites(auth.user_id, Some(cursor), state).await;
}

pub async fn get_inital_guild_members(
    _auth: AuthUser,
    Path(group_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<GuildMembersResponse>, InternalError> {
    return get_guild_members(group_id, None, state).await;
}

pub async fn get_guild_members_from(
    _auth: AuthUser,
    Path((group_id, cursor)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Res<GuildMembersResponse>, InternalError> {
    return get_guild_members(group_id, Some(cursor), state).await;
}

pub async fn get_inital_friend_invites(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Res<RelationInvitesResponse>, InternalError> {
    return get_friend_invites(auth.user_id, None, state).await;
}

pub async fn get_friend_invites_from(
    auth: AuthUser,
    Path(cursor): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<RelationInvitesResponse>, InternalError> {
    return get_friend_invites(auth.user_id, Some(cursor), state).await;
}

pub async fn get_inital_friends(
    auth: AuthUser,
    State(state): State<AppState>,
) -> Result<Res<RelationsResponse>, InternalError> {
    return get_friends(auth.user_id, None, state).await;
}

pub async fn get_friends_from(
    auth: AuthUser,
    Path(cursor): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<RelationsResponse>, InternalError> {
    return get_friends(auth.user_id, Some(cursor), state).await;
}

pub async fn pull(
    _auth: AuthUser,
    State(inf): State<AppState>,
    Path(object_id): Path<Uuid>,
) -> Result<impl IntoResponse, InternalError> {
    let obj: Object = inf
        .ps_interface
        .select::<Uuid, Object>(Some((Object::id_columns(), &[object_id])), None)
        .await
        .map_err(|_| InternalError::DbError)?
        .into_iter()
        .next()
        .ok_or(InternalError::NoMatches)?;

    Ok(Response::builder()
        .header("X-Accel-Redirect", format!("/files/objs/{}", obj.rel_path))
        .header("Content-Type", obj.mime_type)
        .body(Body::empty())
        .map_err(|_| InternalError::OperationsError)?)
}

/* ===== Functions ===== */

async fn get_friends(
    user_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<RelationsResponse>, InternalError> {
    let friends: Vec<StrippedUser> = if let Some(pin) = cursor {
        let sql: &'static str = "SELECT AS user_id, AS nickname, AS pfp FROM users 
        JOIN relations ON relations.related_user = users.id 
        WHERE (relations.relating_user, relations.state) = ($1, $2) AND $3 > users.id
        ORDER BY users.id DESC
        LIMIT 50;";

        let query: QueryAs<'_, Postgres, StrippedUser, PgArguments> = sqlx::query_as(sql)
            .bind(user_id)
            .bind(RelationT::Friends)
            .bind(pin);

        state.ps_interface.generic_fetch(query).await?
    } else {
        let sql: &'static str = "SELECT AS user_id, AS nickname, AS pfp FROM users 
        JOIN relations ON relations.related_user = users.id 
        WHERE (relations.relating_user, relations.state) = ($1, $2)
        ORDER BY users.id DESC
        LIMIT 50;";

        let query: QueryAs<'_, Postgres, StrippedUser, PgArguments> =
            sqlx::query_as(sql).bind(user_id).bind(RelationT::Friends);

        state.ps_interface.generic_fetch(query).await?
    };

    if friends.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor: Option<Uuid> = if friends.len() == 50
        && let Some(pin) = friends.iter().next_back()
    {
        Some(pin.user_id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::FOUND,
        success: true,
        msg: String::new(),
        data: Some(RelationsResponse {
            cursor: cursor,
            pages: friends,
        }),
    });
}

async fn get_guilds(
    user_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<GuildsResponse>, InternalError> {
    let query: QueryAs<'_, Postgres, Guild, PgArguments> = if let Some(pin) = cursor {
        let sql: &'static str = "SELECT guilds.*
        FROM guilds
        WHERE EXISTS (
            SELECT 1
            FROM group_members
            WHERE group_members.group_id = guilds.group_id
              AND group_members.member_id = $1
          )
          AND $2 > guilds.id
        ORDER BY guilds.id DESC
        LIMIT 50;";

        sqlx::query_as(sql).bind(user_id).bind(pin)
    } else {
        let sql: &'static str = "SELECT guilds.*
        FROM guilds
        WHERE EXISTS (
            SELECT 1
            FROM group_members
            WHERE group_members.group_id = guilds.group_id
              AND group_members.member_id = $1
          )
        ORDER BY guilds.id DESC
        LIMIT 50;";

        sqlx::query_as(sql).bind(user_id)
    };

    let guilds = state.ps_interface.generic_fetch(query).await?;

    if guilds.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor: Option<Uuid> = if guilds.len() == 50
        && let Some(pin) = guilds.iter().next_back()
    {
        Some(pin.id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::FOUND,
        success: true,
        msg: String::new(),
        data: Some(GuildsResponse {
            cursor: cursor,
            pages: guilds,
        }),
    });
}

async fn get_channels(
    group_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<ChannelsResponse>, InternalError> {
    let channels: Vec<StrippedChannel> = if let Some(pin) = cursor {
        let sql: &'static str =
            "SELECT * FROM channels WHERE $1 > channels.id ORDER BY channels.id DESC LIMIT 50;";
        let query: QueryAs<'_, Postgres, Channel, PgArguments> = sqlx::query_as(sql).bind(pin);
        state
            .ps_interface
            .generic_fetch(query)
            .await?
            .into_iter()
            .map(|chnl| StrippedChannel {
                id: chnl.id,
                name: chnl.name,
                category: chnl.category,
            })
            .collect()
    } else {
        state
            .ps_interface
            .select::<Uuid, Channel>(Some((&["group_id"], &[group_id])), Some(50))
            .await?
            .into_iter()
            .map(|chnl| StrippedChannel {
                id: chnl.id,
                name: chnl.name,
                category: chnl.category,
            })
            .collect()
    };

    if channels.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor: Option<Uuid> = if channels.len() == 50
        && let Some(back) = channels.iter().next_back()
    {
        Some(back.id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: Some(ChannelsResponse {
            cursor: cursor,
            pages: channels,
        }),
    });
}

async fn get_guild_members(
    group_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<GuildMembersResponse>, InternalError> {
    fetch_group(state.clone(), &group_id).await?;

    let members: Vec<StrippedMember> = if let Some(pin) = cursor {
        let sql: &'static str =
            "SELECT users.id AS user_id, users.nickname AS nickname, users.pfp AS pfp, group_members.rank AS rank FROM users 
            JOIN group_members ON group_members.member_id = users.id 
            WHERE group_members.group_id = $1 AND $2 > users.id
            ORDER BY users.id DESC
            LIMIT 50;";
        let query: QueryAs<'_, Postgres, StrippedMember, PgArguments> =
            sqlx::query_as(sql).bind(group_id).bind(pin);
        state.ps_interface.generic_fetch(query).await?
    } else {
        let sql: &'static str =
            "SELECT users.id AS user_id, users.nickname AS nickname, users.pfp AS pfp, group_members.rank AS rank FROM users 
            JOIN group_members ON group_members.member_id = users.id 
            WHERE group_members.group_id = $1
            ORDER BY users.id DESC
            LIMIT 50;";
        let query: QueryAs<'_, Postgres, StrippedMember, PgArguments> =
            sqlx::query_as(sql).bind(group_id);
        state.ps_interface.generic_fetch(query).await?
    };

    if members.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor: Option<Uuid> = if members.len() == 50
        && let Some(pin) = members.iter().next_back()
    {
        Some(pin.user_id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: Some(GuildMembersResponse {
            cursor: cursor,
            pages: members,
        }),
    });
}

pub async fn get_messages(
    auth: AuthUser,
    State(inf): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> Result<Res<MsgResponse>, InternalError> {
    let sql: &'static str = "SELECT 
        messages.id AS id, 
        COALESCE(array_agg(message_objects.object_id) 
        FILTER (WHERE message_objects.object_id IS NOT NULL), '{}'::uuid[]) AS file_ids, 
        messages.message AS message, 
        messages.group_id AS group_id, 
        messages.user_id AS user_id 
    FROM messages 
    LEFT JOIN message_objects ON message_objects.message_id = messages.id
    WHERE messages.group_id = $1
    GROUP BY messages.id
    ORDER BY messages.id DESC
    LIMIT 50;";

    fetch_group_member(inf.clone(), &group_id, &auth.user_id).await?;

    let query: QueryAs<'_, Postgres, FullMsg, PgArguments> = sqlx::query_as(sql).bind(group_id);

    let messages = inf
        .ps_interface
        .generic_fetch(query)
        .await
        .map_err(|_| InternalError::DbError)?;

    if let Some(oldest) = messages.last() {
        let bytes = oldest.id.to_be_bytes();
        let cursor = ENCODE_DECODE_ENGINE.encode(bytes);

        let res = MsgResponse {
            cursor: cursor,
            pages: messages
                .into_iter()
                .map(
                    |FullMsg {
                         id: _,
                         file_ids,
                         message,
                         group_id,
                         user_id,
                     }| Page {
                        file_ids,
                        message,
                        group_id,
                        user_id,
                    },
                )
                .collect(),
        };

        return Ok(Res {
            status: StatusCode::FOUND,
            success: true,
            msg: String::new(),
            data: Some(res),
        });
    } else {
        return Err(InternalError::NoMatches);
    }
}

async fn get_guild_invites(
    user_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<GuildInvitesResponse>, InternalError> {
    let invites: Vec<Guild_Invite> = if let Some(pin) = cursor {
        let sql: &'static str = "SELECT * FROM guild_invites WHERE $1 > guild_invites.id ORDER BY guild_invites.id DESC LIMIT 50;";
        let query: QueryAs<'_, Postgres, Guild_Invite, PgArguments> = sqlx::query_as(sql).bind(pin);
        state.ps_interface.generic_fetch(query).await?
    } else {
        state
            .ps_interface
            .select(Some((&["user_id"], &[user_id])), Some(50))
            .await?
    };

    if invites.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor: Option<Uuid> = if invites.len() == 50
        && let Some(pin) = invites.iter().next_back()
    {
        Some(pin.id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::FOUND,
        success: true,
        msg: String::new(),
        data: Some(GuildInvitesResponse {
            cursor: cursor,
            pages: invites,
        }),
    });
}

async fn get_dm_invites(
    user_id: Uuid,
    cursor: Option<Uuid>,
    inf: AppState,
) -> Result<Res<DmInvitesResponse>, InternalError> {
    let invites: Vec<Dm_Invite> = if let Some(pin) = cursor {
        let sql: &'static str = "SELECT * FROM dm_invites WHERE $1 > dm_invites.id ORDER BY dm_invites.id DESC LIMIT 50;";
        let query: QueryAs<'_, Postgres, Dm_Invite, PgArguments> = sqlx::query_as(sql).bind(pin);
        inf.ps_interface.generic_fetch(query).await?
    } else {
        inf.ps_interface
            .select(Some((&["user_id"], &[user_id])), None)
            .await?
    };

    if invites.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor: Option<Uuid> = if invites.len() == 50
        && let Some(back) = invites.iter().next_back()
    {
        Some(back.id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::FOUND,
        success: true,
        msg: String::new(),
        data: Some(DmInvitesResponse {
            cursor: cursor,
            pages: invites,
        }),
    });
}

pub async fn get_friend_invites(
    user_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<RelationInvitesResponse>, InternalError> {
    let invites: Vec<StrippedUser> = if let Some(pin) = cursor {
        let sql: &'static str = "SELECT users.id AS user_id, nickname, pfp FROM users 
        JOIN relations ON relations.related_user = users.id 
        WHERE (relations.state, users.id) = ($1, $2) AND $3 > users.id;";

        let query: QueryAs<'_, Postgres, StrippedUser, PgArguments> = sqlx::query_as(sql)
            .bind(RelationT::Pending)
            .bind(user_id)
            .bind(pin);

        state.ps_interface.generic_fetch(query).await?
    } else {
        let sql: &'static str = "SELECT users.id AS user_id, nickname, pfp FROM users 
        JOIN relations ON relations.related_user = users.id 
        WHERE (relations.state, users.id) = ($1, $2);";

        let query: QueryAs<'_, Postgres, StrippedUser, PgArguments> =
            sqlx::query_as(sql).bind(RelationT::Pending).bind(user_id);

        state.ps_interface.generic_fetch(query).await?
    };

    if invites.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor: Option<Uuid> = if invites.len() == 50
        && let Some(pin) = invites.iter().next_back()
    {
        Some(pin.user_id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::OK,
        success: true,
        msg: String::new(),
        data: Some(RelationInvitesResponse {
            cursor: cursor,
            pages: invites,
        }),
    });
}
