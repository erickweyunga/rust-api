//! Error handlers for Rust API framework.
//!
//! ## Available Handlers
//!
//! - `DefaultErrorHandler` - Plain text error responses
//! - `JsonErrorHandler` - JSON formatted error responses
//! - `FnErrorHandler` - Custom function-based handler

use rust_api::{Error, ErrorHandler, Res};

/// Default error handler with plain text responses.
#[derive(Debug, Clone, Copy)]
pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle(&self, error: Error) -> Res {
        match error {
            Error::Status(code, Some(msg)) => Res::builder()
                .status(code)
                .text(format!("{} {}", code, msg)),
            Error::Status(code, None) => Res::status(code),
            Error::Json(e) => Res::builder()
                .status(400)
                .text(format!("JSON error: {}", e)),
            Error::Hyper(e) => Res::builder()
                .status(500)
                .text(format!("HTTP error: {}", e)),
            Error::Io(e) => Res::builder().status(500).text(format!("IO error: {}", e)),
            Error::Custom(msg) => Res::builder().status(500).text(msg),
        }
    }
}

/// JSON error handler with structured error responses.
#[derive(Debug, Clone, Copy)]
pub struct JsonErrorHandler;

impl ErrorHandler for JsonErrorHandler {
    fn handle(&self, error: Error) -> Res {
        let (status_code, message) = match &error {
            Error::Status(code, Some(msg)) => (*code, msg.clone()),
            Error::Status(code, None) => (*code, status_text(*code)),
            Error::Json(e) => (400, format!("JSON error: {}", e)),
            Error::Hyper(e) => (500, format!("HTTP error: {}", e)),
            Error::Io(e) => (500, format!("IO error: {}", e)),
            Error::Custom(msg) => (500, msg.clone()),
        };

        let json = format!(
            r#"{{"error":"{}","status":{}}}"#,
            escape_json(&message),
            status_code
        );

        Res::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .text(json)
    }
}

/// Function-based error handler.
pub struct FnErrorHandler<F>(pub F);

impl<F> ErrorHandler for FnErrorHandler<F>
where
    F: Fn(Error) -> Res + Send + Sync + 'static,
{
    fn handle(&self, error: Error) -> Res {
        (self.0)(error)
    }
}

fn status_text(code: u16) -> String {
    match code {
        400 => "Bad Request".to_string(),
        401 => "Unauthorized".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        405 => "Method Not Allowed".to_string(),
        413 => "Payload Too Large".to_string(),
        422 => "Unprocessable Entity".to_string(),
        500 => "Internal Server Error".to_string(),
        502 => "Bad Gateway".to_string(),
        503 => "Service Unavailable".to_string(),
        _ => format!("HTTP {}", code),
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
