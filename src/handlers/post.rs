use axum::{Json, extract::{Multipart, State}, http::StatusCode};
use tokio::io::AsyncWriteExt;
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::{auth::extractor::AuthUser, types::*, errors::error_t::InternalError};

pub async fn add_group(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group>) -> Result<Res<Group>, InternalError> {
    return match inf.db_interface.insert::<Group>(data).await {
        Ok(grp) => Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(grp) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn add_group_member(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Group_Member>) -> Result<Res<Group_Member>, InternalError> {
    return match inf.db_interface.insert::<Group_Member>(data).await {
        Ok(grp_mem) => Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(grp_mem) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn add_message(_auth: AuthUser, inf: State<AppState>, Json(data): Json<Message>) -> Result<Res<Message>, InternalError> {
    return match inf.db_interface.insert::<Message>(data).await {
        Ok(msg) => Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(msg) }),
        Err(_e) => Err(InternalError::DbError),
    }
}

pub async fn upload(inf: State<AppState>, mut data: Multipart) -> Result<Res<ObjectIds>, InternalError> {
    let match_err = move |status: StatusCode| -> InternalError {
        return match status {
            StatusCode::BAD_REQUEST => InternalError::BadRequest,
            StatusCode::PAYLOAD_TOO_LARGE => InternalError::BodyTooLarge,
            _ => InternalError::DecodeEncodeErr,
        };
    };

    let mut objs = ObjectIds {ids: vec![]};

    while let Some(mut field) = data
    .next_field().await.map_err(|e| match_err(e.status()))? 
    {
        let uuid_name = uuid::Uuid::new_v4().to_string();
        let temp_path = TEMP_PTH_STR.to_string() + &uuid_name;
        let mut temp_file = tokio::fs::File::create_new(&temp_path)
        .await.map_err(|_| InternalError::OperationsError)?;

        let mut hasher = Sha256::new();

        /* ===== LOAD TEMP FILE ===== */
        while let Some(chunk) = field
        .chunk().await.map_err(|e| match_err(e.status()))? 
        {
            hasher.update(&chunk);
            temp_file.write_all(&chunk)
            .await.map_err(|_| InternalError::OperationsError)?;
        }

        temp_file.flush().await.map_err(|_| InternalError::OperationsError)?;
        temp_file.sync_all().await.map_err(|_| InternalError::OperationsError)?;

        let hash = hex::encode(hasher.finalize());

        let matches: Vec<Object> = inf.db_interface
        .select::<&str, Object>(Some((&["hash"], &[&hash])))
        .await.map_err(|_| InternalError::DbError)?;

        /* ===== ERASE TEMP FILE ===== */
        let mime = if let Some(ty) = infer::get(
            &tokio::fs::read(&temp_path)
            .await.map_err(|_| InternalError::OperationsError)?)
        {
            ty
        }
        else {
            return Err(InternalError::UnknownType);
        };
        
        let mut size:i64 = 0;
        let new_obj_path = if let Some(frst) = matches.first() 
        {
            tokio::fs::remove_file(temp_path).await.map_err(|_| InternalError::OperationsError)?;
            frst.rel_path.clone()
        }
        else {
            size = temp_file.metadata()
            .await.map_err(|_| InternalError::OperationsError)?.len() as i64;

            let full_path = OBJ_PTH_STR.to_string() + &hash + "/" + &uuid_name;
            tokio::fs::rename(&temp_path, full_path)
            .await.map_err(|_| {
                tokio::spawn(tokio::fs::remove_file(temp_path));
                InternalError::OperationsError
            })?;

            hash.clone() + "/" + &uuid_name
        };

        let new_obj = Object {
            id: None,
            hash: hash,
            rel_path: new_obj_path.clone(),
            mime_type: mime.to_string(),
            size_bytes: size,
            creation_timestamp: OffsetDateTime::now_utc().unix_timestamp() as i64
        };

        

        let obj = inf.db_interface.insert::<Object>(new_obj)
        .await.map_err(|_| {
            if matches.first().is_none() {
                tokio::spawn(tokio::fs::remove_file(OBJ_PTH_STR.to_string() + &new_obj_path));  
            }
            InternalError::DbError
        })?;
        
        if let Some(tmp_id) = obj.id {
            objs.ids.push(tmp_id);
        }
    }

    return Ok(Res { status: StatusCode::CREATED, success: true, msg: String::new(), data: Some(objs) });
}