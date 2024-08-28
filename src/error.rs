use axum::{http::StatusCode, response::IntoResponse};
use holochain::prelude::ChcError;

#[derive(Debug, thiserror::Error)]
pub enum ChcServiceError {
    #[error("Hash was nout found in the CHC")]
    HashNotFound(String),
    #[error("Invalid request input: {}", 0)]
    InvalidRequestInput(String),
    #[error(transparent)]
    InvalidChain(#[from] ChcError),
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
            Self::InvalidRequestInput(body_text) => {
                tracing::error!("Invalid request input: {}", body_text);
                (
                    StatusCode::BAD_REQUEST,
                    format!("Bad Request: {}", body_text),
                )
                    .into_response()
            }
            Self::InvalidChain(e) => {
                if let ChcError::InvalidChain(seq, action_hash) = e {
                    // local chain is out of sync with CHC
                    // call get_record_data instead of adding record
                    tracing::error!(
                        "Invalid chain error (seq: {}, action_hash: {}",
                        seq,
                        action_hash
                    );
                    let response = (
                        StatusCode::from_u16(409).unwrap(),
                        rmp_serde::to_vec("Local chain is out of sync with the CHC")
                            .expect("Failed to serialize to MessagePack"),
                    )
                        .into_response();
                    return response;
                }
                e500(e.into())
            }
            Self::InternalError(e) => e500(e.into()),
        }
    }
}

fn e498(message: &str) -> axum::response::Response {
    (
        StatusCode::from_u16(498).unwrap(),
        rmp_serde::to_vec(message).expect("Failed to serialize MessagePack"),
    )
        .into_response()
}

fn e500(e: Box<dyn std::error::Error>) -> axum::response::Response {
    tracing::error!("Internal server error: {}", e);
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response()
}
