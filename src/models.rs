use chrono::prelude::NaiveDateTime;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;

use serde::{Deserialize, Serialize};

use crate::schema::idioms;

#[derive(Serialize, Deserialize, Debug, Queryable, Selectable)]
#[diesel(table_name = idioms)]
#[diesel(check_for_backend(Sqlite))]
pub struct Idiom {
    pub id: i32,
    pub phrase: String,
    pub example: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = idioms)]
pub struct NewIdiom {
    pub phrase: String,
    pub example: Option<String>,
}
