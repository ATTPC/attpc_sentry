# attpc_sentry

A HTTP server to monitor the status of the AT-TPC DAQ workstations and
organize run datafiles.

## Installation

The only external dependency of attpc_sentry is the Rust toolchain. Installation instructions for
the Rust toolchain can be found [here](https://rust-lang.org).

Then to install attpc_sentry dowload this repository using 
`git clone https://github.com/gwm17/attpc_sentry.git`. After the repository is dowloaded move to
the repository root and run the following command:

```bash
cargo run --release 
```

This will build and run the attpc_sentry program.

## Server API

The sentry server has two endpoints, `/status` and `/catalogue`. `/status` will query the status of 
a path and disk to return the associated disk usage statistics. `/catalogue` will move the DAQ
run datafiles to a run-specific location. To use each endpoint POST the following JSON:

```json
{
    "disk": "some_disk",
    "path": "/some/path",
    "experiment": "experiment",
    "run_number": 0
}
```

`disk` is the name of the disk (for AT-TPC typically this is "Macintosh HD"), `path` is the path
to which the DAQ writes data, `experiment` is the unqiue experiment name (something like "e22508"),
and `run_number` is the current run number.

Both endpoints return the status as the following JSON:

```json
{
    "disk": "some_disk",
    "path": "/some/path/",
    "path_gb": 0.0,
    "path_n_files": 0,
    "disk_avail_gb": 0.0,
    "disk_total_gb": 0.0
}
```

`disk` and `path` mirror the input JSON. `path_gb` is the total GB stored in files at `path`. 
`disk_total_gb` is the total GB in `disk`. `disk_avail_gb` is the unused GB on `disk`. `path_n_files` is the number of files at `path`.
