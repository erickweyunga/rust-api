//! Error handler trait for custom error responses.

use crate::{Error, Res};

/// Convert errors into HTTP responses.
pub trait ErrorHandler: Send + Sync + 'static {
    /// Convert error to response.
    fn handle(&self, error: Error) -> Res;
}
