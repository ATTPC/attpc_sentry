// @generated automatically by Diesel CLI.

diesel::table! {
    status (id) {
        id -> Integer,
        path -> Text,
        dir_bytes -> Double,
        bytes_avail -> Double,
        total_bytes -> Double,
        n_files -> Integer,
    }
}