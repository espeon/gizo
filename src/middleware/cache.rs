use axum::{
    body::{self, Body, Bytes},
    extract::Request,
    http::{HeaderMap, Response, StatusCode},
    middleware::Next,
};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheConfig {
    pub ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    data: Bytes,
    headers: HeaderMap,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct MemoryCache {
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        let store = Arc::new(RwLock::new(HashMap::<String, CacheEntry>::new()));

        // Spawn a background task to clean expired entries
        let store_clone = Arc::clone(&store);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                let mut cache = store_clone.write().await;
                cache.retain(|_, entry| entry.expires_at > Instant::now());
            }
        });

        Self { store }
    }

    pub async fn get(&self, key: &str) -> Option<(HeaderMap, Bytes)> {
        let cache = self.store.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                return Some((entry.headers.clone(), entry.data.clone()));
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: Bytes, headers: HeaderMap, ttl: Duration) {
        let mut cache = self.store.write().await;
        cache.insert(
            key,
            CacheEntry {
                data: value,
                headers,
                expires_at: Instant::now() + ttl,
            },
        );
    }
}

pub async fn cache_middleware(
    cache: Arc<MemoryCache>,
    config: Arc<CacheConfig>,
    req: Request,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    // Don't cache non-GET requests
    if req.method() != "GET" {
        return Ok(next.run(req).await);
    }

    let cache_key = generate_cache_key(&req);

    // Try to get from cache
    if let Some((headers, cached_response)) = cache.get(&cache_key).await {
        let mut headers = headers;
        headers.insert("x-gizo-cache", "HIT".parse().unwrap());
        let mut res = Response::builder();
        {
            let headers_res = res.headers_mut().unwrap();
            for (key, value) in headers.drain() {
                if let Some(key) = key {
                    headers_res.insert(key, value);
                }
            }
        };

        let res = res
            .status(StatusCode::OK)
            .body(Body::from(cached_response))
            .unwrap();
        return Ok(res);
    }

    // If not in cache, proceed with request
    let mut response = next.run(req).await;

    if response.status().is_success() {
        let (pts, body) = response.into_parts();
        // Cache all responses under 32mb
        let body_bytes = body::to_bytes(body, 32 * 1024 * 1024).await.unwrap();

        cache
            .set(
                cache_key,
                body_bytes.clone(),
                pts.headers.clone(),
                config.ttl,
            )
            .await;
        response = Response::from_parts(pts, Body::from(body_bytes));
    }

    {
        let headers = response.headers_mut();
        headers.insert("x-gizo-cache", "MISS".parse().unwrap());
    }

    Ok(response)
}

fn generate_cache_key(req: &Request) -> String {
    format!("{}:{}", req.method(), req.uri().path_and_query().unwrap())
}
