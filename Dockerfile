# file: Dockerfile
# version: 1.0.1
# guid: 2b3c4d5e-6f7a-8901-bcde-f23456789012
# last-edited: 2026-10-04

# Multi-stage build for optimized Rust binary
FROM rust:1.98-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app user. Uses useradd, not adduser: the rust:1.97-slim base is Debian
# 13 (trixie), whose slim images no longer ship the adduser wrapper, so the old
# invocation failed with exit 127 (command not found). useradd is in the image.
RUN useradd \
    --no-create-home \
    --home-dir "/nonexistent" \
    --shell "/sbin/nologin" \
    --comment "" \
    --uid 10001 \
    appuser

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code. benches/ is required too: Cargo.toml declares a [[bench]]
# target at benches/executor_benchmark.rs, and cargo fails to even parse the
# manifest when that path is missing ("can't find `executor_benchmark` bench").
COPY src ./src
COPY benches ./benches

# Build the application
RUN cargo build --release && \
    strip target/release/copilot-agent-util

# Runtime stage
# Must match the builder's Debian release. The builder is rust:1.97-slim, which
# is Debian 13 (trixie, glibc 2.41); on bookworm (Debian 12, glibc 2.36) the
# binary built above fails at startup with "GLIBC_2.39 not found".
FROM debian:trixie-slim

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

# The working directory must be writable by the runtime user: the binary
# creates ./logs at startup, and a root-owned WORKDIR made every invocation,
# including the default --help, exit with "Permission denied".
RUN install -d -o appuser -g appuser /workspace

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
