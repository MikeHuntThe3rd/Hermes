use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, responses::error_t::InternalError};
use crate::{handlers::*, types::*};

#[derive(Serialize)]
pub struct StrippedChannel {
    pub id: Uuid,
    pub name: String,
    pub category: ChannelT,
}

#[derive(Serialize)]
pub struct GuildChannelMessage {
    pub id: i64,
    pub message: Option<String>,
    pub file_ids: Vec<Uuid>,
    pub channel_id: Uuid,
    pub user_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct DmMessage {
    pub id: i64,
    pub message: Option<String>,
    pub file_ids: Vec<Uuid>,
    pub dm_id: Uuid,
    pub user_id: Option<Uuid>,
}

#[derive(Serialize)]
pub struct GenericMsgResponse<T> {
    pub cursor: Option<i64>,
    pub pages: Vec<T>,
}

pub type GuildMsgResponse = GenericMsgResponse<GuildChannelMessage>;
pub type DmMsgResponse = GenericMsgResponse<DmMessage>;

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

pub async fn get_inital_dm_messages(
    auth: AuthUser,
    Path(dm_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Res<DmMsgResponse>, InternalError> {
    return get_dm_messages(auth.user_id, dm_id, None, state).await;
}

pub async fn get_dm_messages_from(
    auth: AuthUser,
    Path((dm_id, cursor)): Path<(Uuid, i64)>,
    State(state): State<AppState>,
) -> Result<Res<DmMsgResponse>, InternalError> {
    return get_dm_messages(auth.user_id, dm_id, Some(cursor), state).await;
}

pub async fn get_inital_channel_messages(
    auth: AuthUser,
    Path((guild_id, channel_id)): Path<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Res<GuildMsgResponse>, InternalError> {
    return get_guild_channel_messages(auth.user_id, guild_id, channel_id, None, state).await;
}

pub async fn get_channel_messages_from(
    auth: AuthUser,
    Path((guild_id, channel_id, cursor)): Path<(Uuid, Uuid, i64)>,
    State(state): State<AppState>,
) -> Result<Res<GuildMsgResponse>, InternalError> {
    return get_guild_channel_messages(auth.user_id, guild_id, channel_id, Some(cursor), state).await;
}

pub async fn pull(
    _auth: AuthUser,
    State(state): State<AppState>,
    Path(object_id): Path<Uuid>,
) -> Result<impl IntoResponse, InternalError> {
    let obj: Object = state
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
        let query = sqlx::query_as!(
            StrippedUser,
            "SELECT users.id AS user_id, users.nickname AS nickname, users.pfp AS pfp FROM users 
            JOIN relations ON relations.related_user = users.id 
            WHERE (relations.relating_user, relations.state) = ($1, $2) AND $3 > users.id
            ORDER BY users.id DESC
            LIMIT 50;",
            user_id,
            RelationT::Friends as RelationT,
            pin
        );

        state.ps_interface.generic_fetch(query).await?
    } else {
        let query = sqlx::query_as!(
            StrippedUser,
            "SELECT users.id AS user_id, users.nickname AS nickname, users.pfp AS pfp FROM users 
            JOIN relations ON relations.related_user = users.id 
            WHERE (relations.relating_user, relations.state) = ($1, $2)
            ORDER BY users.id DESC
            LIMIT 50;",
            user_id,
            RelationT::Friends as RelationT
        );

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
    let guilds: Vec<Guild> = if let Some(pin) = cursor {
        let query = sqlx::query_as!(
            Guild,
            "SELECT guilds.*
            FROM guilds
            WHERE EXISTS (
                SELECT 1
                FROM guild_members
                WHERE guild_members.guild_id = guilds.id
                  AND guild_members.member_id = $1
              )
              AND $2 > guilds.id
            ORDER BY guilds.id DESC
            LIMIT 50;",
            user_id,
            pin
        );

        state.ps_interface.generic_fetch(query).await?
    } else {
        let query = sqlx::query_as!(
            Guild,
            "SELECT guilds.*
            FROM guilds
            WHERE EXISTS (
                SELECT 1
                FROM guild_members
                WHERE guild_members.guild_id = guilds.id
                  AND guild_members.member_id = $1
              )
            ORDER BY guilds.id DESC
            LIMIT 50;",
            user_id
        );

        state.ps_interface.generic_fetch(query).await?
    };

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
    guild_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<ChannelsResponse>, InternalError> {
    let channels: Vec<StrippedChannel> = if let Some(pin) = cursor {
        let query = sqlx::query_as!(
            StrippedChannel,
            r#"SELECT channels.id AS id, channels.name AS name, channels.category AS "category: ChannelT"
            FROM channels WHERE channels.guild_id = $1 AND $2 > channels.id  ORDER BY channels.id DESC LIMIT 50;"#,
            guild_id,
            pin
        );

        state.ps_interface.generic_fetch(query).await?
    } else {
        let query = sqlx::query_as!(
            StrippedChannel,
            r#"SELECT channels.id AS id, channels.name AS name, channels.category AS "category: ChannelT"
            FROM channels WHERE channels.guild_id = $1 ORDER BY channels.id DESC LIMIT 50;"#,
            guild_id
        );

        state.ps_interface.generic_fetch(query).await?
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
    guild_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<GuildMembersResponse>, InternalError> {
    state
        .ps_interface
        .select::<Uuid, Guild>(Some((&["id"], &[guild_id])), Some(1))
        .await?;

    let members: Vec<StrippedMember> = if let Some(pin) = cursor {
        let query = sqlx::query_as!(
            StrippedMember,
            r#"SELECT users.id AS user_id, users.nickname AS nickname, users.pfp AS pfp, guild_members.rank AS "rank: RankT" FROM
            users 
            JOIN guild_members ON guild_members.member_id = users.id 
            WHERE guild_members.guild_id = $1 AND $2 > users.id 
            ORDER BY guild_members.member_id DESC
            LIMIT 50;"#,
            guild_id,
            pin
        );

        state.ps_interface.generic_fetch(query).await?
    } else {
        let query = sqlx::query_as!(
            StrippedMember,
            r#"SELECT users.id AS user_id, users.nickname AS nickname, users.pfp AS pfp, guild_members.rank AS "rank: RankT" FROM
            users 
            JOIN guild_members ON guild_members.member_id = users.id 
            WHERE guild_members.guild_id = $1
            ORDER BY guild_members.member_id DESC
            LIMIT 50;"#,
            guild_id
        );

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

async fn get_guild_channel_messages(
    user_id: Uuid,
    guild_id: Uuid,
    channel_id: Uuid,
    cursor: Option<i64>,
    state: AppState,
) -> Result<Res<GuildMsgResponse>, InternalError> {
    fetch_guild_member(state.clone(), &guild_id, &user_id).await?;

    let messages: Vec<GuildChannelMessage> = if let Some(pin) = cursor {
        let messages_query = sqlx::query_as!(GuildChannelMessage, 
            r#"SELECT 
                guild_messages.id AS id, 
                guild_messages.message AS message,
                COALESCE(array_agg(guild_message_objects.object_id) 
                FILTER (WHERE guild_message_objects.object_id IS NOT NULL), '{}'::uuid[]) AS "file_ids!", 
                guild_messages.channel_id AS channel_id,
                guild_messages.user_id AS user_id 
            FROM guild_messages 
            LEFT JOIN guild_message_objects ON guild_message_objects.message_id = guild_messages.id
            WHERE guild_messages.channel_id = $1 AND $2 > guild_messages.id
            GROUP BY guild_messages.id
            ORDER BY guild_messages.id ASC
            LIMIT 50;"#,
            channel_id,
            pin
        );

        state.ps_interface.generic_fetch(messages_query).await? 
    } else {
        let messages_query = sqlx::query_as!(GuildChannelMessage, 
            r#"SELECT 
                guild_messages.id AS id, 
                guild_messages.message AS message,
                COALESCE(array_agg(guild_message_objects.object_id) 
                FILTER (WHERE guild_message_objects.object_id IS NOT NULL), '{}'::uuid[]) AS "file_ids!", 
                guild_messages.channel_id AS channel_id,
                guild_messages.user_id AS user_id 
            FROM guild_messages 
            LEFT JOIN guild_message_objects ON guild_message_objects.message_id = guild_messages.id
            WHERE guild_messages.channel_id = $1
            GROUP BY guild_messages.id
            ORDER BY guild_messages.id ASC
            LIMIT 50;"#,
            channel_id
        );

        state.ps_interface.generic_fetch(messages_query).await? 
    };

    if messages.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor = if messages.len() == 50 && let Some(pin) = messages.iter().next_back() {
        Some(pin.id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::FOUND,
        success: true,
        msg: String::new(),
        data: Some(GuildMsgResponse{
            cursor: cursor,
            pages: messages
        }),
    });
}

async fn get_dm_messages(
    user_id: Uuid,
    dm_id: Uuid,
    cursor: Option<i64>,
    state: AppState,
) -> Result<Res<DmMsgResponse>, InternalError> {
    let member_query = sqlx::query_as!(Dm, "SELECT * FROM dms WHERE id =  $1 AND (user_a = $2 OR user_b = $2)", dm_id, user_id);

    state.ps_interface.generic_fetch(member_query)
    .await?.into_iter().next().ok_or(InternalError::NoMatches)?;

    let messages: Vec<DmMessage> = if let Some(pin) = cursor {
        let messages_query = sqlx::query_as!(DmMessage, 
            r#"SELECT 
                dm_messages.id AS id, 
                dm_messages.message AS message,
                COALESCE(array_agg(guild_message_objects.object_id) 
                FILTER (WHERE guild_message_objects.object_id IS NOT NULL), '{}'::uuid[]) AS "file_ids!", 
                dm_messages.dm_id AS dm_id,
                dm_messages.user_id AS user_id 
            FROM dm_messages 
            LEFT JOIN guild_message_objects ON guild_message_objects.message_id = dm_messages.id
            WHERE dm_messages.dm_id = $1 AND $2 > dm_messages.id
            GROUP BY dm_messages.id
            ORDER BY dm_messages.id ASC
            LIMIT 50;"#,
            dm_id,
            pin
        );

        state.ps_interface.generic_fetch(messages_query).await? 
    } else {
        let messages_query = sqlx::query_as!(DmMessage, 
            r#"SELECT 
                dm_messages.id AS id, 
                dm_messages.message AS message,
                COALESCE(array_agg(guild_message_objects.object_id) 
                FILTER (WHERE guild_message_objects.object_id IS NOT NULL), '{}'::uuid[]) AS "file_ids!", 
                dm_messages.dm_id AS dm_id,
                dm_messages.user_id AS user_id 
            FROM dm_messages 
            LEFT JOIN guild_message_objects ON guild_message_objects.message_id = dm_messages.id
            WHERE dm_messages.dm_id = $1
            GROUP BY dm_messages.id
            ORDER BY dm_messages.id ASC
            LIMIT 50;"#,
            dm_id
        );

        state.ps_interface.generic_fetch(messages_query).await? 
    };

    if messages.is_empty() {
        return Err(InternalError::NoMatches);
    }

    let cursor = if messages.len() == 50 && let Some(pin) = messages.iter().next_back() {
        Some(pin.id)
    } else {
        None
    };

    return Ok(Res {
        status: StatusCode::FOUND,
        success: true,
        msg: String::new(),
        data: Some(DmMsgResponse{
            cursor: cursor,
            pages: messages
        }),
    });
}

async fn get_guild_invites(
    user_id: Uuid,
    cursor: Option<Uuid>,
    state: AppState,
) -> Result<Res<GuildInvitesResponse>, InternalError> {
    let invites: Vec<Guild_Invite> = if let Some(pin) = cursor {
        let query = sqlx::query_as!(
            Guild_Invite,
            r#"
            SELECT id, guild_id, recipient, rank AS "rank: RankT" FROM 
            guild_invites WHERE guild_invites.recipient = $2 AND $1 > guild_invites.id  
            ORDER BY guild_invites.id DESC LIMIT 50;"#,
            user_id,
            pin
        );

        state.ps_interface.generic_fetch(query).await?
    } else {
        let query = sqlx::query_as!(
            Guild_Invite,
            r#"
            SELECT id, guild_id, recipient, rank AS "rank: RankT" FROM 
            guild_invites WHERE guild_invites.recipient = $1 
            ORDER BY guild_invites.id DESC LIMIT 50;"#,
            user_id
        );

        state.ps_interface.generic_fetch(query).await?
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
    state: AppState,
) -> Result<Res<DmInvitesResponse>, InternalError> {
    let invites: Vec<Dm_Invite> = if let Some(pin) = cursor {
        let query = sqlx::query_as!(
            Dm_Invite,
            "SELECT * FROM
            dm_invites WHERE dm_invites.recipient = $1 AND $2 > dm_invites.id ORDER BY dm_invites.id DESC LIMIT 50;",
            user_id,
            pin 
        );

        state.ps_interface.generic_fetch(query).await?
    } else {
        let query = sqlx::query_as!(
            Dm_Invite,
            "SELECT * FROM
            dm_invites WHERE dm_invites.recipient = $1 ORDER BY dm_invites.id DESC LIMIT 50;",
            user_id
        );

        state.ps_interface.generic_fetch(query).await?
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
        let query = sqlx::query_as!(StrippedUser, 
            r#"SELECT users.id AS user_id, nickname, pfp FROM users 
            JOIN relations ON relations.related_user = users.id 
            WHERE (relations.state, users.id) = ($1, $2) AND $3 > users.id;"#,
            RelationT::Pending as RelationT,
            user_id,
            pin
        );

        state.ps_interface.generic_fetch(query).await?
    } else {
        let query = sqlx::query_as!(StrippedUser, 
            r#"SELECT users.id AS user_id, nickname, pfp FROM users 
            JOIN relations ON relations.related_user = users.id 
            WHERE (relations.state, users.id) = ($1, $2);"#,
            RelationT::Pending as RelationT,
            user_id
        );

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
