// @generated automatically by Diesel CLI.

diesel::table! {
    status (id) {
        id -> Integer,
        disk -> Text,
        path -> Text,
        dir_gb -> Double,
        avail_gb -> Double,
        total_gb -> Double,
        n_files -> Integer,
    }
}
