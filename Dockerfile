# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY public ./public

# Build for release
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/short-rust /usr/local/bin/short-rust

# Copy public directory
COPY --from=builder /app/public ./public

# Create data directory
RUN mkdir -p /app/data

# Expose port
EXPOSE 3774

# Run the application
ENTRYPOINT ["short-rust"]
CMD ["-h", "0.0.0.0", "-p", "3774"]

