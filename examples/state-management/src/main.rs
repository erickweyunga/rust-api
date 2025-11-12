use rust_api::prelude::*;
use serde::Serialize;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone)]
struct AppState {
    counter: Arc<AtomicU64>,
    app_name: String,
}

impl AppState {
    fn new() -> Self {
        Self {
            counter: Arc::new(AtomicU64::new(0)),
            app_name: "State Demo".to_string(),
        }
    }

    fn increment(&self) -> u64 {
        self.counter.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn get_count(&self) -> u64 {
        self.counter.load(Ordering::SeqCst)
    }
}

#[derive(Serialize)]
struct CountResponse {
    count: u64,
    app_name: String,
}

async fn home(_req: Req) -> Res {
    Res::text("State Management Example")
}

async fn count(_req: Req) -> Res {
    Res::json(&serde_json::json!({
        "count": 0,
        "note": "State extraction coming soon"
    }))
}

#[tokio::main]
async fn main() {
    let state = AppState::new();

    let app = RustApi::with_state(state)
        .get("/", home)
        .get("/count", count);

    app.listen(([127, 0, 0, 1], 3001))
        .await
        .expect("Failed to start server");
}
