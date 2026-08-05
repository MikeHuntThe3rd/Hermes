use axum::extract::{Path, State};
use axum::http::StatusCode;
use sqlx::Postgres;
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, errors::error_t::InternalError};
use crate::types::*;


pub async fn get_friends(auth: AuthUser, State(inf): State<AppState>) -> Result<Res<Vec<User>>, InternalError> {
    let sql: &'static str = "SELECT * FROM users 
    JOIN relations ON relations.related_user = users.id 
    WHERE relations.relating_user = {$1} AND relations.state = {$2};";

    let query: QueryAs<'_, Postgres, User, PgArguments> = sqlx::query_as(&sql)
    .bind(auth.user_id)
    .bind(RelationT::Friends);

    let users = inf.db_interface.generic_fetch(query).await.map_err(|_| InternalError::DbError)?;

    if users.len() < 1 {
        return Err(InternalError::NoMatches);
    }

    return Ok(
        Res { 
            status: StatusCode::FOUND, 
            success: true, 
            msg: String::new(), 
            data: Some(users) 
        }
    );

}

pub async fn get_groups(auth: AuthUser, State(inf): State<AppState>) -> Result<Res<Vec<Group>>, InternalError> {
    let sql: &'static str = "SELECT * FROM groups 
    JOIN group_members ON group_members.group_id = groups.id 
    WHERE group_members.member_id = {$1};";

    let query: QueryAs<'_, Postgres, Group, PgArguments> = sqlx::query_as(&sql)
    .bind(auth.user_id);

    let groups = inf.db_interface.generic_fetch(query)
    .await.map_err(|_| InternalError::DbError)?;

    if groups.len() < 1 {
        return Err(InternalError::NoMatches);
    }

    return Ok(
        Res { 
            status: StatusCode::FOUND, 
            success: true, 
            msg: String::new(), 
            data: Some(groups) 
        }
    );
}

pub async fn get_group_members(_auth: AuthUser, State(inf): State<AppState>, Path(group_id) : Path<Uuid>) -> Result<Res<Vec<User>>, InternalError> {
    let sql: &'static str = "SELECT * FROM users 
    JOIN group_members ON group_members.member_id = users.id 
    WHERE group_members.group_id = {$1};";

    let query: QueryAs<'_, Postgres, User, PgArguments> = sqlx::query_as(&sql)
    .bind(group_id);

    let users = inf.db_interface.generic_fetch(query)
    .await.map_err(|_| InternalError::DbError)?;

    if users.len() < 1 {
        return Err(InternalError::NoMatches);
    }

    return Ok(
        Res { 
            status: StatusCode::FOUND, 
            success: true, 
            msg: String::new(), 
            data: Some(users) 
        }
    );
}

pub async fn get_messages(_auth: AuthUser, State(inf): State<AppState>, Path(group_id) : Path<Uuid>) -> Result<Res<Vec<Message>>, InternalError> {
    let sql: &'static str = "SELECT * FROM messages 
    JOIN groups ON groups.id = messages.group_id 
    WHERE groups.id = {$1};";

    let query: QueryAs<'_, Postgres, Message, PgArguments> = sqlx::query_as(&sql)
    .bind(group_id);

    let messages = inf.db_interface.generic_fetch(query)
    .await.map_err(|_| InternalError::DbError)?;

    if messages.len() < 1 {
        return Err(InternalError::NoMatches);
    }

    return Ok(
        Res { 
            status: StatusCode::FOUND, 
            success: true, 
            msg: String::new(), 
            data: Some(messages) 
        }
    );
}