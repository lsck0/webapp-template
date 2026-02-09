FROM grafana/tempo:2.5.0

COPY tempo.yml /etc/tempo/tempo.yml

CMD ["-config.file=/etc/tempo/tempo.yml"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3200/ready || exit 1
