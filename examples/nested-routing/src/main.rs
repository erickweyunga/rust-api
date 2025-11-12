use rust_api::prelude::*;

async fn home(_req: Req) -> Res {
    Res::text("Nested Routing Example")
}

async fn health(_req: Req) -> Res {
    Res::json(&serde_json::json!({"status": "healthy"}))
}

// API v1 handlers
async fn v1_users(_req: Req) -> Res {
    Res::json(&serde_json::json!({
        "users": ["alice", "bob"],
        "version": "v1"
    }))
}

async fn v1_posts(_req: Req) -> Res {
    Res::json(&serde_json::json!({
        "posts": ["post1", "post2"],
        "version": "v1"
    }))
}

// API v2 handlers
async fn v2_users(_req: Req) -> Res {
    Res::json(&serde_json::json!({
        "users": [
            {"id": 1, "name": "alice"},
            {"id": 2, "name": "bob"}
        ],
        "version": "v2"
    }))
}

async fn v2_posts(_req: Req) -> Res {
    Res::json(&serde_json::json!({
        "posts": [
            {"id": 1, "title": "Post 1"},
            {"id": 2, "title": "Post 2"}
        ],
        "version": "v2"
    }))
}

#[tokio::main]
async fn main() {
    // API v1 routes
    let api_v1 = Router::new()
        .get("/users", v1_users)
        .get("/posts", v1_posts);

    // API v2 routes
    let api_v2 = Router::new()
        .get("/users", v2_users)
        .get("/posts", v2_posts);

    let app = RustApi::new()
        .get("/", home)
        .get("/health", health)
        .nest("/api/v1", api_v1)
        .nest("/api/v2", api_v2);

    app.listen(([127, 0, 0, 1], 3002))
        .await
        .expect("Failed to start server");
}
