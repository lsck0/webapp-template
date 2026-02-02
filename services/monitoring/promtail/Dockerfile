FROM grafana/promtail:3.6

RUN apt-get update && apt-get install -y curl

COPY ./promtail.yml /etc/promtail/promtail.yml

CMD ["-config.file=/etc/promtail/promtail.yml"]

EXPOSE 9080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD curl --fail --silent http://localhost:9080/ready || exit 1
