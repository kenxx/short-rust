# Build stage
FROM rust:1.91-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src
COPY public ./public

# Build the real application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.21

RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /app/target/release/short-rust /usr/local/bin/short-rust

# Copy public directory
COPY --from=builder /app/public /app/public

# Create data directory
RUN mkdir -p /app/data

WORKDIR /app

EXPOSE 3774

ENTRYPOINT ["short-rust"]
CMD ["-h", "0.0.0.0", "-p", "3774"]
