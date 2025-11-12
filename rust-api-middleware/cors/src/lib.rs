//! Production-grade CORS middleware for Rust Api.

use async_trait::async_trait;
use rust_api::{Middleware, Next, Req, Res};
use std::sync::Arc;

/// CORS configuration.
#[derive(Clone, Debug)]
pub struct CorsConfig {
    /// Allowed origins. Use "*" for all origins.
    pub allow_origins: Vec<String>,
    /// Allowed HTTP methods.
    pub allow_methods: Vec<String>,
    /// Allowed headers.
    pub allow_headers: Vec<String>,
    /// Exposed headers.
    pub expose_headers: Vec<String>,
    /// Maximum age for preflight cache in seconds.
    pub max_age: Option<u64>,
    /// Allow credentials.
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "PATCH".to_string(),
                "OPTIONS".to_string(),
            ],
            allow_headers: vec!["*".to_string()],
            expose_headers: vec![],
            max_age: Some(3600),
            allow_credentials: false,
        }
    }
}

impl CorsConfig {
    /// Create permissive CORS configuration allowing all origins.
    pub fn permissive() -> Self {
        Self::default()
    }

    /// Create restrictive CORS configuration.
    pub fn restrictive() -> Self {
        Self {
            allow_origins: vec![],
            allow_methods: vec!["GET".to_string(), "POST".to_string()],
            allow_headers: vec!["Content-Type".to_string()],
            expose_headers: vec![],
            max_age: Some(600),
            allow_credentials: false,
        }
    }

    /// Set allowed origins.
    pub fn allow_origins(mut self, origins: Vec<String>) -> Self {
        self.allow_origins = origins;
        self
    }

    /// Set allowed methods.
    pub fn allow_methods(mut self, methods: Vec<String>) -> Self {
        self.allow_methods = methods;
        self
    }

    /// Set allowed headers.
    pub fn allow_headers(mut self, headers: Vec<String>) -> Self {
        self.allow_headers = headers;
        self
    }

    /// Set exposed headers.
    pub fn expose_headers(mut self, headers: Vec<String>) -> Self {
        self.expose_headers = headers;
        self
    }

    /// Set max age for preflight cache in seconds.
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Enable or disable credentials.
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }
}

/// CORS middleware for handling Cross-Origin Resource Sharing.
#[derive(Clone)]
pub struct Cors {
    config: CorsConfig,
}

impl Cors {
    /// Create CORS middleware with custom configuration.
    pub fn new(config: CorsConfig) -> Self {
        Self { config }
    }

    /// Create CORS middleware with default permissive configuration.
    pub fn default() -> Self {
        Self::new(CorsConfig::default())
    }

    /// Create permissive CORS middleware allowing all origins.
    pub fn permissive() -> Self {
        Self::new(CorsConfig::permissive())
    }

    /// Create restrictive CORS middleware.
    pub fn restrictive() -> Self {
        Self::new(CorsConfig::restrictive())
    }

    fn is_origin_allowed(&self, origin: &str) -> bool {
        if self.config.allow_origins.contains(&"*".to_string()) {
            return true;
        }
        self.config.allow_origins.iter().any(|o| o == origin)
    }

    fn build_preflight_response(&self, origin: Option<&str>) -> Res {
        let mut res = Res::builder().status(204).text("");

        if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                // Access-Control-Allow-Origin
                res = if self.config.allow_origins.contains(&"*".to_string()) {
                    res.with_header("Access-Control-Allow-Origin", "*")
                } else {
                    res.with_header("Access-Control-Allow-Origin", origin)
                        .with_header("Vary", "Origin")
                };

                // Access-Control-Allow-Methods
                if !self.config.allow_methods.is_empty() {
                    res = res.with_header(
                        "Access-Control-Allow-Methods",
                        self.config.allow_methods.join(", "),
                    );
                }

                // Access-Control-Allow-Headers
                if !self.config.allow_headers.is_empty() {
                    res = res.with_header(
                        "Access-Control-Allow-Headers",
                        self.config.allow_headers.join(", "),
                    );
                }

                // Access-Control-Max-Age
                if let Some(max_age) = self.config.max_age {
                    res = res.with_header("Access-Control-Max-Age", max_age.to_string());
                }

                // Access-Control-Allow-Credentials
                if self.config.allow_credentials {
                    res = res.with_header("Access-Control-Allow-Credentials", "true");
                }
            }
        }

        res
    }

    fn add_cors_headers(&self, mut res: Res, origin: Option<&str>) -> Res {
        if let Some(origin) = origin {
            if self.is_origin_allowed(origin) {
                // Access-Control-Allow-Origin
                res = if self.config.allow_origins.contains(&"*".to_string()) {
                    res.with_header("Access-Control-Allow-Origin", "*")
                } else {
                    res.with_header("Access-Control-Allow-Origin", origin)
                        .with_header("Vary", "Origin")
                };

                // Access-Control-Expose-Headers
                if !self.config.expose_headers.is_empty() {
                    res = res.with_header(
                        "Access-Control-Expose-Headers",
                        self.config.expose_headers.join(", "),
                    );
                }

                // Access-Control-Allow-Credentials
                if self.config.allow_credentials {
                    res = res.with_header("Access-Control-Allow-Credentials", "true");
                }
            }
        }

        res
    }
}

#[async_trait]
impl<S: Send + Sync + 'static> Middleware<S> for Cors {
    async fn handle(&self, req: Req, _state: Arc<S>, next: Next<S>) -> Res {
        let origin = req.header("origin").map(|s| s.to_string());
        let is_preflight = req.method() == "OPTIONS";

        // Handle preflight requests
        if is_preflight {
            return self.build_preflight_response(origin.as_deref());
        }

        // Handle actual requests
        let res = next.run(req).await;
        self.add_cors_headers(res, origin.as_deref())
    }
}
