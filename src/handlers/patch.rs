use axum::extract::{Path, State};
use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::handlers::{StrippedGroup, fetch_group_member};
use crate::{auth::extractor::AuthUser, types::*, responses::error_t::{InternalError, AuthError, GenericErr}};

#[derive(Serialize, Deserialize)]
pub struct InvMng {
    pub accept: bool,
}

pub async fn update_user(_auth: AuthUser, inf: State<AppState>, Json(data): Json<User>) -> Result<Res<User>, InternalError> {
    return match inf.ps_interface.update(data).await {
        Ok(usr) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(usr) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn update_group(auth: AuthUser, State(inf): State<AppState>, Path(group_id): Path<Uuid>, Json(data): Json<StrippedGroup>) -> Result<Res<()>, GenericErr> {
    let member = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
    .await.map_err(|e| GenericErr::Internal(e))?;

    if member.rank != RankT::Owner {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    inf.ps_interface.update(Group {
        id: group_id,
        name: data.name,
        is_dm: data.is_dm,
        gp: data.gp
    }).await.map_err(|e| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}

pub async fn manage_group_invite(auth: AuthUser, inf: State<AppState>, Path(invite_id): Path<Uuid>, Json(data): Json<InvMng>) -> Result<Res<()>, InternalError> {
    let invs: Vec<Group_Invite> = inf.ps_interface.select(Some((&["id"], &[invite_id])))
    .await.map_err(|_| InternalError::DbError)?;

    let inv: Group_Invite = invs.into_iter().next().ok_or(InternalError::NoMatches)?;
    
    if inv.user_id != auth.user_id {
        return Err(InternalError::NoMatches);
    }

    if data.accept {
        inf.ps_interface.insert::<Group_Member>(Group_Member {
            group_id: inv.group_id, 
            member_id: inv.user_id, 
            rank: inv.rank }, true)
        .await.map_err(|_| InternalError::DbError)?;
    }

    inf.ps_interface.delete::<Uuid, Group_Invite>(&[inv.id]).await.map_err(|_| InternalError::DbError)?;

    return Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: None });
}

pub async fn update_message(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Message>) -> Result<Res<Message>, InternalError> {
    return match inf.ps_interface.update(data).await {
        Ok(msg) => Ok(Res { status: StatusCode::OK, success: true, msg: String::new(), data: Some(msg) }),
        Err(_e) => Err(InternalError::DbError),
    }
}