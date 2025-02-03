use crate::error::AppError;
use image::ImageReader;
use reqwest::Client;

pub async fn proxy_image(url: &str) -> Result<(axum::http::HeaderMap, Vec<u8>), AppError> {
    let bytes = fetch_image_bytes(url).await?;
    let processed_image = process_image(&bytes)?;
    let headers = create_image_headers()?;

    Ok((headers, processed_image))
}

async fn fetch_image_bytes(url: &str) -> Result<Vec<u8>, AppError> {
    let client = Client::new();
    let bytes = client
        .get(url)
        .header(
            "user-agent",
            "Mozilla/5.0 (compatible; imagebot/1.0; +https://example.com)",
        )
        .send()
        .await?
        .bytes()
        .await?;

    Ok(bytes.to_vec())
}

fn process_image(bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    let img = ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| AppError::ImageProcessingError)?
        .decode()
        .map_err(|_| AppError::ImageProcessingError)?;

    let mut buffer = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buffer),
        image::ImageFormat::WebP,
    )
    .map_err(|_| AppError::ImageProcessingError)?;

    Ok(buffer)
}

fn create_image_headers() -> Result<axum::http::HeaderMap, AppError> {
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        "content-type",
        "image/webp".parse().expect("Invalid header value"),
    );
    headers.insert(
        "cache-control",
        "public, max-age=31536000"
            .parse()
            .expect("Invalid header value"),
    );

    Ok(headers)
}
