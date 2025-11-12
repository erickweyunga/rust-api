use rust_api::prelude::*;

async fn home(_req: Req) -> Res {
    Res::html(
        r#"
<!DOCTYPE html>
<html>
<head><title>File Serving Example</title></head>
<body>
    <h1>Static File Serving</h1>
    <p>Note: Full file streaming will be available in future version.</p>
    <p>For now, this demonstrates the API structure.</p>
    <ul>
        <li><a href="/about">About page (text)</a></li>
        <li><a href="/api">API info (json)</a></li>
    </ul>
</body>
</html>
    "#,
    )
}

async fn about(_req: Req) -> Res {
    Res::text("About this application\n\nBuilt with Rust Api framework.")
}

async fn api_info(_req: Req) -> Res {
    Res::json(&serde_json::json!({
        "name": "Rust Api",
        "version": "0.0.1",
        "features": ["routing", "state", "middleware"]
    }))
}

#[tokio::main]
async fn main() {
    let app = RustApi::new()
        .get("/", home)
        .get("/about", about)
        .get("/api", api_info);

    println!("Listening on http://127.0.0.1:3004");
    println!("Note: Full file streaming coming in future version");
    app.listen(([127, 0, 0, 1], 3004))
        .await
        .expect("Failed to start server");
}
