//! Type-safe extractors for request data
//!
//! This module provides extractors for common request data types:
//! - `Query<T>` - URL query parameters
//! - `Form<T>` - Form data
//! - `Json<T>` - JSON body
//! - `Path<T>` - Path parameters
//! - `State<S>` - Application state

use crate::{Error, Req, Result};
use async_trait::async_trait;
use serde::de::DeserializeOwned;
use std::sync::Arc;

/// Extract data from request
#[async_trait]
pub trait FromRequest<S = ()>: Sized {
    /// Extract from request
    async fn from_request(req: &mut Req, state: &Arc<S>) -> Result<Self>;
}

/// Extract application state
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

/// Extract query parameters from URL
///
/// Deserializes URL query string into a type using serde_urlencoded.
///
/// # Example
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct SearchParams {
///     q: String,
///     page: Option<u32>,
/// }
///
/// async fn search(Query(params): Query<SearchParams>) -> Res {
///     // params.q and params.page are available
/// }
/// ```
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

/// Extract form data from request body
///
/// Content-Type must be `application/x-www-form-urlencoded`.
///
/// # Example
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct LoginForm {
///     username: String,
///     password: String,
/// }
///
/// async fn login(Form(form): Form<LoginForm>) -> Res {
///     // form.username and form.password available
/// }
/// ```
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

        let body = req.body();
        let value = serde_urlencoded::from_bytes::<T>(body.as_ref())
            .map_err(|e| Error::unprocessable(format!("Invalid form data: {}", e)))?;

        Ok(Form(value))
    }
}

/// Extract JSON from request body
///
/// Content-Type must be `application/json`.
///
/// # Example
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct CreateUser {
///     name: String,
///     email: String,
/// }
///
/// async fn create(Json(user): Json<CreateUser>) -> Res {
///     // user.name and user.email available
/// }
/// ```
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

        let body = req.body();
        let value = serde_json::from_slice(body)
            .map_err(|e| Error::bad_request(format!("Invalid JSON: {}", e)))?;

        Ok(Json(value))
    }
}

/// Extract path parameters
///
/// # Example
///
/// ```ignore
/// #[derive(Deserialize)]
/// struct UserPath {
///     id: u32,
/// }
///
/// async fn get_user(Path(params): Path<UserPath>) -> Res {
///     // params.id available
/// }
/// ```
pub struct Path<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for Path<T>
where
    T: DeserializeOwned,
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        let params = req.path_params();

        // Serialize to JSON string then deserialize - serde_json can't auto-convert string to int
        // So the user must use String fields or implement custom deserializer
        let json_str = serde_json::to_string(params).map_err(|e| {
            Error::bad_request(format!("Failed to serialize path parameters: {}", e))
        })?;

        let value = serde_json::from_str::<T>(&json_str)
            .map_err(|e| Error::bad_request(format!("Invalid path parameters: {}. Note: path parameters are strings, use String type or implement custom deserializer for type conversion", e)))?;

        Ok(Path(value))
    }
}

/// Extract request headers
///
/// Provides access to all HTTP headers in the request.
///
/// # Example
///
/// ```ignore
/// async fn handler(Headers(headers): Headers) -> Res {
///     if let Some(auth) = headers.get("authorization") {
///         // Process authorization header
///     }
///     Res::text("OK")
/// }
/// ```
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

/// Extract raw body bytes
///
/// Provides direct access to the request body as bytes without any parsing.
///
/// # Example
///
/// ```ignore
/// async fn upload(BodyBytes(data): BodyBytes) -> Res {
///     // data contains raw bytes
///     Res::text(format!("Received {} bytes", data.len()))
/// }
/// ```
pub struct BodyBytes(pub bytes::Bytes);

#[async_trait]
impl<S> FromRequest<S> for BodyBytes
where
    S: Send + Sync + 'static,
{
    async fn from_request(req: &mut Req, _state: &Arc<S>) -> Result<Self> {
        Ok(BodyBytes(req.body().clone()))
    }
}
