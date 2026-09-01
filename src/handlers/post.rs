use axum::{Json, extract::{Multipart, Path, State}, http::StatusCode};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{auth::{creation::create_priv_jwt, extractor::AuthUser}, handlers::fetch_group_member, responses::error_t::{AuthError, GenericErr, InternalError}, types::*};

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
pub struct Msg {
    pub file_ids: Vec<Uuid>,
    pub message: Option<String>,
    pub group_id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct Grp {
    pub name: String,
    pub is_dm: bool,
    pub gp: Option<Uuid>,
}

pub async fn add_group(auth: AuthUser, inf: State<AppState>, Json(data): Json<Grp>) -> Result<Res<()>, InternalError> {
    let group = inf.ps_interface.insert::<Group>(Group {
        id: PLACE_HOLDER_UUID, 
        name: data.name, 
        is_dm: data.is_dm, 
        gp: data.gp 
    }, false)
    .await.map_err(|_| InternalError::DbError)?;

    let member = inf.ps_interface.insert::<Group_Member>(Group_Member { 
        group_id: group.id, 
        member_id: auth.user_id, 
        rank: RankT::Owner 
    }, true).await;

    if member.is_err() {
        inf.ps_interface.delete::<Uuid, Group>(&[group.id]).await.map_err(|_| InternalError::DbError)?;
        return Err(InternalError::DbError);
    }

    return Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: None });
}

//TODO: SAFEGURAD FOR DB FALIURES AFTER BASE MESSAGE INSERT
pub async fn add_message(auth: AuthUser, inf: State<AppState>, Json(data): Json<Msg>) -> Result<Res<()>, GenericErr> {
    if data.file_ids.first().is_none() && data.message.is_none() {
        return Err(GenericErr::Internal(InternalError::EmptyMessage));
    }

    let grp_member: Vec<Group_Member> = inf.ps_interface.select(Some((&["group_id", "member_id"], &[data.group_id, auth.user_id])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let grp = grp_member.first().ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    if grp.rank < RankT::Admin {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    let msg: Message = inf.ps_interface.insert(Message { 
        id: 0, 
        message: data.message, 
        group_id: data.group_id, 
        user_id: auth.user_id 
    }, false)
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    for file_id in data.file_ids {
        inf.ps_interface.insert(Message_Object {message_id: msg.id, object_id: file_id}, true).await.unwrap();
    }

    Err(GenericErr::Auth(AuthError::ExpiredToken))
}

pub async fn invite_group_member(auth: AuthUser, State(inf): State<AppState>, Path(group_id): Path<Uuid>, Json(data): Json<GrpInv>) -> Result<Res<()>, GenericErr> {
    if auth.user_id == data.user_id {
        return Err(GenericErr::Auth(AuthError::SelfInvite));
    }

    let invs: Vec<Group_Invite> = inf.ps_interface.select(Some((&["group_id", "user_id"], &[group_id, data.user_id])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    if invs.first().is_some() {
        return Err(GenericErr::Internal(InternalError::DuplicateData));
    }

    let member = fetch_group_member(inf.clone(), &group_id, &auth.user_id)
    .await.map_err(|e| GenericErr::Internal(e))?;

    let usr: Vec<User> = inf.ps_interface.select(Some((&["id"], &[data.user_id])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    if member.rank < data.rank {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }

    if usr.first().is_none() {
        return Err(GenericErr::Internal(InternalError::NoMatches));
    }

    let inv = Group_Invite {
        id: PLACE_HOLDER_UUID,
        group_id: group_id,
        user_id: data.user_id,
        rank: data.rank,
    };

    inf.ps_interface.insert(inv, true).await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    return Ok(Res { status: StatusCode::CREATED, success: true, msg: "invite created successfully".to_string(), data: None });
}

pub async fn create_invite(auth: AuthUser, State(inf): State<AppState>, Json(data): Json<PrivLevel>) -> Result<Res<InvJwt>, GenericErr> {
    let user: Vec<User> = inf.ps_interface.select(Some((&["id"], &vec![auth.user_id])))
    .await.map_err(|_| GenericErr::Internal(InternalError::DbError))?;

    let usr = user.first().ok_or(GenericErr::Internal(InternalError::NoMatches))?;

    if usr.prv != PrivilegeT::Proprietor {
        return Err(GenericErr::Auth(AuthError::InvalidPrivilige));
    }
    
    let jwt_str = create_priv_jwt(data.prv_level, &inf.jwt_secret)
    .await.map_err(|_| GenericErr::Internal(InternalError::DecodeEncodeErr))?;

    let res: Res<InvJwt> = Res {
        status: StatusCode::CREATED,
        success: true,
        msg: String::new(),
        data: Some(InvJwt { jwt: jwt_str }),
    };

    return Ok(res);
}

pub async fn upload(_auth: AuthUser, inf: State<AppState>, mut data: Multipart) -> Result<Res<ObjectIds>, InternalError> {
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

    let mut objs = ObjectIds {ids: vec![]};

    while let Some(mut field) = data
    .next_field().await.map_err(|e| match_err(e.status(), None))? 
    {
        let uuid_name = uuid::Uuid::new_v4().to_string();
        let temp_path = TEMP_PTH_STR.to_string() + &uuid_name;
        let mut temp_file = tokio::fs::File::create_new(&temp_path)
        .await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

        let mut hasher = Sha256::new();

        /* ===== LOAD TEMP FILE ===== */
        while let Some(chunk) = field
        .chunk().await.map_err(|e| match_err(e.status(), Some(temp_path.clone())))? 
        {
            hasher.update(&chunk);
            temp_file.write_all(&chunk)
            .await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;
        }

        temp_file.flush().await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;
        temp_file.sync_all().await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

        let hash = hex::encode(hasher.finalize());

        let matches: Vec<Object> = inf.ps_interface
        .select::<&str, Object>(Some((&["hash"], &[&hash])))
        .await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::DbError))?;

        /* ===== CONSUME TEMP FILE ===== */
        let mime = if let Some(typ) = infer::get(
            &tokio::fs::read(&temp_path)
            .await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?)
        {
            typ
        }
        else {
            return Err(cleanup_err(temp_path.clone(), InternalError::UnknownType));
        };
        
        let mut size:i64 = 0;
        let new_obj_path = if let Some(frst) = matches.first() 
        {
            tokio::spawn(tokio::fs::remove_file(temp_path.clone()));
            frst.rel_path.clone()
        }
        else {
            size = temp_file.metadata()
            .await.map_err(|_| InternalError::OperationsError)?.len() as i64;

            let dir_path = OBJ_PTH_STR.to_string() + &hash;
            let full_path = OBJ_PTH_STR.to_string() + &hash + "/" + &uuid_name;

            tokio::fs::create_dir_all(&dir_path)
            .await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

            tokio::fs::rename(&temp_path, full_path)
            .await.map_err(|_| cleanup_err(temp_path.clone(), InternalError::OperationsError))?;

            hash.clone() + "/" + &uuid_name
        };

        let new_obj = Object {
            id: PLACE_HOLDER_UUID,
            hash: hash,
            rel_path: new_obj_path.clone(),
            mime_type: mime.to_string(),
            size_bytes: size,
            creation_timestamp: OffsetDateTime::now_utc().unix_timestamp() as i64
        };

        let obj = inf.ps_interface.insert::<Object>(new_obj, false)
        .await.map_err(|_| {
            if matches.first().is_none() {
                tokio::spawn(tokio::fs::remove_file(OBJ_PTH_STR.to_string() + &new_obj_path));  
            }
            InternalError::DbError
        })?;

        objs.ids.push(obj.id);
    }

    return Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(objs) });
}
