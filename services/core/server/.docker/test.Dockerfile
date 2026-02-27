FROM rustlang/rust:nightly AS base
RUN apt-get update && apt-get install --assume-yes clang mold just
RUN cargo install cargo-afl cargo-audit cargo-chef cargo-deny

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
RUN just check
RUN cargo test --workspace --all-features
