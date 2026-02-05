FROM oliver006/redis_exporter:v1.67.0-alpine

EXPOSE 9121

HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:9121/health || exit 1
