use derive_macros::Bindable;
use serde::{Serialize, Deserialize};
use sqlx::{Postgres, postgres::PgArguments, query::QueryAs};
use uuid::Uuid;

#[derive(PartialEq)]
pub enum BindVal {
    ID,
    BASE,
    ALL,
}

pub trait Bindable {
    fn table_name() -> &'static str;

    fn columns() -> &'static [&'static str];
    fn id_columns() -> &'static [&'static str];
    fn base_columns() -> &'static [&'static str];

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, Self, PgArguments>,
        bind_val: BindVal)
        -> QueryAs<'lftm, Postgres, Self, PgArguments>
        where Self: Sized;
}

#[derive(Serialize)]
pub struct Response<T>
{
    pub success: bool,
    pub msg: String,
    pub data: Option<T>,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Bindable)]
#[ids = "id"]
pub struct User {
    pub id: Option<Uuid>,
    pub username: String,
    pub password: String,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Group {
    pub id: Option<Uuid>,
    pub name: String,
    pub is_dm: bool,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct Message {
    pub id: Option<i32>,
    pub message: Option<String>,
    pub files: Option<Vec<Vec<u8>>>,
    pub group_id: Uuid,
    pub user_id: Uuid,
}

#[derive(sqlx::FromRow, Serialize, Deserialize)]
pub struct GroupMembers {
    pub group_id: Uuid,
    pub member_id: Uuid,
}

//impls
// impl Bindable for User {
//     fn table_name() -> &'static str {
//         return "users";
//     }

//     fn id_columns() -> &'static [&'static str] {
//         return &["id"];
//     }

//     fn base_columns() -> &'static [&'static str] {
//         return &["username", "password"];
//     }

//     fn columns() -> &'static [&'static str] {
//         return &["id", "username", "password"];
//     }

//     fn bind_values<'lftm>(
//         &'lftm self,
//         query: QueryAs<'lftm, Postgres, Self, PgArguments>,
//         bind_val: BindVal)
//         -> QueryAs<'lftm, Postgres, Self, PgArguments> 
//     {
//         let mut res = query;
        
//         if bind_val == BindVal::ID {
//             return res.bind(&self.id);
//         }
//         else if bind_val == BindVal::ALL {
//             res = res.bind(&self.id);
//         }

//         return res.bind(&self.username).bind(&self.password);
//     }
// }

impl Bindable for Group {
    fn table_name() -> &'static str {
        return "groups";
    }

    fn id_columns() -> &'static [&'static str] {
        return &["id"];
    }

    fn base_columns() -> &'static [&'static str] {
        return &["name", "is_dm"];
    }

    fn columns() -> &'static [&'static str] {
        return &["id", "name", "is_dm"];
    }

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, Self, PgArguments>,
        bind_val: BindVal)
        -> QueryAs<'lftm, Postgres, Self, PgArguments> 
    {
        let mut res = query;
        
        if bind_val == BindVal::ID {
            return res.bind(&self.id);
        }
        else if bind_val == BindVal::ALL {
            res = res.bind(&self.id);
        }

        return res.bind(&self.name).bind(&self.is_dm);
    }
}

impl Bindable for Message {
     fn table_name() -> &'static str {
        return "messages";
    }

    fn id_columns() -> &'static [&'static str] {
        return &["id"];
    }

    fn base_columns() -> &'static [&'static str] {
        return &["message", "files", "group_id", "user_id"];
    }

    fn columns() -> &'static [&'static str] {
        return &["id", "message", "files", "group_id", "user_id"];
    }

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, Self, PgArguments>,
        bind_val: BindVal)
        -> QueryAs<'lftm, Postgres, Self, PgArguments> 
    {
        let mut res = query;
        
        if bind_val == BindVal::ID {
            return res.bind(&self.id);
        }
        else if bind_val == BindVal::ALL {
            res = res.bind(&self.id);
        }

        return res.bind(&self.message).bind(&self.files).bind(&self.group_id).bind(&self.user_id);
    }
}

impl Bindable for GroupMembers {
     fn table_name() -> &'static str {
        return "users";
    }

    fn id_columns() -> &'static [&'static str] {
        return &["group_id", "member_id"];
    }

    fn base_columns() -> &'static [&'static str] {
        return &[];
    }

    fn columns() -> &'static [&'static str] {
        return &["group_id", "member_id"];
    }

    fn bind_values<'lftm>(
        &'lftm self,
        query: QueryAs<'lftm, Postgres, Self, PgArguments>,
        bind_val: BindVal)
        -> QueryAs<'lftm, Postgres, Self, PgArguments> 
    {
        let mut res = query;
        
        if bind_val == BindVal::ID {
            return res.bind(&self.group_id).bind(&self.member_id);
        }
        else if bind_val == BindVal::ALL {
            return res.bind(&self.group_id).bind(&self.member_id);
        }

        return res;
    }
}



