FROM minio/minio:RELEASE.2025-09-07T16-13-09Z-cpuv1

COPY ./start.sh /start.sh

ENTRYPOINT ["/start.sh"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD curl --fail --silent http://localhost:9000/minio/health/live || exit 1
