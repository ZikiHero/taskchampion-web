# Plan and build dependencies
FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1.93-slim AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release --locked

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user and data directory
RUN useradd -m -u 1001 taskchampion && \
    mkdir -p /app/data && \
    chown taskchampion:taskchampion /app/data

# Copy binary
COPY --from=builder /app/target/release/taskchampion-middleware /usr/local/bin/

# Set default environment variables
ENV TC_SERVER_DIR=/app/data/server
ENV TC_DB_PATH=/app/data/local-cache.db

# Switch to non-root user
USER taskchampion

WORKDIR /app/data

EXPOSE 3001

CMD ["taskchampion-middleware"]

