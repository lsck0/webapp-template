FROM postgres:17.7

# get dependencies
RUN apt-get update && apt-get install -y \
    bison \
    build-essential \
    ca-certificates \
    cmake \
    cron \
    curl \
    flex \
    git \
    libbrotli-dev \
    liblzo2-dev \
    libreadline-dev \
    libsodium-dev \
    lz4 \
    postgis \
    postgresql-17-postgis-3 \
    postgresql-server-dev-17 \
    wget \
    zlib1g-dev \
    supervisor

# get pgvector
ENV PG_CONFIG=/usr/lib/postgresql/17/bin/pg_config
RUN git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git \
    && cd pgvector \
    && make \
    && make install

# get age
ENV PG_CONFIG=/usr/lib/postgresql/17/bin/pg_config
RUN git clone --branch release/PG17/1.7.0 https://github.com/apache/age.git \
    && cd age \
    && make \
    && make install

# install go 1.25.5
RUN wget https://go.dev/dl/go1.25.5.linux-amd64.tar.gz -O /tmp/go.tar.gz \
    && tar -C /usr/local -xzf /tmp/go.tar.gz \
    && rm /tmp/go.tar.gz
ENV PATH="/usr/local/go/bin:${PATH}"

# get wal-g
RUN git clone https://github.com/wal-g/wal-g.git /usr/local/src/wal-g \
    && cd /usr/local/src/wal-g \
    && make clean \
    && make deps \
    && make pg_build \
    && mv main/pg/wal-g /usr/local/bin/wal-g \
    && rm -rf /usr/local/src/wal-g

# cleanup
RUN apt-get remove -y build-essential cmake git wget libbrotli-dev liblzo2-dev libsodium-dev postgresql-server-dev-17 \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

# copy configuration
COPY ./init.sh /docker-entrypoint-initdb.d/init.sh
COPY ./init.sql /docker-entrypoint-initdb.d/init.sql
COPY ./postgresql.conf /etc/postgresql/postgresql.conf
COPY ./supervisord.conf /etc/supervisor/conf.d/supervisord.conf
COPY ./wal-g-backup-loop.sh /usr/local/bin/wal-g-backup-loop.sh

EXPOSE 5432

CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD pg_isready || exit 1
