# Short Rust

URL shortening service built with Rust and Axum.

## Features

- URL shortening with djb hash (times33) algorithm
- Collision detection and automatic retry
- Persistent storage with cdb64
- Web interface for URL shortening
- Configurable via CLI arguments or environment variables

## Installation

```bash
cargo build --release
```

## Usage

### Command Line Arguments

```bash
cargo run -- \
  -d ./data/short-rust.cdb \
  -h 0.0.0.0 \
  -p 3774 \
  -u http://short.com \
  -s ./public
```

### Environment Variables

```bash
export SHORT_RUST_DB=./data/short-rust.cdb
export SHORT_RUST_HOST=0.0.0.0
export SHORT_RUST_PORT=3774
export SHORT_RUST_URL=http://short.com
export SHORT_RUST_STATIC=./public

cargo run
```

### Default Values

- **DB**: `{cwd}/data/short-rust.cdb`
- **Host**: `0.0.0.0`
- **Port**: `3774`
- **URL**: `http://{host}:{port}`
- **Static**: `{cwd}/public`

## API

### POST /short

Generate a short link.

**Request**:
```json
{
  "longUrl": "https://example.com/very/long/url",
  "shortKey": "custom-key"  // Optional
}
```

**Success Response**:
```json
{
  "Code": 1,
  "ShortUrl": "http://0.0.0.0:3774/abc123"
}
```

**Error Response**:
```json
{
  "Code": 400,
  "Msg": "Empty URL provided"
}
```

### GET /{shortKey}

Redirect to the original URL (301 Permanent Redirect).

## Technology Stack

- Axum 0.8
- Tokio
- cdb64
- djb hash (times33)

## License

MIT
