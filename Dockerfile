# Build Stage
FROM rust:1.80-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    clang \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the entire workspace
# (Optimization note: In a real scenario, you might copy only Cargo.toml first to cache dependencies)
COPY . .

# Build the server binary
RUN cargo build --release -p architect-server

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/architect-server /app/architect-server

# Expose the default port for SSE
EXPOSE 3000

# Set environment variables for SSE
ENV MCP_TRANSPORT=sse
ENV PORT=3000

# Default command
ENTRYPOINT ["/app/architect-server"]
