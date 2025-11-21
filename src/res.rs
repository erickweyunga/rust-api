//! HTTP response with optimized serialization.

use bytes::Bytes;
use http_body_util::Full;
use hyper::{Response, StatusCode, header};
use serde::Serialize;

#[cfg(feature = "websocket")]
use base64::{Engine as _, engine::general_purpose};
#[cfg(feature = "websocket")]
use sha1::{Digest, Sha1};

static CONTENT_TYPE_TEXT: header::HeaderValue =
    header::HeaderValue::from_static("text/plain; charset=utf-8");
static CONTENT_TYPE_HTML: header::HeaderValue =
    header::HeaderValue::from_static("text/html; charset=utf-8");
static CONTENT_TYPE_JSON: header::HeaderValue =
    header::HeaderValue::from_static("application/json");

/// HTTP response.
pub struct Res {
    inner: Response<Full<Bytes>>,
    #[cfg(feature = "websocket")]
    ws_callback: Option<crate::websocket::WebSocketHandler>,
}

impl Res {
    /// Create empty 200 response.
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Response::new(Full::new(Bytes::new())),
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }

    /// Wrap hyper response.
    #[inline]
    pub fn from_hyper(inner: Response<Full<Bytes>>) -> Self {
        Self {
            inner,
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }

    /// Unwrap to hyper response.
    #[inline]
    pub fn into_hyper(self) -> Response<Full<Bytes>> {
        self.inner
    }

    /// Get WebSocket callback if present.
    #[cfg(feature = "websocket")]
    #[inline]
    pub(crate) fn take_ws_callback(&mut self) -> Option<crate::websocket::WebSocketHandler> {
        self.ws_callback.take()
    }

    /// Text response.
    pub fn text(body: impl Into<String>) -> Self {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        res.headers_mut()
            .insert(header::CONTENT_TYPE, CONTENT_TYPE_TEXT.clone());
        Self {
            inner: res,
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }

    /// HTML response.
    pub fn html(body: impl Into<String>) -> Self {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        res.headers_mut()
            .insert(header::CONTENT_TYPE, CONTENT_TYPE_HTML.clone());
        Self {
            inner: res,
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }

    /// JSON response (serializes to Vec<u8> directly).
    pub fn json<T: Serialize>(value: &T) -> Self {
        match serde_json::to_vec(value) {
            Ok(bytes) => {
                let mut res = Response::new(Full::new(Bytes::from(bytes)));
                res.headers_mut()
                    .insert(header::CONTENT_TYPE, CONTENT_TYPE_JSON.clone());
                Self {
                    inner: res,
                    #[cfg(feature = "websocket")]
                    ws_callback: None,
                }
            }
            Err(e) => {
                let error_msg = format!(r#"{{"error": "JSON serialization failed: {}"}}"#, e);
                let mut res = Response::new(Full::new(Bytes::from(error_msg)));
                *res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                res.headers_mut()
                    .insert(header::CONTENT_TYPE, CONTENT_TYPE_JSON.clone());
                Self {
                    inner: res,
                    #[cfg(feature = "websocket")]
                    ws_callback: None,
                }
            }
        }
    }

    /// Status-only response.
    pub fn status(code: u16) -> Self {
        let mut res = Response::new(Full::new(Bytes::new()));
        *res.status_mut() = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Self {
            inner: res,
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }

    /// Create builder.
    pub fn builder() -> ResBuilder {
        ResBuilder::new()
    }

    /// Create WebSocket upgrade response with handler callback.
    ///
    /// Returns 101 Switching Protocols with proper Sec-WebSocket-Accept header.
    #[cfg(feature = "websocket")]
    pub fn websocket<F>(websocket_key: &str, handler: F) -> Self
    where
        F: Fn(
                crate::websocket::WebSocket,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
            + Send
            + Sync
            + 'static,
    {
        const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

        let mut hasher = Sha1::new();
        hasher.update(websocket_key.as_bytes());
        hasher.update(WEBSOCKET_GUID.as_bytes());
        let hash = hasher.finalize();
        let accept_key = general_purpose::STANDARD.encode(&hash);

        let mut res = Response::new(Full::new(Bytes::new()));
        *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;

        let headers = res.headers_mut();
        headers.insert(
            header::UPGRADE,
            header::HeaderValue::from_static("websocket"),
        );
        headers.insert(
            header::CONNECTION,
            header::HeaderValue::from_static("Upgrade"),
        );
        headers.insert(
            header::HeaderName::from_static("sec-websocket-accept"),
            header::HeaderValue::from_str(&accept_key).unwrap(),
        );

        Self {
            inner: res,
            ws_callback: Some(std::sync::Arc::new(move |ws| Box::pin(handler(ws)))),
        }
    }

    /// Get status code.
    pub fn status_code(&self) -> StatusCode {
        self.inner.status()
    }

    /// Add header.
    #[inline]
    pub fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(name), Ok(value)) = (
            header::HeaderName::from_bytes(name.as_ref().as_bytes()),
            header::HeaderValue::from_str(value.as_ref()),
        ) {
            self.inner.headers_mut().insert(name, value);
        }
        self
    }

    /// Get mutable headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut header::HeaderMap {
        self.inner.headers_mut()
    }

    /// Get headers.
    #[inline]
    pub fn headers(&self) -> &header::HeaderMap {
        self.inner.headers()
    }
}

impl Default for Res {
    fn default() -> Self {
        Self::new()
    }
}

/// Response builder with pre-allocated headers.
pub struct ResBuilder {
    status: StatusCode,
    headers: header::HeaderMap,
}

impl ResBuilder {
    /// Create builder.
    pub fn new() -> Self {
        Self {
            status: StatusCode::OK,
            headers: header::HeaderMap::with_capacity(4),
        }
    }

    /// Set status code.
    pub fn status(mut self, code: u16) -> Self {
        self.status = StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        self
    }

    /// Add header.
    pub fn header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let (Ok(name), Ok(value)) = (
            header::HeaderName::from_bytes(name.as_ref().as_bytes()),
            header::HeaderValue::from_str(value.as_ref()),
        ) {
            self.headers.insert(name, value);
        }
        self
    }

    /// Build text response.
    pub fn text(mut self, body: impl Into<String>) -> Res {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        *res.status_mut() = self.status;

        if !self.headers.contains_key(header::CONTENT_TYPE) {
            self.headers
                .insert(header::CONTENT_TYPE, CONTENT_TYPE_TEXT.clone());
        }

        *res.headers_mut() = self.headers;
        Res {
            inner: res,
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }

    /// Build HTML response.
    pub fn html(mut self, body: impl Into<String>) -> Res {
        let body_str = body.into();
        let mut res = Response::new(Full::new(Bytes::from(body_str)));
        *res.status_mut() = self.status;

        if !self.headers.contains_key(header::CONTENT_TYPE) {
            self.headers
                .insert(header::CONTENT_TYPE, CONTENT_TYPE_HTML.clone());
        }

        *res.headers_mut() = self.headers;
        Res {
            inner: res,
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }

    /// Build JSON response.
    pub fn json<T: Serialize>(mut self, value: &T) -> Res {
        match serde_json::to_vec(value) {
            Ok(bytes) => {
                let mut res = Response::new(Full::new(Bytes::from(bytes)));
                *res.status_mut() = self.status;

                if !self.headers.contains_key(header::CONTENT_TYPE) {
                    self.headers
                        .insert(header::CONTENT_TYPE, CONTENT_TYPE_JSON.clone());
                }

                *res.headers_mut() = self.headers;
                Res {
                    inner: res,
                    #[cfg(feature = "websocket")]
                    ws_callback: None,
                }
            }
            Err(_) => Res::builder().status(500).text("Failed to serialize JSON"),
        }
    }

    /// Build with custom body.
    pub fn body(self, bytes: impl Into<Bytes>) -> Res {
        let mut res = Response::new(Full::new(bytes.into()));
        *res.status_mut() = self.status;
        *res.headers_mut() = self.headers;
        Res {
            inner: res,
            #[cfg(feature = "websocket")]
            ws_callback: None,
        }
    }
}

impl Default for ResBuilder {
    fn default() -> Self {
        Self::new()
    }
}
