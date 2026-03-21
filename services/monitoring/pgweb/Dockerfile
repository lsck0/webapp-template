FROM sosedoff/pgweb:0.17.0

USER root
RUN apt-get update && apt-get install -y wget
USER pgweb

ENV PGWEB_DATABASE_URL=postgres://root:1UdEQQTMBGCCWxvY2FsaG9zdIcEfwAAATAdBgNVHQ4EFgQUgGAHp2OojO06lrYu@postgres:5432/root?sslmode=disable

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD wget -qO- http://localhost:8081/ || exit 1
