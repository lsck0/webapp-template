FROM postgres:17.5

# get dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    ca-certificates \
    curl \
    git \
    lz4 \
    postgis \
    postgresql-17-postgis-3 \
    postgresql-server-dev-17

# get pgvector
RUN git clone https://github.com/pgvector/pgvector.git \
    && cd pgvector \
    && make \
    && make install

# get wal-g
RUN curl -L https://github.com/wal-g/wal-g/releases/latest/download/wal-g-pg-ubuntu-20.04-amd64.tar.gz -o /tmp/wal-g.tar.gz \
    && tar -xzf /tmp/wal-g.tar.gz -C /tmp \
    && mv /tmp/wal-g-pg-ubuntu-20.04-amd64 /usr/local/bin/wal-g \
    && rm -rf /tmp/wal-g*

# cleanup
RUN apt-get remove -y build-essential git postgresql-server-dev-17 \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

# copy configuration
COPY init.sql /docker-entrypoint-initdb.d/init.sql
COPY postgresql.conf /etc/postgresql/postgresql.conf

EXPOSE 5432

CMD ["postgres", "-c", "config_file=/etc/postgresql/postgresql.conf"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD pg_isready || exit 1
