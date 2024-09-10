use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use rmp_serde::encode;
use serde::{de::DeserializeOwned, Serialize};

use crate::ChcServiceError;

#[derive(Debug)]
pub struct MsgPack<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for MsgPack<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ChcServiceError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // No explicit msgpack header check
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| ChcServiceError::BadRequest(e.to_string()))?;
        let value = rmp_serde::from_slice(&bytes)
            .map_err(|e| ChcServiceError::BadRequest(e.to_string()))?;
        Ok(MsgPack(value))
    }
}

impl<T> IntoResponse for MsgPack<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match encode::to_vec_named(&self.0) {
            Ok(bytes) => {
                let mut response = bytes.into_response();
                response.headers_mut().insert(
                    header::CONTENT_TYPE,
                    HeaderValue::from_static("application/msgpack"),
                );
                response
            }
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
        }
    }
}
