FROM rustlang/rust:nightly AS base
RUN apt-get update && apt-get install --assume-yes clang mold
RUN cargo install cargo-chef cargo-watch

FROM base AS planner
WORKDIR /app
COPY .. .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json

FROM base AS runtime
WORKDIR /app
COPY .. .
COPY --from=builder /app/target target
CMD ["cargo", "watch", "-x", "run"]

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD curl --fail --silent http://localhost/api/health || exit 1
