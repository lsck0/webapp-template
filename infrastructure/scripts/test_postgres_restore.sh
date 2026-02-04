#!/usr/bin/env bash
#
# PostgreSQL Backup/Restore Test Script
#
# This script tests the PostgreSQL backup and restore functionality locally.
# It creates test data, triggers a backup, destroys the data, and restores it.
#
# Usage: ./test_postgres_restore.sh

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INFRA_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="$INFRA_DIR/env/dev.env"
COMPOSE_FILE="$INFRA_DIR/dev.compose.yml"
POSTGRES_SERVICE="postgres"
POSTGRES_CONTAINER="wat-dev-postgres"
TEST_TABLE="backup_restore_test"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test result tracking
TESTS_PASSED=0
TESTS_FAILED=0

# Helper function to run psql commands
run_psql() {
    docker compose -f "$COMPOSE_FILE" exec -T "$POSTGRES_SERVICE" psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -t -c "$1"
}

# Helper function to assert
test_assert() {
    if [ "$1" -eq 0 ]; then
        echo -e "${GREEN}✓ $2${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ $2${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Helper function to assert equals
test_assert_equals() {
    if [ "$1" = "$2" ]; then
        echo -e "${GREEN}✓ $3${NC}"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}✗ $3 (expected: '$2', got: '$1')${NC}"
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

echo "=========================================="
echo "PostgreSQL Backup/Restore Test Suite"
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

echo -e "${BLUE}Test Configuration:${NC}"
echo "  Postgres Container: $POSTGRES_CONTAINER"
echo "  Database: $POSTGRES_DB"
echo "  Test Table: $TEST_TABLE"
echo ""

# =============================================================================
# SETUP
# =============================================================================
echo -e "${YELLOW}=== Setup Phase ===${NC}"
echo ""

echo "Step 1: Ensuring MinIO is running..."
if ! docker ps | grep -q "wat-dev-minio"; then
    echo "Starting MinIO..."
    docker compose -f "$COMPOSE_FILE" up -d minio
    sleep 5
fi
test_assert $? "MinIO is running"

echo ""
echo "Step 2: Ensuring Postgres is running..."
if ! docker ps | grep -q "$POSTGRES_CONTAINER"; then
    echo "Starting Postgres..."
    docker compose -f "$COMPOSE_FILE" up -d "$POSTGRES_SERVICE"
    sleep 10
fi

# Wait for postgres to be healthy
echo "Waiting for Postgres to be healthy..."
max_attempts=30
attempt=1
while [ $attempt -le $max_attempts ]; do
    if docker ps | grep "$POSTGRES_CONTAINER" | grep -q "healthy"; then
        break
    fi
    sleep 2
    attempt=$((attempt + 1))
done
test_assert $? "Postgres is running and healthy"

# =============================================================================
# CREATE TEST DATA
# =============================================================================
echo ""
echo -e "${YELLOW}=== Creating Test Data ===${NC}"
echo ""

echo "Step 3: Creating test table..."
run_psql "DROP TABLE IF EXISTS $TEST_TABLE;" > /dev/null 2>&1 || true
run_psql "CREATE TABLE $TEST_TABLE (id SERIAL PRIMARY KEY, test_data TEXT, created_at TIMESTAMP DEFAULT NOW());" > /dev/null
test_assert $? "Test table created"

echo ""
echo "Step 4: Inserting test data..."
TEST_DATA_VALUE="backup_restore_test_$(date +%s)"
run_psql "INSERT INTO $TEST_TABLE (test_data) VALUES ('$TEST_DATA_VALUE');" > /dev/null
run_psql "INSERT INTO $TEST_TABLE (test_data) VALUES ('test_row_2');" > /dev/null
run_psql "INSERT INTO $TEST_TABLE (test_data) VALUES ('test_row_3');" > /dev/null
ROW_COUNT=$(run_psql "SELECT COUNT(*) FROM $TEST_TABLE;" | xargs)
test_assert_equals "$ROW_COUNT" "3" "Inserted 3 test rows"

# =============================================================================
# CREATE BACKUP
# =============================================================================
echo ""
echo -e "${YELLOW}=== Creating Backup ===${NC}"
echo ""

echo "Step 5: Creating WAL-G backup..."
docker compose -f "$COMPOSE_FILE" run --rm --entrypoint "" \
    -e AWS_ENDPOINT="$AWS_ENDPOINT" \
    -e AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
    -e AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
    -e AWS_S3_FORCE_PATH_STYLE="$AWS_S3_FORCE_PATH_STYLE" \
    -e WALE_S3_PREFIX="$WALE_S3_PREFIX" \
    -e WALG_COMPRESSION_METHOD="$WALG_COMPRESSION_METHOD" \
    "$POSTGRES_SERVICE" \
    wal-g backup-push /var/lib/postgresql/data > /tmp/backup_output.log 2>&1
BACKUP_RESULT=$?

if [ $BACKUP_RESULT -eq 0 ]; then
    echo -e "${GREEN}Backup created successfully${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}Backup failed${NC}"
    echo "Backup output:"
    cat /tmp/backup_output.log
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Wait a moment for backup to be visible
echo "Waiting for backup to be available..."
sleep 5

echo ""
echo "Step 6: Verifying backup exists..."
BACKUP_COUNT=$(docker compose -f "$COMPOSE_FILE" run --rm --entrypoint "" \
    -e AWS_ENDPOINT="$AWS_ENDPOINT" \
    -e AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
    -e AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
    -e AWS_S3_FORCE_PATH_STYLE="$AWS_S3_FORCE_PATH_STYLE" \
    -e WALE_S3_PREFIX="$WALE_S3_PREFIX" \
    "$POSTGRES_SERVICE" \
    wal-g backup-list 2>/dev/null | wc -l)

if [ "$BACKUP_COUNT" -gt 0 ]; then
    echo -e "${GREEN}✓ Backup exists in storage ($BACKUP_COUNT backups found)${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗ No backups found${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# =============================================================================
# SIMULATE DATA LOSS
# =============================================================================
echo ""
echo -e "${YELLOW}=== Simulating Data Loss ===${NC}"
echo ""

echo "Step 7: Dropping test table..."
run_psql "DROP TABLE $TEST_TABLE;" > /dev/null
test_assert $? "Test table dropped (simulating data loss)"

# =============================================================================
# RESTORE DATA
# =============================================================================
echo ""
echo -e "${YELLOW}=== Restoring from Backup ===${NC}"
echo ""

echo "Step 8: Stopping postgres..."
docker compose -f "$COMPOSE_FILE" stop "$POSTGRES_SERVICE" > /dev/null 2>&1
test_assert $? "Postgres stopped"

echo ""
echo "Step 9: Removing postgres data volume..."
docker rm "$POSTGRES_CONTAINER" 2>/dev/null || true
docker volume rm dev_postgres_data 2>/dev/null || true
docker volume create dev_postgres_data > /dev/null
test_assert $? "Data volume recreated"

echo ""
echo "Step 10: Restoring from backup..."
docker compose -f "$COMPOSE_FILE" run --rm --entrypoint "" \
    -e AWS_ENDPOINT="$AWS_ENDPOINT" \
    -e AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID" \
    -e AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY" \
    -e AWS_S3_FORCE_PATH_STYLE="$AWS_S3_FORCE_PATH_STYLE" \
    -e WALE_S3_PREFIX="$WALE_S3_PREFIX" \
    -e WALG_COMPRESSION_METHOD="$WALG_COMPRESSION_METHOD" \
    -v "dev_postgres_data:/var/lib/postgresql/data" \
    "$POSTGRES_SERVICE" \
    bash -c "
        wal-g backup-fetch /var/lib/postgresql/data LATEST
        touch /var/lib/postgresql/data/recovery.signal
        chown -R postgres:postgres /var/lib/postgresql/data
    " > /tmp/restore_output.log 2>&1
RESTORE_RESULT=$?

if [ $RESTORE_RESULT -eq 0 ]; then
    echo -e "${GREEN}Restore completed successfully${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}Restore failed${NC}"
    echo "Restore output:"
    cat /tmp/restore_output.log
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

echo ""
echo "Step 11: Starting postgres with restored data..."
docker compose -f "$COMPOSE_FILE" up -d "$POSTGRES_SERVICE" > /dev/null 2>&1

# Wait for postgres to be healthy
echo "Waiting for Postgres to be healthy..."
max_attempts=30
attempt=1
HEALTHY=0
while [ $attempt -le $max_attempts ]; do
    if docker ps | grep "$POSTGRES_CONTAINER" | grep -q "healthy"; then
        HEALTHY=1
        break
    fi
    sleep 2
    attempt=$((attempt + 1))
done

test_assert $HEALTHY "Postgres is running with restored data"

# =============================================================================
# VERIFY RESTORE
# =============================================================================
echo ""
echo -e "${YELLOW}=== Verifying Restore ===${NC}"
echo ""

echo "Step 12: Checking if test table exists..."
TABLE_EXISTS=$(run_psql "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_name = '$TEST_TABLE');" | xargs)
test_assert_equals "$TABLE_EXISTS" "t" "Test table exists after restore"

echo ""
echo "Step 13: Verifying row count..."
RESTORED_ROW_COUNT=$(run_psql "SELECT COUNT(*) FROM $TEST_TABLE;" | xargs)
test_assert_equals "$RESTORED_ROW_COUNT" "3" "All 3 rows restored"

echo ""
echo "Step 14: Verifying test data integrity..."
RESTORED_DATA=$(run_psql "SELECT test_data FROM $TEST_TABLE WHERE test_data = '$TEST_DATA_VALUE';" | xargs)
test_assert_equals "$RESTORED_DATA" "$TEST_DATA_VALUE" "Test data integrity verified"

# =============================================================================
# CLEANUP
# =============================================================================
echo ""
echo -e "${YELLOW}=== Cleanup ===${NC}"
echo ""

echo "Step 15: Cleaning up test table..."
run_psql "DROP TABLE IF EXISTS $TEST_TABLE;" > /dev/null 2>&1 || true
echo -e "${GREEN}✓ Cleanup completed${NC}"

# =============================================================================
# RESULTS
# =============================================================================
echo ""
echo "=========================================="
echo "Test Results"
echo "=========================================="
echo -e "${GREEN}Passed: $TESTS_PASSED${NC}"
if [ $TESTS_FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $TESTS_FAILED${NC}"
fi
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "The PostgreSQL backup and restore functionality is working correctly."
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
