use async_trait::async_trait;
use rust_api::MiddlewareChain;
use rust_api::prelude::*;
use std::sync::Arc;

struct LoggerMiddleware;

#[async_trait]
impl<S: Send + Sync + 'static> Middleware<S> for LoggerMiddleware {
    async fn handle(&self, req: Req, _state: Arc<S>, next: Next<S>) -> Res {
        println!("[Logger] {} {}", req.method(), req.path());
        let res = next.run(req).await;
        println!("[Logger] Response: {}", res.status_code());
        res
    }
}

struct TimingMiddleware;

#[async_trait]
impl<S: Send + Sync + 'static> Middleware<S> for TimingMiddleware {
    async fn handle(&self, req: Req, _state: Arc<S>, next: Next<S>) -> Res {
        let start = std::time::Instant::now();
        let res = next.run(req).await;
        let elapsed = start.elapsed();
        println!("[Timing] Request took {:?}", elapsed);
        res
    }
}

struct AuthMiddleware;

#[async_trait]
impl<S: Send + Sync + 'static> Middleware<S> for AuthMiddleware {
    async fn handle(&self, req: Req, _state: Arc<S>, next: Next<S>) -> Res {
        // Check for auth header
        if let Some(auth) = req.header("authorization") {
            println!("[Auth] Authorized: {}", auth);
            next.run(req).await
        } else {
            println!("[Auth] No authorization header - blocking request");
            Res::builder()
                .status(401)
                .text("Unauthorized: Missing authorization header")
        }
    }
}

#[tokio::main]
async fn main() {
    // Example 1: Using MiddlewareChain builder
    let api_chain = Arc::new(
        MiddlewareChain::new()
            .add(TimingMiddleware)
            .add(LoggerMiddleware)
            .when(|req, _state| req.path().starts_with("/api"), AuthMiddleware)
            .build(),
    );

    let app = RustApi::new()
        .get("/", |_req: Req| async {
            Res::text("Public route - no auth needed")
        })
        .get("/api/users", |_req: Req| async {
            Res::text("Protected API route - auth required")
        })
        .get("/public", |_req: Req| async {
            Res::text("Another public route")
        });

    // Apply the composed middleware chain
    let app = app.layer(move |req, state, next| {
        let chain = Arc::clone(&api_chain);
        async move { chain.handle(req, state, next).await }
    });

    println!("Server starting on http://127.0.0.1:3006");
    println!("");
    println!("Try these requests:");
    println!("  1. curl http://127.0.0.1:3006/");
    println!("     -> Public route, no auth");
    println!("");
    println!("  2. curl http://127.0.0.1:3006/api/users");
    println!("     -> Will be blocked (no auth header)");
    println!("");
    println!("  3. curl -H 'Authorization: Bearer token123' http://127.0.0.1:3006/api/users");
    println!("     -> Will succeed (has auth header)");
    println!("");

    app.listen(([127, 0, 0, 1], 3006)).await.unwrap();
}
