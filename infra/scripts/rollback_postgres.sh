#!/bin/bash

set -e

echo "ARE U SURE?"
read -p "Type 'y' to continue: " confirm
if [ "$confirm" != "y" ]; then
    echo "Aborting rollback."
    exit 1
fi

echo "ARE U REALLY SURE?"
read -p "Type 'y' to continue: " confirm
if [ "$confirm" != "y" ]; then
    echo "Aborting rollback."
    exit 1
fi

echo "Configure the Script first!" && exit 1

# STEP 1: Load environment variables for the correct environment
export $(grep -v '^#' ../../../infra/env/dev.env | xargs)
POSTGRES_CONTAINER_NAME="wat-dev-postgres"

# STEP 2: Take down the database
docker compose stop $POSTGRES_CONTAINER_NAME

# STEP 3: Rollback the database to a previous state
docker compose run --rm $POSTGRES_CONTAINER_NAME \
  wal-g backup-fetch /var/lib/postgresql/data LATEST # <- PICK THE CORRECT BACKUP HERE wal-g backup-fetch /var/lib/postgresql/data <BACKUP_NAME>

# STEP 4: Use WAL log for point-in-time recovery (PITR)
