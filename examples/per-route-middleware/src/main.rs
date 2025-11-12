use rust_api::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() {
    let app = RustApi::new()
        // Global middleware - runs on ALL routes
        .layer(|req, _state, next| async move {
            println!("[Global] Request to: {}", req.path());
            let res = next.run(req).await;
            println!("[Global] Response status: {}", res.status_code());
            res
        })
        // Public routes - no additional middleware
        .get("/", |_req: Req| async {
            Res::text("Welcome! This is a public route.")
        })
        .get("/public", |_req: Req| async {
            Res::text("Another public route - only global middleware runs here")
        })
        // Admin route with authentication middleware
        .route(
            Route::get("/admin", |_req: Req| async {
                Res::text("Admin panel - you made it past auth!")
            })
            .layer(|req, _state, next| async move {
                println!("[Auth] Checking authorization for /admin");

                // Check for auth header
                if let Some(auth) = req.header("authorization") {
                    if auth.starts_with("Bearer ") {
                        println!("[Auth] ✓ Authorized");
                        next.run(req).await
                    } else {
                        println!("[Auth] ✗ Invalid token format");
                        Res::builder()
                            .status(401)
                            .text("Unauthorized: Invalid token format")
                    }
                } else {
                    println!("[Auth] ✗ No authorization header");
                    Res::builder()
                        .status(401)
                        .text("Unauthorized: Missing authorization header")
                }
            }),
        )
        // API route with rate limiting and validation
        .route(
            Route::post("/api/data", |_req: Req| async {
                Res::text("Data created successfully!")
            })
            // Rate limiting middleware
            .layer(|req, _state, next| async move {
                println!("[RateLimit] Checking rate limit");
                // In a real app, you'd check Redis or in-memory store
                println!("[RateLimit] ✓ Within limit");
                next.run(req).await
            })
            // Validation middleware
            .layer(|req, _state, next| async move {
                println!("[Validation] Checking request");

                // Check content-type
                if let Some(ct) = req.header("content-type") {
                    if ct.contains("application/json") {
                        println!("[Validation] ✓ Valid content-type");
                        next.run(req).await
                    } else {
                        println!("[Validation] ✗ Invalid content-type: {}", ct);
                        Res::builder()
                            .status(400)
                            .text("Bad Request: Content-Type must be application/json")
                    }
                } else {
                    println!("[Validation] ✗ Missing content-type");
                    Res::builder()
                        .status(400)
                        .text("Bad Request: Missing Content-Type header")
                }
            }),
        )
        // Protected API with auth + logging
        .route(
            Route::get("/api/users", |_req: Req| async {
                let users = json!({
                    "users": ["alice", "bob", "charlie"]
                });
                Res::json(&users)
            })
            .layer(|req, _state, next| async move {
                println!("[Auth] Checking API access");
                if req.header("authorization").is_some() {
                    println!("[Auth] ✓ API access granted");
                    next.run(req).await
                } else {
                    println!("[Auth] ✗ No API key");
                    Res::builder()
                        .status(401)
                        .text("Unauthorized: API key required")
                }
            })
            .layer(|req, _state, next| async move {
                println!("[Logging] API call from IP: [simulated]");
                let res = next.run(req).await;
                println!("[Logging] API response: {}", res.status_code());
                res
            }),
        )
        // Health check - global middleware only
        .get("/health", |_req: Req| async { Res::text("OK") });

    println!("Server running on http://127.0.0.1:3020\n");
    println!("Try these requests:\n");
    println!("1. Public routes (global middleware only):");
    println!("   curl http://127.0.0.1:3020/");
    println!("   curl http://127.0.0.1:3020/public\n");

    println!("2. Admin route (global + auth middleware):");
    println!("   curl http://127.0.0.1:3020/admin");
    println!("   curl -H 'Authorization: Bearer token123' http://127.0.0.1:3020/admin\n");

    println!("3. API POST route (global + rate limit + validation):");
    println!("   curl -X POST http://127.0.0.1:3020/api/data");
    println!("   curl -X POST -H 'Content-Type: text/plain' http://127.0.0.1:3020/api/data");
    println!(
        "   curl -X POST -H 'Content-Type: application/json' http://127.0.0.1:3020/api/data\n"
    );

    println!("4. API GET route (global + auth + logging):");
    println!("   curl http://127.0.0.1:3020/api/users");
    println!("   curl -H 'Authorization: Bearer secret' http://127.0.0.1:3020/api/users\n");

    println!("5. Health check (global middleware only):");
    println!("   curl http://127.0.0.1:3020/health\n");

    app.listen(([127, 0, 0, 1], 3020)).await.unwrap();
}
