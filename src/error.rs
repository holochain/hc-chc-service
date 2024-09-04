use axum::{http::StatusCode, response::IntoResponse};
use holochain::prelude::PrevActionError;

#[derive(Debug, thiserror::Error)]
pub enum ChcServiceError {
    #[error("Hash was nout found in the CHC")]
    HashNotFound(String),
    #[error("Bad request error: {}", 0)]
    BadRequest(String),
    #[error("Invalid record input: {}", 0)]
    InvalidRecordInput(u32),
    #[error(transparent)]
    InvalidChain(#[from] PrevActionError),
    #[error(transparent)]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for ChcServiceError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::HashNotFound(message) => {
                tracing::error!("Hash was nout foundin the CHC");
                e498(&message)
            }
            Self::BadRequest(body_text) => {
                tracing::error!("Bad request error: {}", body_text);
                (
                    StatusCode::BAD_REQUEST,
                    format!("Bad request error: {}", body_text),
                )
                    .into_response()
            }
            Self::InvalidRecordInput(seq) => {
                tracing::error!("Invalid record input: {}", seq);
                e498(seq)
            }
            Self::InvalidChain(e) => {
                // local chain is out of sync with CHC
                // call get_record_data instead of adding record
                tracing::error!("Invalid chain error: {}", e);
                let response = (
                    StatusCode::from_u16(409).unwrap(),
                    rmp_serde::to_vec("Local chain is out of sync with the CHC")
                        .expect("Failed to serialize to MessagePack"),
                )
                    .into_response();
                response
            }
            Self::InternalError(e) => {
                tracing::error!("Internal server error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
            }
        }
    }
}

fn e498<T: serde::Serialize>(value: T) -> axum::response::Response {
    (
        StatusCode::from_u16(498).unwrap(),
        rmp_serde::to_vec(&value).expect("Failed to serialize MessagePack"),
    )
        .into_response()
}
