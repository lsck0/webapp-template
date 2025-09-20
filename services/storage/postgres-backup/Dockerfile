FROM ubuntu:20.04

# get dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    lz4

# get wal-g
RUN curl -L https://github.com/wal-g/wal-g/releases/latest/download/wal-g-pg-ubuntu-20.04-amd64.tar.gz -o /tmp/wal-g.tar.gz \
    && tar -xzf /tmp/wal-g.tar.gz -C /tmp \
    && mv /tmp/wal-g-pg-ubuntu-20.04-amd64 /usr/local/bin/wal-g \
    && rm -rf /tmp/wal-g*

COPY start.sh /start.sh

ENTRYPOINT ["/start.sh"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD echo 0
