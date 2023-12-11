use axum::{
    extract::Multipart,
    http::StatusCode,
    routing::post,
    Router
};
use image::{
    load_from_memory_with_format,
    load_from_memory,
    ImageFormat
};
//use tokio::fs::read;
use tower_http::services::ServeFile;

async fn count_red_pixels(mut multipart: Multipart) -> Result<String, (StatusCode, String)> {
    let mut result = String::new();

    while let Some(field) = multipart.next_field()
        .await.map_err(|e| (StatusCode::BAD_REQUEST, format!("Unable to get multipart format: {}", e)))? {
            if field.name() == Some("image") {
                let content_type = field.content_type().map(|s| s.to_owned()); // Clone type before consuming field.

                let data = field.bytes().await
                    .or(Err((StatusCode::BAD_REQUEST, "Unable to decode bytes.".into())))?;

                let img = if let Some(mime_type) = content_type {
                    let fmt = ImageFormat::from_mime_type(&mime_type)
                        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("Unable to parse content type: {}", &mime_type)))?;

                    load_from_memory_with_format(&data, fmt)
                } else {
                    load_from_memory(&data)
                }.map_err(|e| (StatusCode::BAD_REQUEST, format!("Unable to decode image: {}", e)))?;

                let img_rgb = img.as_rgb8()
                    .ok_or((StatusCode::BAD_REQUEST, "Cannot convert to RGB".into()))?;

                let red_count = img_rgb.pixels()
                    .filter(|p| {
                        let [r, g, b] = p.0;
                        match g.checked_add(b) {
                            Some(s) => r > s,
                            None => false
                        }
                    }).count();
                // Add line in case of multi-output.
                if !result.is_empty() {
                    result.push_str("\r\n");
                }

                result.push_str(&red_count.to_string());
            }
    }

    Ok(result)
}

pub fn ornament_router() -> Router {
    Router::new().nest_service("/assets/decoration.png", ServeFile::new("assets/decoration.png"))
        .route("/red_pixels", post(count_red_pixels))
}