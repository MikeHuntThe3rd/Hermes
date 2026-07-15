use tokio::sync::OnceCell;

use crate::handlers::db::DbInterface;

mod db;
pub mod post;


pub static INTERFACE: OnceCell<DbInterface> = OnceCell::const_new();

pub async fn get_db_interface() -> &'static DbInterface {
    return INTERFACE.get_or_try_init(DbInterface::new).await.expect("failed to connect to database");
}