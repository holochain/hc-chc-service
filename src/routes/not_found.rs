use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
};

#[tracing::instrument]
pub async fn not_found(request: Request<Body>) -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 - Not Found")
}
