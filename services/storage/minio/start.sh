#!/bin/env bash

set -ex

# start minio server
minio server /data --console-address :3002 &

sleep 10

# configure mc
mc alias set localminio $MINIO_URL $MINIO_ROOT_USER $MINIO_ROOT_PASSWORD

# create buckets
mc mb --ignore-existing localminio/pg-backups

wait
