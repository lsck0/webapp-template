FROM grafana/grafana:12.4.0-21419697389

ENV GF_PLUGINS_PREINSTALL=https://storage.googleapis.com/integration-artifacts/grafana-lokiexplore-app/grafana-lokiexplore-app-latest.zip;grafana-lokiexplore-app;grafana-investigations-app;grafana-pyroscope-app

COPY ./provisioning /etc/grafana/provisioning

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD curl --fail --silent http://localhost:3000/api/health || exit 1
