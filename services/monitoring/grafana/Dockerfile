FROM grafana/grafana:12.4.0-21419697389

ENV GF_PLUGINS_PREINSTALL=grafana-investigations-app;grafana-lokiexplore-app;grafana-pyroscope-app;grafana-tempo-datasource

COPY ./provisioning /etc/grafana/provisioning

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD curl --fail --silent http://localhost:3000/api/health || exit 1
