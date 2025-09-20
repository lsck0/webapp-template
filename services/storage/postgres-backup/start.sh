#!/bin/sh

while true; do
    echo 'Starting backup...';
    wal-g backup-push /var/lib/postgresql/data &&

    echo 'Backup completed. Deleting old backups...';
    wal-g delete --confirm retain 72 &&

    echo 'Old backups deleted. Sleeping for 1 hour...';
    sleep 3600;
done
