//! Request extractors for type-safe data extraction.

use crate::{Error, Req, Result};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Extract data from incoming requests.
#[async_trait]
pub trait FromRequest<S = ()>: Sized {
    /// Extract data from request.
    async fn from_request(req: &mut Req, state: &Arc<S>) -> Result<Self>;
}

/// Extract application state.
pub struct State<S>(pub S);

#[async_trait]
impl<S> FromRequest<S> for State<S>
where
    S: Clone + Send + Sync + 'static,
{
    async fn from_request(_req: &mut Req, state: &Arc<S>) -> Result<Self> {
        Ok(State((**state).clone()))
    }
}

/// Extract URL query parameters.
pub struct Query<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        let query = req
            .uri()
            .query()
            .ok_or_else(|| Error::bad_request("Missing query string"))?;

        let value = serde_urlencoded::from_str::<T>(query)
            .map_err(|e| Error::bad_request(format!("Invalid query parameters: {}", e)))?;

        Ok(Query(value))
    }
}

/// Extract form data from request body.
pub struct Form<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Form<T>
where
    T: DeserializeOwned,
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        let content_type = req
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.starts_with("application/x-www-form-urlencoded") {
            return Err(Error::bad_request(
                "Content-Type must be application/x-www-form-urlencoded",
            ));
        }

        let body = req.body().await?;
        let value = serde_urlencoded::from_bytes::<T>(body.as_ref())
            .map_err(|e| Error::unprocessable(format!("Invalid form data: {}", e)))?;

        Ok(Form(value))
    }
}

/// Extract JSON from request body.
pub struct Json<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        let content_type = req
            .headers()
            .get(hyper::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if !content_type.starts_with("application/json") {
            return Err(Error::bad_request("Content-Type must be application/json"));
        }

        let body = req.body().await?;
        let value = serde_json::from_slice(body)
            .map_err(|e| Error::bad_request(format!("Invalid JSON: {}", e)))?;

        Ok(Json(value))
    }
}

/// Extract path parameters.
pub struct Path<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Path<T>
where
    T: DeserializeOwned,
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        let params = req.path_params();

        let json_str = serde_json::to_string(params).map_err(|e| {
            Error::bad_request(format!("Failed to serialize path parameters: {}", e))
        })?;

        let value = serde_json::from_str::<T>(&json_str)
            .map_err(|e| Error::bad_request(format!("Invalid path parameters: {}. Path parameters are strings, use String type or implement custom deserializer", e)))?;

        Ok(Path(value))
    }
}

/// Extract all request headers.
pub struct Headers(pub hyper::HeaderMap);

#[async_trait]
impl<S> FromRequest<S> for Headers
where
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        Ok(Headers(req.headers().clone()))
    }
}

/// Extract raw request body as bytes.
pub struct BodyBytes(pub bytes::Bytes);

#[async_trait]
impl<S> FromRequest<S> for BodyBytes
where
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        let body = req.body().await?;
        Ok(BodyBytes(body.clone()))
    }
}
