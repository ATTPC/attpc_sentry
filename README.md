# attpc_sentry
![CI](https://github.com/ATTPC/attpc_sentry/actions/workflows/ci.yml/badge.svg)

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

The sentry server has three endpoints

- `/status`: query the status of data on the workstation 
- `/catalog`: move the DAQ run datafiles to a run-specific location
- `/backup`: backup the DAQ .xcfg configuration files 

All endpoints return the status as the following JSON:

```json
{
    "disk": "some_disk",
    "process": "some_process",
    "data_path": "/some/path/",
    "data_written_gb": 0.0,
    "data_path_files": 0,
    "disk_avail_gb": 0.0,
    "disk_total_gb": 0.0
}
```


### Status Route

The status route checks the status of the dataRouter/DataExporter
process, the amount of data written (since last check). This route
is accessed using an HTTP GET request.

### Catalog Route

This route moves DAQ data files to a experiment/run specific location
This route is accessed using HTTP POST request with the following
JSON data

```json
{
    "experiment": "some_experiment",
    "run_number": 0,
}
```

### Backup Route

This route moves DAQ configuration files to a experiment/run specific location
This route is accessed using HTTP POST request with the following
JSON data

```json
{
    "experiment": "some_experiment",
    "run_number": 0,
}
```

## Environment variables

The following variables should be defined in a `.env` file at the location
from which attpc_sentry should be run.

```bash
DISK_NAME="Macintosh HD"
DATA_PATH="/Users/attpc/Data"
PROCESS_NAME="dataRouter"
CONFIG_PATH="/Users/attpc/configs/"
CONFIG_BACKUP_PATH="/Users/attpc/configs_backup/"
```

## Extra scripts

Also included in the repo are two scripts: `attpc.sentry.plist` and `mmStartSentry.sh`
These are used in the AT-TPC DAQ workstations to autostart the sentry with the other 
DAQ tools. Generally they won't need modified.

