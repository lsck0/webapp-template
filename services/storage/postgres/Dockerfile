FROM postgres:17.5

# get build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    postgresql-server-dev-17

# install plugins
RUN git clone https://github.com/pgvector/pgvector.git \
    && cd pgvector \
    && make \
    && make install

RUN apt-get install -y postgis postgresql-17-postgis-3

# cleanup
RUN apt-get remove -y build-essential git postgresql-server-dev-17 \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

COPY init.sql /docker-entrypoint-initdb.d/init.sql

EXPOSE 5432

CMD ["postgres", "-c", "max_connections=500", "-c", "shared_buffers=50MB", "-c", "kernel.shmmax=57671680"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD pg_isready || exit 1
