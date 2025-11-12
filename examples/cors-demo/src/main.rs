use rust_api::prelude::*;
use rust_api_cors::{Cors, CorsConfig};

#[tokio::main]
async fn main() {
    // Example 1: Permissive CORS (allows all origins)
    let permissive_cors = Cors::permissive();

    // Example 2: Restrictive CORS (specific origins only)
    let _restrictive_cors = Cors::new(
        CorsConfig::restrictive()
            .allow_origins(vec![
                "http://localhost:3000".to_string(),
                "https://example.com".to_string(),
            ])
            .allow_methods(vec!["GET".to_string(), "POST".to_string()])
            .allow_headers(vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
            ])
            .allow_credentials(true)
            .max_age(7200),
    );

    // Example 3: Custom CORS
    let _custom_cors = Cors::new(
        CorsConfig::default()
            .allow_origins(vec!["http://localhost:8080".to_string()])
            .allow_methods(vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
            ])
            .allow_headers(vec![
                "Content-Type".to_string(),
                "X-Custom-Header".to_string(),
            ])
            .expose_headers(vec!["X-Total-Count".to_string()])
            .max_age(3600),
    );

    let app = RustApi::new()
        // Apply CORS middleware globally
        .layer(move |req, state, next| {
            let cors = permissive_cors.clone();
            async move { cors.handle(req, state, next).await }
        })
        .get("/", |_req: Req| async { Res::text("Hello with CORS!") })
        .get("/api/users", |_req: Req| async {
            Res::json(&serde_json::json!({
                "users": ["Alice", "Bob", "Charlie"]
            }))
        })
        .post("/api/users", |_req: Req| async {
            Res::json(&serde_json::json!({
                "success": true,
                "message": "User created"
            }))
        })
        .get("/health", |_req: Req| async { Res::text("OK") });

    app.listen(([127, 0, 0, 1], 3040)).await.unwrap();
}
