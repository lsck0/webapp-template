FROM grafana/loki:2.9.15

COPY ./loki-config.yml /etc/loki/local-config.yml

CMD ["-config.file=/etc/loki/local-config.yml"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD wget -qO- http://localhost/ready | grep -q -w ready || exit 1
