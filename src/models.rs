use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, AsChangeset, Insertable)]
#[diesel(table_name = crate::schema::status)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MachineStatus {
    pub id: i32,
    pub disk: String,
    pub path: String,
    pub dir_gb: f64,
    pub avail_gb: f64,
    pub total_gb: f64,
    pub n_files: i32,
}
