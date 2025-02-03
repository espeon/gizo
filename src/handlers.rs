use crate::{
    error::AppError,
    img, metadata,
    models::{ExtractQuery, MetadataResponse},
};
use axum::{extract::Query, http::HeaderMap, Json};

pub async fn extract_metadata(
    Query(query): Query<ExtractQuery>,
) -> Result<Json<MetadataResponse>, AppError> {
    metadata::extract_metadata(&query.url).await.map(Json)
}

#[axum::debug_handler]
pub async fn proxy_image(
    Query(query): Query<ExtractQuery>,
) -> Result<(HeaderMap, Vec<u8>), AppError> {
    img::proxy_image(&query.url).await
}
