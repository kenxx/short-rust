# Short Rust - Agent Documentation

## Project Overview

This is a URL shortening service built with Rust and Axum framework. It converts long URLs to short URLs using djb hash (times33) algorithm and stores them using Cdb64Store (currently implemented as in-memory storage).

## Technology Stack

- **Web Framework**: Axum 0.8
- **Async Runtime**: Tokio
- **Serialization**: Serde
- **Hash Algorithm**: djb hash (times33)
- **Storage**: Cdb64Store (in-memory HashMap implementation, ready for cdb64 integration)
- **Base64 Encoding**: base64 0.22
- **Logging**: tracing and tracing-subscriber

## Project Structure

```
short-rust/
├── src/
│   ├── main.rs      # Main application entry point
│   ├── hash.rs      # djb hash (times33) implementation
│   └── db.rs        # Cdb64Store database operations
├── public/
│   └── index.html   # Web UI for URL shortening
├── Cargo.toml       # Project dependencies
└── AGENTS.md        # This file
```

## Routes

1. **GET /** - Serves the HTML interface for URL shortening
2. **POST /short** - Converts long URL to short URL
3. **GET /{shortKey}** - Redirects to the original long URL (Axum 0.8 uses `{shortKey}` syntax, not `:shortKey`)

## API Specification

### POST /short

Converts a long URL to a short URL.

**Request Body**:
```json
{
  "longUrl": "https://example.com/very/long/url",
  "shortKey": "custom-key"  // Optional
}
```

**Response** (200 OK):
```json
{
  "short_url": "/abc123",
  "short_key": "abc123"
}
```

**Error Responses**:
- `400 Bad Request` - Invalid or empty longUrl
- `409 Conflict` - Short key already exists and points to a different URL

### GET /{shortKey}

Redirects to the original long URL.

**Response**:
- `301 Permanent Redirect` - Redirects to the original URL
- `404 Not Found` - Short key does not exist

## Key Components

### Cdb64Store (`src/db.rs`)

In-memory storage implementation with the following methods:
- `new()` - Creates a new store instance
- `get(key: &str)` - Retrieves a value by key
- `set(key: String, value: String)` - Stores a key-value pair
- `exists(key: &str)` - Checks if a key exists

### djb_hash (`src/hash.rs`)

Implements the djb hash (times33) algorithm:
- Takes a string input
- Returns a u64 hash value
- Used for generating short keys when custom key is not provided

### generate_short_key (`src/main.rs`)

Generates a short key from a URL:
1. Computes djb hash of the URL
2. Converts hash to base64 URL-safe encoding
3. Takes first 8 characters
4. Removes padding characters

## Important Notes

1. **Storage**: Currently uses in-memory HashMap. The Cdb64Store struct is designed to be easily replaced with actual cdb64 implementation.

2. **Hash Algorithm**: Uses djb hash (times33) as specified. The hash value is converted to base64 for URL-safe short keys.

3. **Short Key Generation**: 
   - If custom shortKey is provided and not empty, it will be used
   - Otherwise, a short key is auto-generated using djb hash + base64 encoding
   - Generated keys are 8 characters long (or less if padding is removed)

4. **Conflict Handling**: If a short key already exists and points to a different URL, the API returns 409 Conflict. If it points to the same URL, it's considered valid.

5. **Server Configuration**: 
   - Binds to `0.0.0.0:3000`
   - Serves static files from `public/` directory
   - Logo is served at `/logo.png`
   - Uses tracing for logging (INFO level by default)

6. **Language Requirement**: 
   - **ABSOLUTELY NO CHINESE CHARACTERS** in any part of the project
   - All text, comments, strings, documentation must be in English only
   - This includes: source code, HTML, JavaScript, comments, error messages, logs, documentation

## Development Guidelines

- **CRITICAL: No Chinese characters anywhere in the project**
  - All code, comments, documentation, and strings must be in English only
  - No Chinese characters in source code files (`.rs`, `.html`, `.js`, etc.)
  - No Chinese characters in comments, variable names, function names, or string literals
  - No Chinese characters in error messages, log messages, or user-facing text
  - No Chinese characters in documentation files (`.md`, `.txt`, etc.)
- Use Rust 2021 edition
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Keep handlers async and use proper error handling
- Store is shared across requests using Arc for thread safety

## Future Enhancements

- Integrate actual cdb64 database for persistent storage
- Add URL expiration functionality
- Add access statistics/analytics
- Add batch URL shortening
- Add URL validation

