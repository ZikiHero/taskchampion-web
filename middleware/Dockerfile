# Plan and build dependencies
FROM lukemathwalker/cargo-chef:latest-rust-1.93-slim AS chef
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

# Create non-root user
RUN useradd -m -u 1001 taskchampion

# Copy binary
COPY --from=builder /app/target/release/taskchampion-service /usr/local/bin/

# Switch to non-root user
USER taskchampion

EXPOSE 3001

CMD ["taskchampion-service"]

