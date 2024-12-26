# attpc_sentry

A HTTP server to monitor the status of the AT-TPC DAQ workstations

## Installation

The only external dependency of attpc_sentry is the Rust toolchain. Installation instructions for
the Rust toolchain can be found [here](https://rust-lang.org).

Then to install attpc_sentry dowload this repository using 
`git clone https://github.com/gwm17/attpc_sentry.git`. After the repository is dowloaded move to
the repository root and run the following commands:

```bash
cargo install diesel_cli --no-default-features --features sqlite-bundled
diesel database setup
```

That will setup the SQLite database used to store the status. Then to run the sentry use

```bash
cargo run --release 
```

This will build and run the attpc_sentry program.

## Server API

To retrieve the current status of the directory and disk being monitored use the HTTP GET protocol at the route
`/status`. This will return JSON of the format:

```json
{
    "id": 1,
    "disk": "some_disk",
    "path": "/some/path/",
    "dir_gb": 0.0,
    "avail_gb": 0.0,
    "total_gb": 0.0,
    "n_files": 0
}
```

`id` is not relevant. `disk` is the name of the disk being monitored. `path` is the directory being monitored. `dir_gb`
is the total GB stored in files at `path`. `total_gb` is the total GB in `disk`. `avail_gb` is the unused GB on `disk`.
`n_files` is the number of files at `path`.

To change the directory and disk being monitored use the HTTP POST protocol at the route `/switch` with the following
JSON payload:

```json
{
    "path": "/some/path/",
    "disk": "some_disk"
}
```

By default, attpc_sentry monitors the `/Volumes` directory on the `Macintosh HD` disk. If you plan to use the sentry on 
a non-MacOS device, you will need to change these defaults which are found in the `.env` file in the repository as 
`DEFAULT_PATH` and `DEFAULT_DISK` variables.

## Advanced Settings

There is a `.env` file which includes the SQLite database url (the `DATABASE_URL` variable). You can change
this to point whatever location you want.
