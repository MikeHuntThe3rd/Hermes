use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use sqlx::Postgres;
use sqlx::postgres::PgArguments;
use sqlx::query::QueryAs;
use uuid::Uuid;

use crate::{auth::extractor::AuthUser, responses::error_t::InternalError};
use crate::{types::*, handlers::*};


pub async fn get_friends(auth: AuthUser, State(inf): State<AppState>) -> Result<Res<Vec<StrippedUser>>, InternalError> {
    //TODO: SQL HASNT BEEN REWRITTEN YET!!!!!!!!!
    let sql: &'static str = "SELECT * FROM users 
    JOIN relations ON relations.related_user = users.id 
    WHERE relations.relating_user = $1 AND relations.state = $2;";

    let query: QueryAs<'_, Postgres, StrippedUser, PgArguments> = sqlx::query_as(sql)
    .bind(auth.user_id)
    .bind(RelationT::Friends);

    let users: Vec<StrippedUser> = inf.ps_interface.generic_fetch(query).await.map_err(|_| InternalError::DbError)?;

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
    WHERE group_members.member_id = $1;";

    let query: QueryAs<'_, Postgres, Group, PgArguments> = sqlx::query_as(sql)
    .bind(auth.user_id);

    let groups = inf.ps_interface.generic_fetch(query)
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

pub async fn get_group_members(_auth: AuthUser, State(inf): State<AppState>, Path(group_id) : Path<Uuid>) -> Result<Res<Vec<StrippedUser>>, InternalError> {
    let sql: &'static str = "SELECT id AS user_id, nickname, pfp, group_members.rank AS rank FROM users 
    JOIN group_members ON group_members.member_id = users.id 
    WHERE group_members.group_id = $1;";

    //checks if the group exists
    fetch_group(inf.clone(), &group_id).await?;

    let query: QueryAs<'_, Postgres, StrippedUser, PgArguments> = sqlx::query_as(sql)
    .bind(group_id);

    let users: Vec<StrippedUser> = inf.ps_interface.generic_fetch(query)
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
    WHERE groups.id = $1;";

    let query: QueryAs<'_, Postgres, Message, PgArguments> = sqlx::query_as(sql)
    .bind(group_id);

    let messages = inf.ps_interface.generic_fetch(query)
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

pub async fn get_invites(auth: AuthUser, State(inf): State<AppState>) -> Result<Res<Vec<Group_Invite>>, InternalError> {
    let invs: Vec<Group_Invite> = inf.ps_interface.select(Some((&["user_id"], &[auth.user_id])))
    .await.map_err(|_| InternalError::DbError)?;

    if invs.first().is_none() {
        return Err(InternalError::NoMatches);
    }
    return Ok(Res { status: StatusCode::FOUND, success: true, msg: String::new(), data: Some(invs) });
}

pub async fn pull(_auth: AuthUser, State(inf): State<AppState>, Path(object_id): Path<Uuid>) -> Result<impl IntoResponse, InternalError> {
    let obj: Object = 
    inf.ps_interface
    .select::<Uuid, Object>(Some((Object::id_columns(), &[object_id])))
    .await.map_err(|_| InternalError::DbError)?
    .into_iter().next()
    .ok_or(InternalError::NoMatches)?;

    Ok(
        Response::builder()
        .header("X-Accel-Redirect", format!("/files/objs/{}", obj.rel_path))
        .header("Content-Type", obj.mime_type)
        .body(Body::empty())
        .map_err(|_| InternalError::OperationsError)?
    )
}