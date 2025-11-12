//! # Rust Api
//!
//! Fast and scalable web framework for Rust.
//!
//! ## Example
//!
//! ```rust,no_run
//! use rust_api::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     let app = RustApi::new()
//!         .get("/", |_req: Req| async {
//!             Res::text("Hello, world!")
//!         })
//!         .get("/health", |_req: Req| async {
//!             Res::json(&serde_json::json!({"status": "healthy"}))
//!         });
//!
//!     app.listen(([127, 0, 0, 1], 3000)).await.unwrap();
//! }
//! ```
//!
//! ## Features
//!
//! - Fast async runtime with HTTP/1 and HTTP/2 support
//! - Intuitive routing with nested routers
//! - Type-safe state management
//! - Composable middleware
//! - Zero-cost abstractions
//! - Configurable request body size limits

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

mod api;
mod error;
mod handler;
mod into_res;
mod middleware;
mod req;
mod res;
mod router;

pub mod extractors;
pub mod layers;

// Re-exports
pub use api::RustApi;
pub use error::{Error, Result};
pub use handler::Handler;
pub use into_res::IntoRes;
pub use middleware::{Middleware, Next};
pub use req::Req;
pub use res::{Res, ResBuilder};
pub use router::Router;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::extractors::{Json, Path, Query, State};
    pub use crate::{Error, Handler, IntoRes, Middleware, Next, Req, Res, Result, Router, RustApi};
}
