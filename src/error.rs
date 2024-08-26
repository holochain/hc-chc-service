use axum::response::IntoResponse;

#[derive(Debug, thiserror::Error)]
pub enum ChcServiceError {
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for ChcServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::InternalError(e) => {
                tracing::error!("Internal server error: {}", e);
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong",
                )
                    .into_response()
            }
        }
    }
}
