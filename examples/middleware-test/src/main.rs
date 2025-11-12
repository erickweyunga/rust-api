use rust_api::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone)]
struct AppState {
    counter: Arc<AtomicU32>,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        counter: Arc::new(AtomicU32::new(0)),
    };

    let app = RustApi::with_state(state)
        .layer(|req, state, next| async move {
            let count = state.counter.fetch_add(1, Ordering::SeqCst) + 1;
            println!("Middleware 1: Before handler (Request #{})", count);
            let res = next.run(req).await;
            println!("Middleware 1: After handler");
            res
        })
        .layer(|req, state, next| async move {
            let count = state.counter.load(Ordering::SeqCst);
            println!("Middleware 2: Before handler (Counter: {})", count);
            let res = next.run(req).await;
            println!("Middleware 2: After handler");
            res
        })
        .get("/", |State(state): State<AppState>| async move {
            let count = state.counter.load(Ordering::SeqCst);
            println!("Handler: Processing request #{}", count);
            Res::text(format!(
                "Hello with middleware and state! Request #{}",
                count
            ))
        });

    println!("Starting server on http://127.0.0.1:3005");
    println!("Visit http://127.0.0.1:3005/ to test middleware with state");
    app.listen(([127, 0, 0, 1], 3005)).await.unwrap();
}
