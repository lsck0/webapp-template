#!/usr/bin/env bash
#
# List PostgreSQL Backups Script for Local Development
#
# This script lists all available WAL-G backups stored in MinIO.
#
# Usage: ./list_postgres_backups.sh

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$INFRA_DIR/env/dev.env"
COMPOSE_FILE="$INFRA_DIR/dev.compose.yml"
POSTGRES_SERVICE="postgres"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "=========================================="
echo "PostgreSQL Backups List"
echo "=========================================="
echo ""

# Load environment variables
if [ ! -f "$ENV_FILE" ]; then
    echo -e "${RED}Error: Environment file not found at $ENV_FILE${NC}"
    exit 1
fi

set -a
source "$ENV_FILE"
set +a

echo -e "${YELLOW}Fetching backup list from MinIO...${NC}"
echo ""

# Check if minio is running
if ! docker ps | grep -q "wat-dev-minio"; then
    echo -e "${YELLOW}MinIO is not running. Starting it...${NC}"
    docker compose -f "$COMPOSE_FILE" up -d minio
    sleep 3
fi

# Run wal-g backup-list in postgres container
docker compose -f "$COMPOSE_FILE" run --rm --entrypoint "" \
    -e AWS_ENDPOINT="$AWS_ENDPOINT" \
    -e AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
    -e AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
    -e AWS_S3_FORCE_PATH_STYLE="$AWS_S3_FORCE_PATH_STYLE" \
    -e WALE_S3_PREFIX="$WALE_S3_PREFIX" \
    -e WALG_COMPRESSION_METHOD="$WALG_COMPRESSION_METHOD" \
    "$POSTGRES_SERVICE" \
    wal-g backup-list

BACKUP_LIST_STATUS=$?

if [ $BACKUP_LIST_STATUS -ne 0 ]; then
    echo ""
    echo -e "${RED}Failed to list backups. Make sure MinIO is running and has backups.${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}To restore a specific backup, run:${NC}"
echo "  ./restore_postgres.sh <backup_name>"
echo ""
echo -e "${GREEN}To restore the latest backup, run:${NC}"
echo "  ./restore_postgres.sh"
echo ""
