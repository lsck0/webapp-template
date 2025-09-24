FROM grafana/loki:3.5.5

COPY ./loki-config.yml /etc/loki/local-config.yml

CMD ["-config.file=/etc/loki/local-config.yml"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD wget -qO- http://localhost/ready | grep -q -w ready || exit 1
