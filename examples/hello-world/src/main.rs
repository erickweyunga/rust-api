use rust_api::prelude::*;

async fn hello(_req: Req) -> Res {
    Res::text("Hello, world!")
}

async fn greet(_req: Req) -> Res {
    Res::json(&serde_json::json!({
        "message": "Hello from Rust Api!",
        "framework": "rust-api"
    }))
}

#[tokio::main]
async fn main() {
    let app = RustApi::new().get("/", hello).get("/greet", greet);

    app.listen(([127, 0, 0, 1], 3000))
        .await
        .expect("Failed to start server");
}
