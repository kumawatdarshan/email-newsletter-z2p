# Builder stage
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt update && apt install mold clang -y

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build only deps
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build the Project
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin z2p

# Runtime Stage
FROM debian:trixie-slim AS runtime
WORKDIR /app

RUN apt update -y \
    && apt install -y --no-install-recommends openssl ca-certificates \
    # Cleanup
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/z2p z2p

COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./z2p"]
