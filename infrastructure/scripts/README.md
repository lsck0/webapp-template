# PostgreSQL Backup and Restore Scripts

This directory contains scripts for managing PostgreSQL backups and restores in the local development environment.

## Prerequisites

- Docker and docker compose installed
- The development environment configured (`infrastructure/env/dev.env`)
- MinIO running (for backup storage)
- PostgreSQL configured with WAL-G for backups

## Available Scripts

### `list_postgres_backups.sh`

Lists all available PostgreSQL backups stored in MinIO.

**Usage:**
```bash
./list_postgres_backups.sh
```

**Output:**
- Displays a table of available backups with their details
- Shows backup name, modification time, and size

### `restore_postgres.sh`

Restores the PostgreSQL database from a WAL-G backup.

**Usage:**
```bash
# Restore the latest backup
./restore_postgres.sh

# Restore a specific backup
./restore_postgres.sh <backup_name>
```

**What it does:**
1. Stops the running PostgreSQL container
2. Removes the existing data volume
3. Creates a new empty volume
4. Fetches the backup from MinIO using WAL-G
5. Sets up recovery signal for point-in-time recovery
6. Starts PostgreSQL with the restored data
7. Waits for PostgreSQL to become healthy

**⚠️ Warning:** This will destroy the current database data!

### `test_postgres_restore.sh`

Comprehensive test suite for the PostgreSQL backup and restore functionality.

**Usage:**
```bash
./test_postgres_restore.sh
```

**What it tests:**
1. Creates test data in PostgreSQL
2. Creates a WAL-G backup
3. Verifies the backup exists in MinIO
4. Simulates data loss by dropping the test table
5. Restores from the backup
6. Verifies the data was restored correctly
7. Cleans up test artifacts

**This is a non-destructive test** - it only affects test tables.

## How Backups Work

The PostgreSQL container is configured to:

1. **Continuous Archiving**: WAL (Write-Ahead Log) files are continuously archived to MinIO using `wal-g wal-push`
2. **Periodic Full Backups**: A full backup is created every 24 hours via the `wal-g-backup-loop.sh` script
3. **Backup Retention**: Old backups are automatically cleaned up (retains 14 days of backups)
4. **Storage**: All backups are stored in MinIO under the `pg-backups` bucket

## Configuration

Backup configuration is defined in `infrastructure/env/dev.env`:

```bash
# MinIO/S3 Configuration
WALE_S3_PREFIX=s3://pg-backups
AWS_ENDPOINT=http://minio:9000
AWS_ACCESS_KEY_ID=admin
AWS_SECRET_ACCESS_KEY=password
AWS_S3_FORCE_PATH_STYLE=true
WALG_COMPRESSION_METHOD=lz4
```

## Troubleshooting

### Backup List Shows No Backups

1. Ensure MinIO is running: `docker ps | grep minio`
2. Check if the pg-backups bucket exists in MinIO
3. Verify environment variables are loaded correctly

### Restore Fails

1. Check MinIO is accessible from the postgres container
2. Verify the backup name is correct
3. Check postgres container logs: `docker logs wat-dev-postgres`
4. Ensure the data volume is not in use by another container

### WAL-G Commands Fail

The WAL-G commands need AWS environment variables. They are automatically loaded from `dev.env` by the scripts.

## Related Files

- `../services/storage/postgres/wal-g-backup-loop.sh` - Backup loop that runs inside postgres container
- `../services/storage/postgres/postgresql.conf` - PostgreSQL configuration for WAL archiving
- `../services/storage/postgres/Dockerfile` - Builds postgres with WAL-G installed
