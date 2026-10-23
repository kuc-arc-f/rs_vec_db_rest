#!/bin/bash

export TOKIO_WORKER_THREADS=4

#sqlite3 ./data/backup.db .dump > ./data/backup.sql

./target/release/vec_db_api_rest
