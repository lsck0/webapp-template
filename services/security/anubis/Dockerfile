FROM ghcr.io/techarohq/anubis:v1.21.0

COPY ./robots.txt /data/cfg/robots.txt
COPY ./policy.yaml /data/cfg/policy.yaml

ENV ED25519_PRIVATE_KEY_HEX="5da0e0cebd4399f494d56a7afca2eaa8740e1e92f00000cfc144486267a1b5f4"

ENV BIND=:8080
ENV METRICS_BIND=:9090
ENV TARGET=http://nginx:8080

ENV SERVE_ROBOTS_TXT=true
ENV POLICY_FNAME=/data/cfg/policy.yaml

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD ["anubis", "--healthcheck"]
