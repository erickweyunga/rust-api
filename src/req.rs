//! HTTP request with lock-free body consumption.

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Method, Request, Uri, body::Incoming, header};
use std::collections::HashMap;
use tokio::sync::OnceCell;

use crate::extensions::Extensions;
use crate::{Error, Result};

/// HTTP request.
pub struct Req {
    method: Method,
    uri: Uri,
    headers: header::HeaderMap,
    body_cell: OnceCell<Bytes>,
    incoming: Option<Incoming>,
    path_params: HashMap<String, String>,
    extensions: Extensions,
    body_limit: Option<usize>,
}

impl Req {
    /// Create from hyper request.
    pub fn from_hyper(req: Request<Incoming>) -> Self {
        let (parts, body) = req.into_parts();

        Self {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
            body_cell: OnceCell::new(),
            incoming: Some(body),
            path_params: HashMap::new(),
            extensions: Extensions::new(),
            body_limit: None,
        }
    }

    /// Set body size limit.
    pub(crate) fn set_body_limit(&mut self, limit: Option<usize>) {
        self.body_limit = limit;
    }

    /// Get HTTP method.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get request URI.
    #[inline]
    pub fn uri(&self) -> &Uri {
        &self.uri
    }

    /// Get request path.
    #[inline]
    pub fn path(&self) -> &str {
        self.uri.path()
    }

    /// Get query string.
    #[inline]
    pub fn query(&self) -> Option<&str> {
        self.uri.query()
    }

    /// Get header value.
    #[inline]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// Get headers.
    #[inline]
    pub fn headers(&self) -> &header::HeaderMap {
        &self.headers
    }

    /// Get mutable headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut header::HeaderMap {
        &mut self.headers
    }

    /// Get path parameter.
    #[inline]
    pub fn param(&self, name: &str) -> Option<&str> {
        self.path_params.get(name).map(|s| s.as_str())
    }

    /// Get all path parameters.
    #[inline]
    pub fn params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    /// Get path parameters (for extractors).
    #[inline]
    pub fn path_params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    /// Consume body as bytes (cached on first call).
    pub async fn body(&mut self) -> Result<&Bytes> {
        self.body_cell
            .get_or_try_init(|| async {
                let incoming = self
                    .incoming
                    .take()
                    .ok_or_else(|| Error::internal("Request body already consumed"))?;

                // Check Content-Length header against limit
                if let Some(limit) = self.body_limit {
                    if let Some(content_length) = self.headers.get(header::CONTENT_LENGTH) {
                        if let Ok(length_str) = content_length.to_str() {
                            if let Ok(length) = length_str.parse::<usize>() {
                                if length > limit {
                                    return Err(Error::payload_too_large(&format!(
                                        "Request body size {} exceeds limit of {}",
                                        length, limit
                                    )));
                                }
                            }
                        }
                    }
                }

                let collected = incoming
                    .collect()
                    .await
                    .map_err(|e| Error::Custom(format!("Failed to read body: {}", e)))?;

                let body_bytes = collected.to_bytes();

                // Check actual body size against limit
                if let Some(limit) = self.body_limit {
                    if body_bytes.len() > limit {
                        return Err(Error::payload_too_large(&format!(
                            "Request body size {} exceeds limit of {}",
                            body_bytes.len(),
                            limit
                        )));
                    }
                }

                Ok(body_bytes)
            })
            .await
    }

    /// Get Content-Type header.
    #[inline]
    pub fn content_type(&self) -> Option<&str> {
        self.header(header::CONTENT_TYPE.as_str())
    }

    /// Check if Content-Type is JSON.
    #[inline]
    pub fn is_json(&self) -> bool {
        self.content_type()
            .map(|ct| ct.contains("application/json"))
            .unwrap_or(false)
    }

    /// Get request extensions.
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// Get mutable extensions.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    #[inline]
    pub(crate) fn set_path_params(&mut self, params: HashMap<String, String>) {
        self.path_params = params;
    }
}
