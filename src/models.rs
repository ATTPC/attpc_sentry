use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::status)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MachineStatus {
    pub id: i32,
    pub disk: String,
    pub path: String,
    pub dir_bytes: f64,   // GB
    pub bytes_avail: f64, // GB
    pub total_bytes: f64, // GB
    pub n_files: i32,
}
