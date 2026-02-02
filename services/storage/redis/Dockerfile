FROM redis:8.4.0-alpine

EXPOSE 6379

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD redis-cli ping | grep -q PONG || exit 1
