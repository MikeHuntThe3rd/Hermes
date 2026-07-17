use tokio::sync::OnceCell;

use crate::db::*;

pub mod delete;
pub mod patch;
pub mod post;
pub mod get;


pub static INTERFACE: OnceCell<DbInterface> = OnceCell::const_new();

pub async fn get_db_interface() -> &'static DbInterface {
    return INTERFACE.get_or_try_init(DbInterface::new).await.expect("failed to connect to database");
}