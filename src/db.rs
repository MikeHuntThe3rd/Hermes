use sqlx::{Encode, FromRow, Pool, Postgres, QueryBuilder, Sqlite, postgres::{PgArguments, PgConnectOptions, PgPoolOptions, PgRow}, query::{Query, QueryAs}, sqlite::{self, SqliteConnectOptions}};
use crate::{logging::Logs, types::*};

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

    pub async fn insert<T>(&self, data: T, insert_all: bool) -> Result<T, sqlx::Error>
    where T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let cols = if insert_all {
            T::columns()
        } else {
            T::base_columns()
        };
        let mut sql: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO ");
        sql.push(T::table_name().to_string() + " (");

        let mut sepr = sql.separated(", ");
        cols.iter().for_each(|col| {sepr.push(col);});

        sql.push(") VALUES ( ");

        let mut sepr = sql.separated(", ");
        (1..=cols.len()).for_each(|param| {sepr.push(format!("${param}"));});

        sql.push(") RETURNING *;");

        let query = data.bind_values(sql.build_query_as::<T>(), BindVal::BASE);
        let res = query.fetch_one(&self.pool).await?;
        Ok(res)
    }

    pub async fn update<T>(&self, data: T) -> Result<T, sqlx::Error>
    where T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let mut sql: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE ");
        sql.push(T::table_name())
        .push(" SET (");

        let mut sepr = sql.separated(", ");
        T::base_columns().iter().for_each(|col| {sepr.push(col);});

        sql.push(") = (");

        let mut sepr = sql.separated(", ");
        ((T::id_columns().len() + 1)..=T::columns().len()).for_each(|param| {sepr.push(format!("${param}"));});

        sql.push(") WHERE (");

        let mut sepr = sql.separated(", ");
        T::id_columns().iter().for_each(|col| {sepr.push(col);});

        sql.push(") = (");

        let mut sepr = sql.separated(", ");
        (1..=T::id_columns().len()).for_each(|param| {sepr.push(format!("${param}"));});

        sql.push(") RETURNING *;");

        let query = data.bind_values(sql.build_query_as::<T>(), BindVal::ALL);
        let res = query.fetch_one(&self.pool).await?;
        Ok(res)
    }

    pub async fn delete<I, T>(&self, id_s: &[I]) -> Result<Vec<T>, sqlx::Error>
    where I: for<'q> Encode<'q, Postgres> + sqlx::Type<sqlx::Postgres>
    , T: Bindable + for<'r> FromRow<'r, PgRow> + Send + Unpin
    {

        let mut sql: QueryBuilder<Postgres> = QueryBuilder::new("DELETE FROM ");
        sql.push(T::table_name())
        .push(" WHERE (");

        let mut sepr = sql.separated(", ");
        T::id_columns().iter().for_each(|col| {sepr.push(col);});

        sql.push(") = (");
        
        let mut sepr = sql.separated(", ");
        (1..=T::id_columns().len()).for_each(|param| {sepr.push(format!("${param}"));});

        sql.push(") RETURNING *;");

        let mut query = sql.build_query_as::<T>();
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
        let mut sql: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM ");
        sql.push(T::table_name());

        if let Some(some_id_s) = id_s {
            if some_id_s.0.len() != some_id_s.1.len() {
                return Err(sqlx::Error::InvalidArgument("given tuple arrays have different sizes".to_string()));
            }

            sql.push(" WHERE (");

            let mut sepr = sql.separated(", ");
            some_id_s.0.iter().for_each(|col| {sepr.push(col);});

            sql.push(") = (");

            let mut sepr = sql.separated(", ");
            some_id_s.1.iter().for_each(|param| {sepr.push_bind(param);});

            sql.push(");");
        }
        else {
            sql.push(";");
        }

        let res = sql.build_query_as::<T>().fetch_all(&self.pool).await?;
        Ok(res)
    }

    pub async fn generic_exec(&self, query: Query<'_, Postgres, PgArguments>) -> Result<(), sqlx::Error>
    {
        query.execute(&self.pool).await?;
        Ok(())
    }

    pub async fn generic_fetch<T>(&self, query: QueryAs<'_, Postgres, T, PgArguments>) -> Result<Vec<T>, sqlx::Error>
    where T: for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let res = query.fetch_all(&self.pool).await?;
        Ok(res)
    }

}

impl LiteInterface {
    pub async fn new() -> Result<LiteInterface, sqlx::Error> {
        let initalizer_sql: &'static str = "CREATE TABLE IF NOT EXISTS logs(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            request TEXT NOT NULL,
            response TEXT NOT NULL,
            err_msg TEXT
        );";

        let opt: SqliteConnectOptions = sqlite::SqliteConnectOptions::new().filename(LOGS_PTH_STR).journal_mode(sqlite::SqliteJournalMode::Wal);
        let pool = sqlite::SqlitePoolOptions::new().max_connections(1).connect_with(opt).await?;

        sqlx::query(initalizer_sql).execute(&pool).await?;

        return Ok(LiteInterface { conn: pool });
    }

    pub async fn insert<I, O>(&self, logs: Logs<I, O>) -> Result<(), sqlx::Error> 
    where O: serde::Serialize,
    I: serde::Serialize
    {
        let sql2: &'static str = "INSERT INTO logs (request, response) VALUES (json($1), json($2))";
        let sql3: &'static str = "INSERT INTO logs (request, response, err_msg) VALUES (json($1), json($2), $3)";
        
        let (req, res) = (
            serde_json::to_string(&logs.request).unwrap_or("json parsing failed here".to_string()),
            serde_json::to_string(&logs.response).unwrap_or("json parsing failed here".to_string())
        );

        let query = if logs.err_msg.is_some() {
            sqlx::query(sql3).bind(req).bind(res).bind(logs.err_msg)
        } else {
            sqlx::query(sql2).bind(req).bind(res)
        };

        query.execute(&self.conn).await?;

        return Ok(());
    }
}