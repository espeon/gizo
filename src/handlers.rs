use crate::{
    error::AppError,
    img, metadata,
    models::{CardyBResponse, ExtractQuery, MetadataResponse},
};
use axum::{extract::Query, http::HeaderMap, Json};

pub async fn extract_metadata(
    Query(query): Query<ExtractQuery>,
) -> Result<Json<MetadataResponse>, AppError> {
    metadata::extract_metadata(&query.url).await.map(Json)
}

pub async fn extract_metadata_cardyb(
    Query(query): Query<ExtractQuery>,
) -> Result<Json<CardyBResponse>, AppError> {
    let res = metadata::extract_metadata(&query.url).await;
    match res {
        Ok(res) => Ok(Json(match res.metadata {
            crate::models::MetadataResponseType::OpenGraph(res) => res.into(),
        })),
        Err(e) => Err(e),
    }
}

#[axum::debug_handler]
pub async fn proxy_image(
    Query(query): Query<ExtractQuery>,
) -> Result<(HeaderMap, Vec<u8>), AppError> {
    img::proxy_image(&query.url).await
}
