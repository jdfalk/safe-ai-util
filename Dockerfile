# file: Dockerfile
# version: 1.0.0
# guid: 2b3c4d5e-6f7a-8901-bcde-f23456789012

# Multi-stage build for optimized Rust binary
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    appuser

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release && \
    strip target/release/copilot-agent-util

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Import from builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

# Copy the binary
COPY --from=builder /app/target/release/copilot-agent-util /usr/local/bin/copilot-agent-util

# Use an unprivileged user
USER appuser:appuser

# Set up working directory
WORKDIR /workspace

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD copilot-agent-util --version || exit 1

# Default command
ENTRYPOINT ["copilot-agent-util"]
CMD ["--help"]
