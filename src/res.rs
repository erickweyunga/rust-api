//! HTTP response builder.

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode, header};
use serde::Serialize;

/// HTTP response wrapper.
pub struct Res {
    inner: Response<Full<Bytes>>,
}

impl Res {
    /// Create response with status 200 OK.
    pub fn new() -> Self {
        Self {
            inner: Response::new(Full::new(Bytes::new())),
        }
    }

    /// Create from hyper response.
    pub fn from_hyper(inner: Response<Full<Bytes>>) -> Self {
        Self { inner }
    }

    /// Convert to hyper response.
    pub fn into_hyper(self) -> Response<Full<Bytes>> {
        self.inner
    }

    /// Create plain text response.
    pub fn text(body: impl Into<String>) -> Self {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        Self { inner: res }
    }

    /// Create HTML response.
    pub fn html(body: impl Into<String>) -> Self {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        res.headers_mut().insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("text/html; charset=utf-8"),
        );
        Self { inner: res }
    }

    /// Create JSON response.
    pub fn json<T: Serialize>(value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(bytes) => {
                let mut res = Response::new(Full::new(Bytes::from(bytes)));
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                );
                Self { inner: res }
            }
            Err(e) => {
                let error_msg = format!(r#"{{"error": "JSON serialization failed: {}"}}"#, e);
                let mut res = Response::new(Full::new(Bytes::from(error_msg)));
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                res.headers_mut().insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json"),
                );
                Self { inner: res }
            }
        }
    }

    /// Create response with status code.
    pub fn status(code: u16) -> Self {
        let mut res = Response::new(Full::new(Bytes::new()));
        *res.status_mut() = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Self { inner: res }
    }

    /// Create response builder.
    pub fn builder() -> ResBuilder {
        ResBuilder::new()
    }

    /// Get status code.
    pub fn status_code(&self) -> StatusCode {
        self.inner.status()
    }

    /// Add header to response.
    pub fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(name), Ok(value)) = (
            header::HeaderName::from_bytes(name.as_ref().as_bytes()),
            header::HeaderValue::from_str(value.as_ref()),
        ) {
            self.inner.headers_mut().insert(name, value);
        }
        self
    }

    /// Get mutable access to headers.
    pub fn headers_mut(&mut self) -> &mut header::HeaderMap {
        self.inner.headers_mut()
    }

    /// Get read-only access to headers.
    pub fn headers(&self) -> &header::HeaderMap {
        self.inner.headers()
    }
}

impl Default for Res {
    fn default() -> Self {
        Self::new()
    }
}

/// Response builder with fluent API.
pub struct ResBuilder {
    status: StatusCode,
    headers: Vec<(header::HeaderName, header::HeaderValue)>,
}

impl ResBuilder {
    /// Create response builder.
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: Vec::new(),
        }
    }

    /// Set HTTP status code.
    pub fn status(mut self, code: u16) -> Self {
        self.status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        self
    }

    /// Add header to response.
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(name), Ok(value)) = (
            header::HeaderName::from_bytes(name.as_ref().as_bytes()),
            header::HeaderValue::from_str(value.as_ref()),
        ) {
            self.headers.push((name, value));
        }
        self
    }

    /// Build text response.
    pub fn text(self, body: impl Into<String>) -> Res {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        *res.status_mut() = self.status;

        let has_content_type = self
            .headers
            .iter()
            .any(|(name, _)| name == header::CONTENT_TYPE);
        if !has_content_type {
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; charset=utf-8"),
            );
        }

        for (name, value) in self.headers {
            res.headers_mut().insert(name, value);
        }

        Res { inner: res }
    }

    /// Build HTML response.
    pub fn html(self, body: impl Into<String>) -> Res {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        *res.status_mut() = self.status;

        let has_content_type = self
            .headers
            .iter()
            .any(|(name, _)| name == header::CONTENT_TYPE);
        if !has_content_type {
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/html; charset=utf-8"),
            );
        }

        for (name, value) in self.headers {
            res.headers_mut().insert(name, value);
        }

        Res { inner: res }
    }

    /// Build JSON response.
    pub fn json<T: serde::Serialize>(self, value: &T) -> Res {
        match serde_json::to_string(value) {
            Ok(body) => {
                let mut res = Response::new(Full::new(Bytes::from(body)));
                *res.status_mut() = self.status;

                let has_content_type = self
                    .headers
                    .iter()
                    .any(|(name, _)| name == header::CONTENT_TYPE);
                if !has_content_type {
                    res.headers_mut().insert(
                        header::CONTENT_TYPE,
                        header::HeaderValue::from_static("application/json"),
                    );
                }

                for (name, value) in self.headers {
                    res.headers_mut().insert(name, value);
                }

                Res { inner: res }
            }
            Err(_) => Res::builder().status(500).text("Failed to serialize JSON"),
        }
    }
}

impl Default for ResBuilder {
    fn default() -> Self {
        Self::new()
    }
}
