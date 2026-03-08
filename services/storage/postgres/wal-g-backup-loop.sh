#!/usr/bin/env bash

set -e

sleep 300

while true; do
    echo "$(date '+%F %T') - Starting WAL-G backup"

    wal-g backup-push /var/lib/postgresql/data
    if [ $? -eq 0 ]; then
      echo "$(date '+%F %T') - Backup completed successfully"
    else
      echo "$(date '+%F %T') - Backup failed" >&2
    fi

    wal-g delete retain 14 --confirm
    if [ $? -eq 0 ]; then
      echo "$(date '+%F %T') - Old backups deleted successfully"
    else
      echo "$(date '+%F %T') - Failed to delete old backups" >&2
    fi

    sleep 86400 # one day
done
