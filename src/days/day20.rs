use axum::{
    body::Bytes,
    http::{header::CONTENT_TYPE, Request, StatusCode},
    middleware::{from_fn, Next},
    response::IntoResponse,
    routing::post,
    Router
};
use tar::Archive;

async fn tar_only_middleware<T>(request: Request<T>, next: Next<T>) -> Result<impl IntoResponse, StatusCode> {
    let content_type = request.headers().get(CONTENT_TYPE);

    if Some("application/x-tar") == content_type.map(|h| h.to_str().unwrap_or("[none]")) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNSUPPORTED_MEDIA_TYPE)
    }
}

async fn file_count_in_tar(tar_data: Bytes) -> Result<String, (StatusCode, String)> {
    let mut tar_archive = Archive::new(tar_data.as_ref());
    tar_archive.entries()
        .map(|es| es.count().to_string())
        .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Unable to parse tar: {}", e)))
}

async fn file_size_in_tar(tar_data: Bytes) -> Result<String, (StatusCode, String)> {
    let mut tar_archive = Archive::new(tar_data.as_ref());
    tar_archive.entries()
        .map(
            |es| {
                es.filter_map(|e| e.ok())
                    .map(|e| e.size())
                    .sum::<u64>()
                    .to_string()
            }
        ).map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, format!("Unable to parse tar: {}", e)))
}


pub fn archive_router() -> Router {
    Router::new().route("/archive_files", post(file_count_in_tar))
            .route("/archive_files_size", post(file_size_in_tar))
            .layer(from_fn(tar_only_middleware))
}