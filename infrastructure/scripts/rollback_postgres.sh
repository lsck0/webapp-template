#!/usr/bin/env bash
#
# DEPRECATED: This script has been replaced by restore_postgres.sh
#
# Please use one of the following scripts instead:
#
#   - restore_postgres.sh     - Restore postgres from a backup
#   - list_postgres_backups.sh - List available backups
#   - test_postgres_restore.sh - Test the backup/restore functionality
#
# For more information, see the infrastructure/scripts directory.

set -e

echo "This script is deprecated."
echo ""
echo "Please use restore_postgres.sh instead:"
echo "  ./restore_postgres.sh [backup_name]"
echo ""
echo "To list available backups:"
echo "  ./list_postgres_backups.sh"
echo ""
echo "To test backup/restore functionality:"
echo "  ./test_postgres_restore.sh"
echo ""
exit 1
