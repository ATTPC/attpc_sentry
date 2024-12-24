use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::status)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct DirectoryStatus {
    pub id: i32,
    pub path: String,
    pub dir_bytes: f64,   // GB
    pub bytes_avail: f64, // GB
    pub total_bytes: f64, // GB
    pub n_files: i32,
}
