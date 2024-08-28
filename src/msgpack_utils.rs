use axum::{
    async_trait,
    body::{Body, Bytes},
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
        if !message_pack_content_type(&req) {
            return Err(ChcServiceError::InvalidRequestInput(
                "Invalid msgpack header".to_string(),
            ));
        }
        let bytes = Bytes::from_request(req, state)
            .await
            .map_err(|e| ChcServiceError::InvalidRequestInput(e.to_string()))?;
        let value = rmp_serde::from_slice(&bytes)
            .map_err(|e| ChcServiceError::InvalidRequestInput(e.to_string()))?;
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
            Err(err) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"))
                .body(Body::from(err.to_string()))
                .unwrap(),
        }
    }
}

/// Check is valid msgpack content-type header
fn message_pack_content_type<B>(req: &Request<B>) -> bool {
    let Some(content_type) = req.headers().get(header::CONTENT_TYPE) else {
        return false;
    };
    let Ok(content_type) = content_type.to_str() else {
        return false;
    };

    let parts: Vec<&str> = content_type
        .split(';')
        .next()
        .unwrap_or("")
        .split('/')
        .collect();

    if parts.len() != 2 {
        return false;
    }

    let (type_, subtype) = (parts[0].trim(), parts[1].trim());

    if type_ != "application" {
        return false;
    }

    let subtype_parts: Vec<&str> = subtype.split('+').collect();

    subtype_parts
        .iter()
        .any(|&s| s == "msgpack" || s == "x-msgpack")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{method, Request};

    fn create_request(content_type: Option<&str>) -> Request<()> {
        let builder = Request::builder()
            .method(method::Method::POST)
            .uri("https://localhost");
        let request = if let Some(ct) = content_type {
            builder.header(header::CONTENT_TYPE, ct).body(()).unwrap()
        } else {
            builder.body(()).unwrap()
        };
        request
    }

    #[test]
    fn test_message_pack_content_type() {
        // Test valid MessagePack content types
        assert!(message_pack_content_type(&create_request(Some(
            "application/msgpack"
        ))));
        assert!(message_pack_content_type(&create_request(Some(
            "application/x-msgpack"
        ))));
        assert!(message_pack_content_type(&create_request(Some(
            "application/vnd.company+msgpack"
        ))));
        assert!(message_pack_content_type(&create_request(Some(
            "application/msgpack; charset=utf-8"
        ))));

        // Test invalid content types
        assert!(!message_pack_content_type(&create_request(Some(
            "application/json"
        ))));
        assert!(!message_pack_content_type(&create_request(Some(
            "text/plain"
        ))));
        assert!(!message_pack_content_type(&create_request(Some(
            "application/octet-stream"
        ))));

        // Test edge cases
        assert!(!message_pack_content_type(&create_request(None)));
        assert!(!message_pack_content_type(&create_request(Some(""))));
        assert!(!message_pack_content_type(&create_request(Some(
            "application"
        ))));
        assert!(!message_pack_content_type(&create_request(Some(
            "application/"
        ))));
    }
}
