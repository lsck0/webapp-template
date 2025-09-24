FROM prom/prometheus:v3.6.0

COPY ./prometheus.yml /etc/prometheus/prometheus.yml

CMD ["--config.file=/etc/prometheus/prometheus.yml"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD wget -qO- http://localhost:9090/-/healthy || exit 1
