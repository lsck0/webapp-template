FROM rustlang/rust:nightly AS base
RUN apt-get update && apt-get install --assume-yes clang mold
RUN cargo install cargo-chef

FROM base AS planner
WORKDIR /app
COPY .. .
RUN cargo chef prepare --recipe-path recipe.json

FROM base AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY .. .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install --assume-yes curl libpq5 libpq-dev
WORKDIR /app
COPY --from=builder /app/target/release/server ./server
ENTRYPOINT ["./server"]

EXPOSE 80

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s \
    CMD curl --fail --silent http://localhost/api/health || exit 1
