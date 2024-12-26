FROM rustlang/rust:nightly AS base
RUN apt-get update && apt-get install --assume-yes clang mold
RUN cargo install cargo-chef cargo-deny

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
RUN cargo deny check --allow unlicensed --allow license-not-encountered --allow duplicate
RUN cargo clippy --workspace --all-targets --all-features -- -D warnings
RUN cargo test --workspace --all-features
