use sqlx::{Encode, FromRow, Pool, Postgres, Sqlite, postgres::{PgArguments, PgConnectOptions, PgPoolOptions, PgRow}, query::{Query, QueryAs}, sqlite::{self, SqliteArguments, SqliteConnectOptions, SqliteRow}};
use crate::types::*;

#[derive(Clone)]
pub struct PsInterface {
    pool: Pool<Postgres>,
}

#[derive(Clone)]
pub struct LiteInterface {
    conn: Pool<Sqlite>,
}

impl PsInterface {
    pub async fn new() -> Result<PsInterface, sqlx::Error> {
        let opt = PgConnectOptions::new()
        .socket("/run/postgresql")
        .username(&std::env::var("PS_USERNAME").expect("a PS_USERNAME enviroment variable is expected"))
        .database("records_ps");

        let conn_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect_with(opt.clone()).await?;
    
        return Ok(PsInterface{ pool: conn_pool});
    }

    pub async fn insert<T>(&self, data: T) -> Result<T, sqlx::Error>
    where T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let vals: Vec<String>= (1..=T::base_columns().len())
        .map(|i| format!("${}", i))
        .collect();
        
        let sql = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING *;"
        , T::table_name()
        , T::base_columns().join(", ")
        , vals.join(", "));

        let query = data.bind_values(sqlx::query_as(&sql), BindVal::BASE);
        let res = query.fetch_one(&self.pool).await?;
        Ok(res)
    }

    pub async fn update<T>(&self, data: T) -> Result<T, sqlx::Error>
    where T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let vals: Vec<String>= (1..=T::columns().len())
        .map(|i| format!("${}", i))
        .collect();

        let id_vals = &vals[0..T::id_columns().len()];
        let base_vals = &vals[(T::id_columns().len())..vals.iter().len()];

        let sql = format!("UPDATE {} SET ({}) = ({}) WHERE ({}) = ({}) RETURNING *;"
        , T::table_name()
        , T::base_columns().join(", ")
        , base_vals.join(", ")
        , T::id_columns().join(", ")
        , id_vals.join(", "));

        let query = data.bind_values(sqlx::query_as(&sql), BindVal::ALL);
        let res = query.fetch_one(&self.pool).await?;
        Ok(res)
    }

    pub async fn delete<I, T>(&self, id_s: &[I]) -> Result<Vec<T>, sqlx::Error>
    where I: for<'q> Encode<'q, Postgres> + sqlx::Type<sqlx::Postgres>
    , T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let vals: Vec<String>= (1..=T::id_columns().len())
        .map(|i| format!("${}", i))
        .collect();

        let sql = format!("DELETE FROM {} WHERE ({}) = ({}) RETURNING *;"
        ,T::table_name()
        ,T::id_columns().join(", ")
        ,vals.join(", "));

        let mut query = sqlx::query_as(&sql);
        for val in id_s {
            query = query.bind(val);
        }
        let res = query.fetch_all(&self.pool).await?;
        Ok(res)
    }

    pub async fn select<I, T>(&self, id_s: Option<(&[&str], &[I])>) -> Result<Vec<T>, sqlx::Error>
    where I: for<'q> Encode<'q, Postgres> + sqlx::Type<sqlx::Postgres>
    , T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let mut sql = format!("SELECT * FROM {}", T::table_name());

        if let Some(some_id_s) = id_s {
            if some_id_s.0.len() != some_id_s.1.len() {
                return Err(sqlx::Error::InvalidArgument("given tuple arrays have different sizes".to_string()));
            }

            let vals: Vec<String> = (1..=some_id_s.0.len())
            .map(|i| format!("${}", i))
            .collect();

            sql = format!("{} WHERE ({}) = ({});"
            , sql
            , some_id_s.0.join(", ")
            , vals.join(", "));
        }
        else {
            sql += ";";
        }

        let mut query: QueryAs<'_, Postgres, T, PgArguments> = sqlx::query_as(&sql);

        if let Some(some_id_s) = id_s {
            for val in some_id_s.1 {
                query = query.bind(val);
            }
        }

        let res = query.fetch_all(&self.pool).await?;
        Ok(res)
    }


    pub async fn generic_exec(&self, query: Query<'_, Postgres, PgArguments>) -> Result<(), sqlx::Error>
    {
        query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn generic_fetch<T>(&self, query: QueryAs<'_, Postgres, T, PgArguments>) -> Result<Vec<T>, sqlx::Error>
    where T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let res = query.fetch_all(&self.pool).await?;
        Ok(res)
    }

}

impl LiteInterface {
    pub async fn new() -> Result<LiteInterface, sqlx::Error> {

        let opt: SqliteConnectOptions = sqlite::SqliteConnectOptions::new().filename(LOGS_PTH_STR).journal_mode(sqlite::SqliteJournalMode::Wal);

        let pool = sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(opt).await?;

        return Ok(LiteInterface { conn: pool });
    }

    pub async fn insert() -> Result<(), sqlx::Error> {
        
        return Ok(());
    }
}