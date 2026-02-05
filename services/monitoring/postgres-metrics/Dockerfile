FROM prometheuscommunity/postgres-exporter:v0.17.1

ENV DATA_SOURCE_NAME="postgresql://root:1UdEQQTMBGCCWxvY2FsaG9zdIcEfwAAATAdBgNVHQ4EFgQUgGAHp2OojO06lrYu@postgres:5432/root?sslmode=disable"

EXPOSE 9187

HEALTHCHECK --interval=10s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -qO- http://localhost:9187/metrics > /dev/null 2>&1 || exit 1
