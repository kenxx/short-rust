use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, Redirect},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::services::ServeDir;
use tracing::{info, warn};

mod db;
mod hash;

use db::Cdb64Store;
use hash::djb_hash;

#[derive(Debug, Deserialize, Serialize)]
struct LongToShortParams {
    #[serde(rename = "longUrl")]
    long_url: String,
    #[serde(rename = "shortKey")]
    short_key: Option<String>,
}

#[derive(Debug, Serialize)]
struct ShortUrlResponse {
    short_url: String,
    short_key: String,
}

// Use Cdb64Store for storage
type Store = Arc<Cdb64Store>;

async fn index_handler() -> Html<&'static str> {
    Html(include_str!("../public/index.html"))
}

async fn long_to_short_handler(
    store: axum::extract::State<Store>,
    params: Json<LongToShortParams>,
) -> Result<Json<ShortUrlResponse>, StatusCode> {
    let long_url = params.long_url.trim();
    info!("Received request to shorten URL: {}", long_url);
    
    if long_url.is_empty() {
        warn!("Empty URL provided");
        return Err(StatusCode::BAD_REQUEST);
    }

    // Use custom short_key if provided, otherwise generate from hash
    let short_key = if let Some(ref key) = params.short_key {
        if !key.is_empty() {
            key.clone()
        } else {
            generate_short_key(long_url)
        }
    } else {
        generate_short_key(long_url)
    };

    // Check if key already exists
    // If short_key exists and points to a different URL, return error
    if store.exists(&short_key) {
        if let Some(existing_url) = store.get(&short_key) {
            if existing_url != long_url {
                warn!("Short key '{}' already exists with different URL", short_key);
                return Err(StatusCode::CONFLICT);
            }
            info!("Short key '{}' already exists with same URL", short_key);
        }
    } else {
        store.set(short_key.clone(), long_url.to_string());
        info!("Created new short key '{}' for URL: {}", short_key, long_url);
    }

    let short_url = format!("/{}", short_key);
    Ok(Json(ShortUrlResponse {
        short_url,
        short_key,
    }))
}

async fn short_to_long_handler(
    store: axum::extract::State<Store>,
    Path(short_key): Path<String>,
) -> Result<Redirect, StatusCode> {
    info!("Redirect request for short key: {}", short_key);
    
    if let Some(long_url) = store.get(&short_key) {
        info!("Redirecting '{}' to '{}'", short_key, long_url);
        Ok(Redirect::permanent(&long_url))
    } else {
        warn!("Short key '{}' not found", short_key);
        Err(StatusCode::NOT_FOUND)
    }
}

fn generate_short_key(url: &str) -> String {
    let hash = djb_hash(url);
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

#[tokio::main]
async fn main() {
    // Initialize tracing subscriber for logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Initializing storage");
    let store: Store = Arc::new(Cdb64Store::new());

    // Build routes
    info!("Building routes");
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/short", post(long_to_short_handler))
        .route("/{shortKey}", get(short_to_long_handler))
        .nest_service("/logo.png", ServeDir::new("public"))
        .with_state(store);

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    
    info!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await.unwrap();
}
