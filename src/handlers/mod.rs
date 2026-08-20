pub mod delete;
pub mod patch;
pub mod post;
pub mod get;

use crate::types::{User, StrippedUser};

impl User {
    fn strip(&self) -> StrippedUser {
        return StrippedUser { id: self.id, nicname: self.nickname.clone(), prv: self.prv.clone() , pfp: self.pfp };
    }
}