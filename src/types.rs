use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct user {
    id: Uuid,
    username: String,
    password: String,
}