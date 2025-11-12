//! Type-safe extractors for Rust Api
//!
//! This crate provides extractors for common request data types:
//! - `Query<T>` - URL query parameters
//! - `Form<T>` - Form data
//! - `Json<T>` - JSON body
//! - `Path<T>` - Path parameters
//!
//! # Example
//!
//! ```ignore
//! use rust_api_extractors::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct SearchParams {
//!     q: String,
//! }
//!
//! async fn search(Query(params): Query<SearchParams>) -> rust_api::Res {
///     Res::json(&serde_json::json!({"results": []}))
//! }
//! ```

use async_trait::async_trait;
use rust_api::{Error, Result};
use serde::de::DeserializeOwned;
use std::sync::Arc;

// Re-export FromRequest trait and Req from core
pub use rust_api::{FromRequest, Req};

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
/// async fn search(Query(params): Query<SearchParams>) -> rust_api::Res {
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
/// async fn login(Form(form): Form<LoginForm>) -> rust_api::Res {
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
/// async fn create(Json(user): Json<CreateUser>) -> rust_api::Res {
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
/// async fn get_user(Path(params): Path<UserPath>) -> rust_api::Res {
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

        let value = serde_json::from_value(serde_json::to_value(params).map_err(|e| Error::Json(e.to_string()))?)
            .map_err(|e| Error::bad_request(format!("Invalid path parameters: {}", e)))?;

        Ok(Path(value))
    }
}

/// Response helper with JSON support
///
/// Re-export of `rust_api::Res` with additional JSON methods.
pub struct Res;

impl Res {
    /// Create a plain text response
    pub fn text(body: impl Into<String>) -> rust_api::Res {
        rust_api::Res::text(body)
    }

    /// Create HTML response
    pub fn html(body: impl Into<String>) -> rust_api::Res {
        rust_api::Res::html(body)
    }

    /// Create a JSON response
    ///
    /// # Example
    ///
    /// ```ignore
    /// use rust_api_extractors::Res;
    ///
    /// async fn handler() -> rust_api::Res {
    ///     Res::json(&serde_json::json!({"status": "ok"}))
    /// }
    /// ```
    pub fn json<T: serde::Serialize>(value: &T) -> rust_api::Res {
        use bytes::Bytes;
        use http_body_util::Full;
        use hyper::{Response, StatusCode, header};

        match serde_json::to_vec(value) {
            Ok(bytes) => {
                let mut res = Response::new(Full::new(Bytes::from(bytes)));
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                );
                rust_api::Res::from_hyper(res)
            }
            Err(e) => {
                let error_msg = format!(r#"{{"error": "JSON serialization failed: {}"}}"#, e);
                let mut res = Response::new(Full::new(Bytes::from(error_msg)));
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                );
                rust_api::Res::from_hyper(res)
            }
        }
    }

    /// Create response with status code
    pub fn status(code: u16) -> rust_api::Res {
        rust_api::Res::status(code)
    }

    /// Create a response builder
    pub fn builder() -> rust_api::ResBuilder {
        rust_api::Res::builder()
    }
}

/// Prelude for convenient imports
pub mod prelude {
    pub use super::{Form, Json, Path, Query, Res};
}
