FROM redis:7.4-alpine

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD redis-cli ping | grep -q PONG || exit 1
