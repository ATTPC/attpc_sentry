CREATE TABLE status (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL,
    dir_bytes DOUBLE NOT NULL,
    bytes_avail DOUBLE NOT NULL,
    total_bytes DOUBLE NOT NULL,
    n_files INTEGER NOT NULL
)
