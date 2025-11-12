use rust_api::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct UserPath {
    id: String, // Path parameters are always strings
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: String,
    page: Option<u32>,
    limit: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateUser {
    name: String,
    email: String,
    age: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct LoginForm {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    println!("Extractors Demo - Comprehensive Example");
    println!("=========================================\n");

    let app = RustApi::new()
        // 1. Path extractor - Extract route parameters
        .get("/users/{id}", |Path(params): Path<UserPath>| async move {
            Res::text(format!("Getting user with ID: {}", params.id))
        })
        // 2. Query extractor - Extract query parameters
        .get("/search", |Query(query): Query<SearchQuery>| async move {
            let page = query.page.unwrap_or(1);
            let limit = query.limit.unwrap_or(10);
            Res::text(format!(
                "Searching for '{}' - Page: {}, Limit: {}",
                query.q, page, limit
            ))
        })
        // 3. Json extractor - Parse JSON body
        .post("/users", |Json(user): Json<CreateUser>| async move {
            println!("Creating user: {:?}", user);
            Res::json(&serde_json::json!({
                "success": true,
                "user": user
            }))
        })
        // 4. Form extractor - Parse form data
        .post("/login", |Form(form): Form<LoginForm>| async move {
            println!("Login attempt - Username: {}", form.username);
            if form.username == "admin" && form.password == "secret" {
                Res::text("Login successful!")
            } else {
                Res::builder().status(401).text("Invalid credentials")
            }
        })
        // 5. Headers extractor - Access all headers
        .get("/headers", |Headers(headers): Headers| async move {
            let mut response = String::from("Request Headers:\n");
            for (name, value) in headers.iter() {
                if let Ok(v) = value.to_str() {
                    response.push_str(&format!("  {}: {}\n", name, v));
                }
            }
            Res::text(response)
        })
        // 6. BodyBytes extractor - Raw body access
        .post("/upload", |BodyBytes(data): BodyBytes| async move {
            let size = data.len();
            println!("Received {} bytes", size);
            Res::text(format!("Uploaded {} bytes", size))
        })
        // 7. Multiple extractors - Combine different extractors
        .post(
            "/posts/{id}/comments",
            |Path(path): Path<UserPath>,
             Query(query): Query<SearchQuery>,
             Json(body): Json<CreateUser>| async move {
                Res::text(format!(
                    "Post ID: {}, Query: {}, Body: {:?}",
                    path.id, query.q, body
                ))
            },
        )
        // 8. Headers + Json - Check auth header and parse body
        .post(
            "/api/users",
            |Headers(headers): Headers, Json(user): Json<CreateUser>| async move {
                if let Some(auth) = headers.get("authorization") {
                    if auth.to_str().ok() == Some("Bearer secret-token") {
                        return Res::json(&serde_json::json!({
                            "success": true,
                            "message": "User created",
                            "user": user
                        }));
                    }
                }
                Res::builder().status(401).json(&serde_json::json!({
                    "success": false,
                    "error": "Unauthorized"
                }))
            },
        )
        // 9. Path + Query combination
        .get(
            "/users/{id}/posts",
            |Path(path): Path<UserPath>, Query(query): Query<SearchQuery>| async move {
                Res::text(format!("User {} posts - Searching: {}", path.id, query.q))
            },
        )
        // Health check
        .get("/", |_req: Req| async {
            Res::text("Extractors Demo is running!")
        });

    println!("Server running on http://127.0.0.1:3030\n");
    println!("Test the extractors:\n");

    println!("1. Path parameters:");
    println!("   curl http://127.0.0.1:3030/users/42\n");

    println!("2. Query parameters:");
    println!("   curl 'http://127.0.0.1:3030/search?q=rust&page=2&limit=20'\n");

    println!("3. JSON body:");
    println!(r#"   curl -X POST http://127.0.0.1:3030/users \"#);
    println!(r#"        -H 'Content-Type: application/json' \"#);
    println!(r#"        -d '{{"name":"Alice","email":"alice@example.com","age":25}}'"#);
    println!();

    println!("4. Form data:");
    println!(r#"   curl -X POST http://127.0.0.1:3030/login \"#);
    println!(r#"        -H 'Content-Type: application/x-www-form-urlencoded' \"#);
    println!(r#"        -d 'username=admin&password=secret'"#);
    println!();

    println!("5. Headers:");
    println!(r#"   curl http://127.0.0.1:3030/headers -H 'X-Custom: test-value'"#);
    println!();

    println!("6. Raw body bytes:");
    println!(r#"   curl -X POST http://127.0.0.1:3030/upload -d 'raw binary data here'"#);
    println!();

    println!("7. Multiple extractors:");
    println!(r#"   curl -X POST 'http://127.0.0.1:3030/posts/10/comments?q=test' \"#);
    println!(r#"        -H 'Content-Type: application/json' \"#);
    println!(r#"        -d '{{"name":"Bob","email":"bob@example.com"}}'"#);
    println!();

    println!("8. Auth + JSON:");
    println!(r#"   curl -X POST http://127.0.0.1:3030/api/users \"#);
    println!(r#"        -H 'Content-Type: application/json' \"#);
    println!(r#"        -H 'Authorization: Bearer secret-token' \"#);
    println!(r#"        -d '{{"name":"Charlie","email":"charlie@example.com"}}'"#);
    println!();

    println!("9. Path + Query combination:");
    println!(r#"   curl 'http://127.0.0.1:3030/users/5/posts?q=search'"#);
    println!();

    app.listen(([127, 0, 0, 1], 3030)).await.unwrap();
}
