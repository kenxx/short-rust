use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::{info, warn};

mod db;
mod hash;

use db::Cdb64Store;
use hash::djb_hash;

#[derive(Parser, Debug)]
#[command(name = "short-rust")]
#[command(about = "URL shortening service built with Rust")]
#[command(disable_help_flag = true)]
struct Args {
    /// CDB database file path
    #[arg(short = 'd', long = "db", env = "SHORT_RUST_DB")]
    db_path: Option<String>,

    /// Server host to bind to
    #[arg(short = 'h', long = "host", env = "SHORT_RUST_HOST", default_value = "0.0.0.0")]
    host: String,

    /// Server port to bind to
    #[arg(short = 'p', long = "port", env = "SHORT_RUST_PORT", default_value = "3774")]
    port: u16,

    /// Base URL for short links (e.g., https://short.com)
    #[arg(short = 'u', long = "url", env = "SHORT_RUST_URL")]
    base_url: Option<String>,

    /// Static files directory path
    #[arg(short = 's', long = "static", env = "SHORT_RUST_STATIC")]
    static_dir: Option<String>,

    /// Print help information
    #[arg(long = "help", action = clap::ArgAction::Help)]
    help: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
struct LongToShortParams {
    #[serde(rename = "longUrl")]
    long_url: String,
    #[serde(rename = "shortKey")]
    short_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct ShortUrlResponse {
    #[serde(rename = "Code")]
    code: i32,
    #[serde(rename = "ShortUrl", skip_serializing_if = "Option::is_none")]
    short_url: Option<String>,
    #[serde(rename = "Msg", skip_serializing_if = "Option::is_none")]
    msg: Option<String>,
}

// Use Cdb64Store for storage
type Store = Arc<Cdb64Store>;

// Application state
#[derive(Clone)]
struct AppState {
    store: Store,
    base_url: Option<String>,
}

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../public/index.html"))
}

async fn long_to_short_handler(
    State(state): State<AppState>,
    params: Json<LongToShortParams>,
) -> Result<Json<ShortUrlResponse>, Json<ShortUrlResponse>> {
    let long_url = params.long_url.trim();
    info!("Received request to shorten URL: {}", long_url);
    
    if long_url.is_empty() {
        warn!("Empty URL provided");
        return Err(Json(ShortUrlResponse {
            code: 400,
            short_url: None,
            msg: Some("Empty URL provided".to_string()),
        }));
    }

    // Use custom short_key if provided, otherwise generate from hash with collision detection
    let short_key = if let Some(ref key) = params.short_key {
        if !key.is_empty() {
            key.clone()
        } else {
            generate_short_key_with_collision_check(long_url, &state.store)
        }
    } else {
        generate_short_key_with_collision_check(long_url, &state.store)
    };

    // Check if key already exists
    // If short_key exists and points to a different URL, return error
    if state.store.exists(&short_key) {
        if let Some(existing_url) = state.store.get(&short_key) {
            if existing_url != long_url {
                warn!("Short key '{}' already exists with different URL", short_key);
                return Err(Json(ShortUrlResponse {
                    code: 409,
                    short_url: None,
                    msg: Some(format!("Short key '{}' already exists with different URL", short_key)),
                }));
            }
            info!("Short key '{}' already exists with same URL", short_key);
        }
    } else {
        state.store.set(short_key.clone(), long_url.to_string());
        info!("Created new short key '{}' for URL: {}", short_key, long_url);
    }

    // Build short URL with base URL if provided
    let short_url = if let Some(ref base_url) = state.base_url {
        format!("{}/{}", base_url.trim_end_matches('/'), short_key)
    } else {
        format!("/{}", short_key)
    };
    
    Ok(Json(ShortUrlResponse {
        code: 1,
        short_url: Some(short_url),
        msg: None,
    }))
}

async fn short_to_long_handler(
    State(state): State<AppState>,
    Path(short_key): Path<String>,
) -> Result<Redirect, StatusCode> {
    info!("Redirect request for short key: {}", short_key);
    
    if let Some(long_url) = state.store.get(&short_key) {
        info!("Redirecting '{}' to '{}'", short_key, long_url);
        Ok(Redirect::permanent(&long_url))
    } else {
        warn!("Short key '{}' not found", short_key);
        Err(StatusCode::NOT_FOUND)
    }
}

fn generate_short_key(url: &str) -> String {
    generate_short_key_with_salt(url, 0)
}

fn generate_short_key_with_salt(url: &str, salt: u64) -> String {
    let mut input = url.to_string();
    if salt > 0 {
        input.push_str(&salt.to_string());
    }
    
    let hash = djb_hash(&input);
    // Convert hash to base64 URL-safe string, take first 8 characters
    let mut key = base64::engine::general_purpose::URL_SAFE
        .encode(hash.to_le_bytes())
        .chars()
        .take(8)
        .collect::<String>();
    
    // Remove possible padding characters
    key.retain(|c| c != '=');
    
    if key.is_empty() {
        key = "default".to_string();
    }
    
    key
}

fn generate_short_key_with_collision_check(url: &str, store: &Store) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Generate initial key
    let mut key = generate_short_key(url);
    let max_attempts = 1000; // Prevent infinite loop
    
    // Check for collision: if key exists and points to different URL, regenerate
    for attempt in 0..max_attempts {
        if let Some(existing_url) = store.get(&key) {
            if existing_url == url {
                // Same URL, same key - this is fine
                return key;
            }
            // Collision detected: different URL, same key
            // Generate random salt and try again
            let mut hasher = DefaultHasher::new();
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            timestamp.hash(&mut hasher);
            attempt.hash(&mut hasher);
            let salt = hasher.finish();
            key = generate_short_key_with_salt(url, salt);
        } else {
            // Key doesn't exist, we can use it
            return key;
        }
    }
    
    // If we've tried too many times, return the last generated key
    // This should be extremely rare
    warn!("Collision detection reached max attempts for URL: {}", url);
    key
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Get current working directory
    let cwd = std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."));

    // Resolve database path (default: {cwd}/data/short-rust.cdb)
    let db_path = args.db_path
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(|| cwd.join("data").join("short-rust.cdb"));

    // Resolve static directory path (default: {cwd}/public)
    let static_dir = args.static_dir
        .unwrap_or_else(|| cwd.join("public").to_string_lossy().to_string());

    // Resolve base URL (default: http://{host}:{port})
    let base_url = args.base_url
        .unwrap_or_else(|| format!("http://{}:{}", args.host, args.port));

    // Log configuration
    info!("=== Configuration ===");
    info!("Database path: {}", db_path.display());
    info!("Static directory: {}", static_dir);
    info!("Server host: {}", args.host);
    info!("Server port: {}", args.port);
    info!("Base URL: {}", base_url);
    info!("===================");

    info!("Initializing storage at: {}", db_path.display());
    let store: Store = Arc::new(Cdb64Store::with_path(db_path));

    // Create application state
    let app_state = AppState {
        store,
        base_url: Some(base_url),
    };

    // Build routes
    info!("Building routes");
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/short", post(long_to_short_handler))
        .route("/{shortKey}", get(short_to_long_handler))
        .nest_service("/logo.png", ServeDir::new(&static_dir))
        .with_state(app_state);

    // Start the server
    let bind_addr = format!("{}:{}", args.host, args.port);
    info!("Starting server on {}", bind_addr);
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap();
    
    info!("Server running on http://{}", bind_addr);
    
    axum::serve(listener, app).await.unwrap();
}
