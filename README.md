# Short Rust - URL Shortener

A URL shortening service built with Rust and Axum framework.

## Features

- ✅ Convert long URLs to short URLs
- ✅ Custom short link keys
- ✅ Redirect short links to original URLs
- ✅ Beautiful web interface
- ✅ Generate short links using djb hash (times33) algorithm

## Technology Stack

- **Web Framework**: Axum 0.8
- **Async Runtime**: Tokio
- **Serialization**: Serde
- **Hash Algorithm**: djb hash (times33)
- **Storage**: In-memory storage (can be extended to cdb64)
- **Logging**: tracing and tracing-subscriber

## Quick Start

### Install Dependencies

Make sure you have Rust and Cargo installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Run the Project

```bash
cargo run
```

The server will start at `http://0.0.0.0:3000`.

### Usage

1. Visit `http://localhost:3000` to open the web interface
2. Enter a long URL
3. Optionally enter a custom short link key
4. Click "Generate Short Link" button
5. Copy and use the generated short link

## API Endpoints

### POST /short

Generate a short link.

**Request Body**:
```json
{
  "longUrl": "https://example.com/very/long/url",
  "shortKey": "custom-key"  // Optional
}
```

**Response**:
```json
{
  "short_url": "/abc123",
  "short_key": "abc123"
}
```

### GET /{shortKey}

Redirect to the original URL based on the short link key.

**Response**: 301 Permanent Redirect to the original URL

## Project Structure

```
short-rust/
├── src/
│   ├── main.rs      # Main application entry point
│   ├── hash.rs      # djb hash algorithm implementation
│   └── db.rs        # Database operations module (reserved)
├── public/
│   └── index.html   # Web interface
├── Cargo.toml       # Project configuration
└── README.md        # Project documentation
```

## Development Plan

- [ ] Integrate cdb64 database for persistent storage
- [ ] Add URL expiration functionality
- [ ] Add access statistics
- [ ] Add batch URL shortening functionality

## License

MIT
