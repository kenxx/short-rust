# Build stage
FROM rust:1.83-alpine AS builder

WORKDIR /app

# Install dependencies
RUN apk update && apk add --no-cache \
    pkg-config \
    openssl-dev \
    musl-dev \
    build-base

# Copy manifests
COPY Cargo.toml ./
COPY Cargo.lock* ./

# Copy source code
COPY src ./src
COPY public ./public

# Build for release
RUN cargo build --release

# Runtime stage
FROM alpine:latest AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates

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

