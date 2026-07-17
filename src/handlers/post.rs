use axum::{Json, http::StatusCode};

use crate::types::*;
use crate::handlers::get_db_interface;

pub async fn add_user(Json(data): Json<User>) -> Result<Json<User>, StatusCode> {
    let inf = get_db_interface().await;

    return match inf.insert(data).await {
        Ok(usr) => Ok(Json(usr)),
        Err(_e) => {
            println!("{}", _e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

pub async fn del_test(Json(data): Json<User>) -> Result<Json<User>, StatusCode> {
    let inf = get_db_interface().await;

    return match inf.delete(&vec![data.id]).await {
        Ok(usr) => Ok(Json(usr)),
        Err(_e) => {
            println!("{}", _e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

pub async fn upd_test(Json(data): Json<User>) -> Result<Json<User>, StatusCode> {
    let inf = get_db_interface().await;

    return match inf.update(data).await {
        Ok(usr) => Ok(Json(usr)),
        Err(_e) => {
            println!("{}", _e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}

pub async fn get_test(Json(data): Json<User>) -> Result<Json<Vec<User>>, StatusCode> {
    let inf = get_db_interface().await;

    return match inf.select(&vec![data.id]).await {
        Ok(usr) => Ok(Json(usr)),
        Err(_e) => {
            println!("{}", _e.to_string());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}