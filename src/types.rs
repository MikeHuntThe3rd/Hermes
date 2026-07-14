use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct msg {
    pub text: String,
}