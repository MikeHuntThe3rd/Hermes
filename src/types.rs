use serde::{Serialize, Deserialize};
use sqlx::{Postgres, postgres::PgArguments, query::QueryAs};
use uuid::Uuid;

pub trait Bindable<T> {
    fn table_name() -> &'static str;
    fn columns() -> &'static [&'static str];

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, T, PgArguments>)
        -> QueryAs<'lftm, Postgres, T, PgArguments>;
}

#[derive(Deserialize)]
pub struct UninitializedUser {
    pub username: String,
    pub password: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password: String,
}

impl Bindable<User> for UninitializedUser {
    fn table_name() -> &'static str {
        return "users";
    }

    fn columns() -> &'static [&'static str] {
        return &["username", "password"];
    }

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, User, PgArguments>)
        -> QueryAs<'lftm, Postgres, User, PgArguments> {
            return query.bind(&self.username).bind(&self.password);
        }
}

impl Bindable<User> for User {
    fn table_name() -> &'static str {
        return "users";
    }

    fn columns() -> &'static [&'static str] {
        return &["id", "username", "password"];
    }

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, User, PgArguments>)
        -> QueryAs<'lftm, Postgres, User, PgArguments> {
            return query.bind(&self.id).bind(&self.username).bind(&self.password);
        }
}