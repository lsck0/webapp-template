#!/usr/bin/env bash
#
# PostgreSQL Restore Script for Local Development
#
# This script restores the PostgreSQL database from a WAL-G backup stored in MinIO.
# It is designed for local development/testing purposes.
#
# Usage: ./restore_postgres.sh [backup_name]
#   - If backup_name is not provided, it will use LATEST
#
# Prerequisites:
#   - Docker and docker compose must be installed
#   - The dev environment must be running (or at least minio and postgres services)
#   - A backup must exist in MinIO (s3://pg-backups)

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$INFRA_DIR/env/dev.env"
COMPOSE_FILE="$INFRA_DIR/dev.compose.yml"
POSTGRES_SERVICE="postgres"
POSTGRES_CONTAINER="wat-dev-postgres"
DATA_VOLUME="dev_postgres_data"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
BACKUP_NAME="${1:-LATEST}"

echo "=========================================="
echo "PostgreSQL Restore Script (Local Dev)"
echo "=========================================="
echo ""
echo "This will restore the PostgreSQL database from backup: $BACKUP_NAME"
echo "WARNING: This will DESTROY the current database data!"
echo ""

# Confirm with user
read -p "Are you sure you want to continue? (yes/no): " confirm
if [ "$confirm" != "yes" ]; then
    echo -e "${RED}Aborting restore.${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Step 1: Loading environment variables...${NC}"
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}Error: Environment file not found at $ENV_FILE${NC}"
    exit 1
fi

# Export environment variables from dev.env
set -a
source "$ENV_FILE"
set +a

echo -e "${GREEN}Environment loaded successfully${NC}"

echo ""
echo -e "${YELLOW}Step 2: Checking if postgres container is running...${NC}"
if docker ps | grep -q "$POSTGRES_CONTAINER"; then
    echo -e "${YELLOW}Stopping postgres container...${NC}"
    docker compose -f "$COMPOSE_FILE" stop "$POSTGRES_SERVICE"
    echo -e "${GREEN}Container stopped${NC}"
else
    echo -e "${GREEN}Container is not running${NC}"
fi

echo ""
echo -e "${YELLOW}Step 3: Removing existing postgres data volume...${NC}"
# Remove the container if it exists
if docker ps -a | grep -q "$POSTGRES_CONTAINER"; then
    docker rm "$POSTGRES_CONTAINER" 2>/dev/null || true
fi

# Remove the volume
if docker volume ls | grep -q "$DATA_VOLUME"; then
    docker volume rm "$DATA_VOLUME"
    echo -e "${GREEN}Volume removed${NC}"
else
    echo -e "${GREEN}Volume does not exist${NC}"
fi

echo ""
echo -e "${YELLOW}Step 4: Creating new empty volume...${NC}"
docker volume create "$DATA_VOLUME"
echo -e "${GREEN}Volume created${NC}"

echo ""
echo -e "${YELLOW}Step 5: Starting postgres container temporarily to restore backup...${NC}"

# Start postgres container with entrypoint override to keep it running for restore
docker compose -f "$COMPOSE_FILE" run --rm --entrypoint "" \
    -e AWS_ENDPOINT="$AWS_ENDPOINT" \
    -e AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
    -e AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
    -e AWS_S3_FORCE_PATH_STYLE="$AWS_S3_FORCE_PATH_STYLE" \
    -e WALE_S3_PREFIX="$WALE_S3_PREFIX" \
    -e WALG_COMPRESSION_METHOD="$WALG_COMPRESSION_METHOD" \
    -v "$DATA_VOLUME:/var/lib/postgresql/data" \
    "$POSTGRES_SERVICE" \
    bash -c "
        set -e
        echo 'Fetching backup from MinIO...'
        wal-g backup-fetch /var/lib/postgresql/data $BACKUP_NAME
        echo 'Backup fetched successfully'
        
        echo 'Creating recovery.signal for PITR...'
        touch /var/lib/postgresql/data/recovery.signal
        
        echo 'Setting correct ownership...'
        chown -R postgres:postgres /var/lib/postgresql/data
        
        echo 'Restore complete!'
    "

RESTORE_STATUS=$?

if [ $RESTORE_STATUS -ne 0 ]; then
    echo -e "${RED}Restore failed with exit code $RESTORE_STATUS${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}Step 6: Backup restored successfully!${NC}"

echo ""
echo -e "${YELLOW}Step 7: Starting postgres container with restored data...${NC}"
docker compose -f "$COMPOSE_FILE" up -d "$POSTGRES_SERVICE"

echo ""
echo -e "${YELLOW}Waiting for postgres to be healthy...${NC}"
max_attempts=30
attempt=1
while [ $attempt -le $max_attempts ]; do
    if docker ps | grep "$POSTGRES_CONTAINER" | grep -q "healthy"; then
        echo -e "${GREEN}Postgres is healthy!${NC}"
        break
    fi
    echo "Attempt $attempt/$max_attempts: Waiting for postgres to become healthy..."
    sleep 5
    attempt=$((attempt + 1))
done

if [ $attempt -gt $max_attempts ]; then
    echo -e "${RED}Postgres did not become healthy within the expected time${NC}"
    echo -e "${YELLOW}Check logs with: docker logs $POSTGRES_CONTAINER${NC}"
    exit 1
fi

echo ""
echo "=========================================="
echo -e "${GREEN}PostgreSQL restore completed successfully!${NC}"
echo "=========================================="
echo ""
echo "Backup used: $BACKUP_NAME"
echo "Postgres container: $POSTGRES_CONTAINER"
echo ""
echo "To verify the restore, connect to the database:"
echo "  docker compose -f $COMPOSE_FILE exec postgres psql -U \$POSTGRES_USER -d \$POSTGRES_DB"
echo ""
