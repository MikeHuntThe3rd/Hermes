use axum::{Json, extract::{Multipart, State}, http::StatusCode};
use tokio::io::AsyncWriteExt;

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

pub async fn upload(mut data: Multipart) -> Result<Res<()>, InternalError> {
    let match_err = move |status: StatusCode| -> InternalError {
        return match status {
            StatusCode::BAD_REQUEST => InternalError::BadRequest,
            StatusCode::PAYLOAD_TOO_LARGE => InternalError::BodyTooLarge,
            _ => InternalError::DecodeEncodeErr,
        };
    };

    while let Some(mut field) = data
    .next_field().await.map_err(|e| match_err(e.status()))? 
    {
        let temp_path = TEMP_PTH_STR.to_string() + &uuid::Uuid::new_v4().to_string();
        let mut temp_file = tokio::fs::File::create_new(temp_path)
        .await.map_err(|_| InternalError::OperationsError)?;

        while let Some(chunk) = field
        .chunk().await.map_err(|e| match_err(e.status()))? 
        {
            temp_file.write_all(&chunk)
            .await.map_err(|_| InternalError::OperationsError)?;    
        }

        temp_file.flush().await.map_err(|_| InternalError::OperationsError)?;
        temp_file.sync_all().await.map_err(|_| InternalError::OperationsError)?;
    }

    return Err(InternalError::OperationsError);
}