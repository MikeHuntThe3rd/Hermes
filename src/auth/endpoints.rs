use axum::{Json, http::StatusCode};
use serde::{Serialize, Deserialize};

use crate::auth::extractor::AuthUser;
use crate::types::*;
use crate::auth::creation::create_jwt;
use crate::get_app_state;