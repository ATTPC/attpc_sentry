CREATE TABLE status (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    disk TEXT NOT NULL,
    path TEXT NOT NULL,
    dir_bytes DOUBLE NOT NULL,
    bytes_avail DOUBLE NOT NULL,
    total_bytes DOUBLE NOT NULL,
    n_files INTEGER NOT NULL
)
