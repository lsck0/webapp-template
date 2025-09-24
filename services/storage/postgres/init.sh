#!/bin/env bash

set -e

sleep 30 && wal-g backup-push /var/lib/postgresql/data && echo "Initial Base Update Completed." &
