use rust_api::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SearchParams {
    q: String,
    page: Option<u32>,
}

#[derive(Deserialize)]
struct ContactForm {
    name: String,
    email: String,
    message: Option<String>,
}

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
    age: u32,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

async fn home(_req: Req) -> Res {
    Res::html(
        r#"
<!DOCTYPE html>
<html>
<head><title>Extractor Examples</title></head>
<body>
    <h2>Form Handling with Extractors</h2>

    <h3>Form Extractor (POST)</h3>
    <form action="/submit" method="post">
        Name: <input type="text" name="name" required><br>
        Email: <input type="email" name="email" required><br>
        Message: <textarea name="message"></textarea><br>
        <input type="submit" value="Submit Form">
    </form>

    <h3>Query Extractor (GET)</h3>
    <p>Try: <a href="/search?q=rust&page=1">/search?q=rust&page=1</a></p>
    <p>Or: <a href="/search?q=web+framework">/search?q=web framework</a></p>

    <h3>JSON Extractor (POST)</h3>
    <p>POST to /api/users with JSON body:</p>
    <pre>
curl -X POST http://127.0.0.1:3003/api/users \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","age":25}'
    </pre>
</body>
</html>
    "#,
    )
}

async fn submit(Form(form): Form<ContactForm>) -> Res {
    Res::json(&ApiResponse {
        success: true,
        message: format!("Thank you, {}! We received your message.", form.name),
        data: Some(serde_json::json!({
            "email": form.email,
            "has_message": form.message.is_some(),
        })),
    })
}

async fn search(Query(params): Query<SearchParams>) -> Res {
    let page = params.page.unwrap_or(1);

    Res::json(&ApiResponse {
        success: true,
        message: format!("Search results for '{}' (page {})", params.q, page),
        data: Some(serde_json::json!({
            "query": params.q,
            "page": page,
            "results": ["Result 1", "Result 2", "Result 3"],
        })),
    })
}

async fn create_user(Json(user): Json<CreateUser>) -> Res {
    if user.age < 18 {
        return Res::json(&ApiResponse {
            success: false,
            message: "User must be at least 18 years old".to_string(),
            data: None,
        });
    }

    Res::json(&ApiResponse {
        success: true,
        message: format!("User '{}' created successfully", user.username),
        data: Some(serde_json::json!({
            "id": 1,
            "username": user.username,
            "email": user.email,
            "age": user.age,
        })),
    })
}

#[tokio::main]
async fn main() {
    let app = RustApi::new()
        .get("/", home)
        .post("/submit", submit)
        .get("/search", search)
        .post("/api/users", create_user);

    println!("Listening on http://127.0.0.1:3003");
    println!("");
    println!("Examples:");
    println!("  - GET  http://127.0.0.1:3003/");
    println!("  - GET  http://127.0.0.1:3003/search?q=rust&page=1");
    println!("  - POST http://127.0.0.1:3003/submit (form data)");
    println!("  - POST http://127.0.0.1:3003/api/users (JSON)");

    app.listen(([127, 0, 0, 1], 3003))
        .await
        .expect("Failed to start server");
}
