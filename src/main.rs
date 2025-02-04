use std::{sync::Arc, time::Duration};

use axum::{middleware as axum_middleware, routing::get, Router};
use middleware::cache::{cache_middleware, CacheConfig, MemoryCache};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

mod error;
mod handlers;
mod img;
mod metadata;
mod models;

mod middleware;

#[derive(Serialize, Deserialize)]
struct ExtractQuery {
    url: String,
}

#[tokio::main]
async fn main() {
    let app = create_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

pub fn create_router() -> Router {
    let cache = Arc::new(MemoryCache::new());
    let cache_config = Arc::new(CacheConfig {
        ttl: Duration::from_secs(3600), // 1 hour
    });
    Router::new()
        // cardyb compat route
        .route("/v1/image", get(handlers::proxy_image))
        .route("/v1/extract", get(handlers::extract_metadata_cardyb))
        .route("/api/v1/extract", get(handlers::extract_metadata))
        .route("/api/v1/image", get(handlers::proxy_image))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(axum_middleware::from_fn(move |req, next| {
                    let redis_cache = Arc::clone(&cache);
                    let config = Arc::clone(&cache_config);
                    async move { cache_middleware(redis_cache, config, req, next).await }
                })),
        )
}
