use sqlx::{Pool, Postgres, postgres::{PgPoolOptions, PgConnectOptions}};

pub struct DbInterface {
    options: PgConnectOptions,
    pool: Pool<Postgres>,
}

impl DbInterface {
    pub async fn new() -> Result<DbInterface, sqlx::Error> {
        let opt = PgConnectOptions::new()
        .host("localhost")
        .port(5432)
        .username("postgres")
        .password("lol no")
        .database("records_ps");

        let conn_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_with(opt.clone()).await?;
    
        return Ok(DbInterface{ options: opt, pool: conn_pool});
    }

    pub async fn generic_query(&self, usr: &str, pswr: &str) {
        sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2);")
        .bind(usr)
        .bind(pswr)
        .execute(&self.pool)
        .await.unwrap();
    }
}