use sqlx::{FromRow, Pool, Postgres, postgres::{PgConnectOptions, PgPoolOptions, PgRow}};
use crate::types::*;

pub struct DbInterface {
    options: PgConnectOptions,
    pool: Pool<Postgres>,
}

impl DbInterface {
    pub async fn new() -> Result<DbInterface, sqlx::Error> {
        let opt = PgConnectOptions::new()
        .host("localhost")
        .port(5432)
        .username(&std::env::var("PS_USERNAME").expect("a PS_USERNAME enviroment variable is expected"))
        .password(&std::env::var("PS_PASSWORD").expect("a PS_PASSWORD enviroment variable is expected"))
        .database("records_ps");

        let conn_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(opt.clone()).await?;
    
        return Ok(DbInterface{ options: opt, pool: conn_pool});
    }

    pub async fn insert<O, T>(&self, data: T) -> Result<O, sqlx::Error>
    where O: for<'r> FromRow<'r, PgRow> + Send + Unpin, T: Bindable<O>
    {
        let cols = T::columns().join(", ");

        let vals: Vec<String>= (1..=T::columns().len())
        .map(|i| format!("${}", i))
        .collect();

        let returning = if T::columns()[0] == "id" { cols.clone() } 
        else { [ &["id"], T::columns()].concat().join(", ") };

        let sql = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING {};", T::table_name(), cols, vals.join(", "), returning);
        
        let query = data.bind_values(sqlx::query_as(&sql));
        let res = query.fetch_one(&self.pool).await?;
        Ok(res)
    }
}