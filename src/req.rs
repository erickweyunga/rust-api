//! HTTP request wrapper with streaming body support.
//!
//! The request body is consumed lazily on-demand when extractors need it,
//! avoiding unnecessary buffering for requests that don't use the body.

use bytes::Bytes;
use http_body_util::BodyExt;
use hyper::{Method, Request, Uri, body::Incoming, header};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::extensions::Extensions;
use crate::{Error, Result};

/// Lazy body consumer for streaming support.
/// Body is only consumed when first accessed by an extractor.
#[derive(Clone)]
enum Body {
    Streaming(Arc<Mutex<Option<Incoming>>>),
    Consumed(Bytes),
}

/// HTTP request with streaming body support.
///
/// Body consumption is deferred until first access, enabling:
/// - Zero-copy for requests without body extractors
/// - Single buffering for requests with multiple body extractors
/// - Efficient header-only routing decisions
pub struct Req {
    method: Method,
    uri: Uri,
    headers: header::HeaderMap,
    body: Body,
    path_params: HashMap<String, String>,
    extensions: Extensions,
}

impl Req {
    /// Create from hyper request.
    pub fn from_hyper(req: Request<Incoming>) -> Self {
        let (parts, body) = req.into_parts();

        Self {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
            body: Body::Streaming(Arc::new(Mutex::new(Some(body)))),
            path_params: HashMap::new(),
            extensions: Extensions::new(),
        }
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

    /// Get header value by name.
    #[inline]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }

    /// Get all headers.
    #[inline]
    pub fn headers(&self) -> &header::HeaderMap {
        &self.headers
    }

    /// Get path parameter by name.
    #[inline]
    pub fn param(&self, name: &str) -> Option<&str> {
        self.path_params.get(name).map(|s| s.as_str())
    }

    /// Get all path parameters.
    #[inline]
    pub fn params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    /// Get path parameters for extractors.
    #[inline]
    pub fn path_params(&self) -> &HashMap<String, String> {
        &self.path_params
    }

    /// Consume and read request body as bytes (lazy evaluation).
    /// Only called when body is actually needed by extractors.
    pub async fn body(&mut self) -> Result<&Bytes> {
        match &self.body {
            Body::Consumed(_) => {
                // Already consumed, return cached bytes
                if let Body::Consumed(ref bytes) = self.body {
                    Ok(bytes)
                } else {
                    unreachable!()
                }
            }
            Body::Streaming(incoming) => {
                // Consume streaming body on-demand
                let body_opt = incoming.lock().unwrap().take();

                if let Some(incoming_body) = body_opt {
                    let collected = incoming_body
                        .collect()
                        .await
                        .map_err(|e| Error::Custom(format!("Failed to read body: {}", e)))?;

                    let bytes = collected.to_bytes();
                    self.body = Body::Consumed(bytes.clone());

                    if let Body::Consumed(ref bytes) = self.body {
                        Ok(bytes)
                    } else {
                        unreachable!()
                    }
                } else {
                    // Body already taken (shouldn't happen with proper usage)
                    Err(Error::internal("Request body already consumed"))
                }
            }
        }
    }

    /// Get content type header value.
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

    /// Get mutable request extensions.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    #[inline]
    pub(crate) fn set_path_params(&mut self, params: HashMap<String, String>) {
        self.path_params = params;
    }
}
