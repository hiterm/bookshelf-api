// E2E tests that run against a real Postgres instance.
// This lives in a dedicated crate inside the workspace so it can be run independently.

#![cfg(test)]

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serial_test::serial;

/// Response payload for GET /me endpoint
#[derive(Debug, Deserialize)]
struct MeResponse {
    id: String,
}

fn get_server_url() -> String {
    std::env::var("TEST_SERVER_URL")
        .expect("TEST_SERVER_URL environment variable must be set. Please set it to the external server URL (e.g., http://localhost:8080)")
}

#[tokio::test]
async fn e2e_health_check() {
    let base_url = get_server_url();
    let addr = format!("{}/health", base_url);

    let client = Client::new();
    let res = client.get(&addr).send().await.expect("request failed");
    assert!(res.status().is_success());
}

// ============================================
// GET /me Endpoint E2E Tests
// ============================================

#[tokio::test]
async fn e2e_me_endpoint_without_auth_returns_401() {
    // Given: No authentication token provided
    let base_url = get_server_url();
    let me_addr = format!("{}/me", base_url);

    // When: Requesting /me without authentication
    let client = Client::new();
    let res = client.get(&me_addr).send().await.expect("request failed");

    // Then: Should return 401 Unauthorized
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /me without authentication"
    );

    // Verify error response structure
    let body = res.json::<serde_json::Value>().await.expect("invalid JSON");
    assert!(
        body.get("message").is_some(),
        "Error response should contain 'message' field"
    );
}

#[tokio::test]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_me_endpoint_with_valid_token_returns_user_info() {
    // Given: A valid JWT token
    let base_url = get_server_url();
    let me_addr = format!("{}/me", base_url);

    // Get valid token from environment
    let token = std::env::var("TEST_JWT_TOKEN")
        .expect("TEST_JWT_TOKEN environment variable must be set for this test");

    // When: Requesting /me with valid authentication
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("request failed");

    // Then: Should return 200 OK with user info
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Expected 200 OK when accessing /me with valid token"
    );

    let body = res.json::<MeResponse>().await.expect("invalid JSON");
    assert!(
        !body.id.is_empty(),
        "Response should contain non-empty user ID"
    );
}

#[tokio::test]
async fn e2e_me_endpoint_with_invalid_token_returns_401() {
    // Given: An invalid JWT token
    let base_url = get_server_url();
    let me_addr = format!("{}/me", base_url);

    // When: Requesting /me with invalid authentication
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", "Bearer invalid_token_here")
        .send()
        .await
        .expect("request failed");

    // Then: Should return 401 Unauthorized
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /me with invalid token"
    );
}

#[tokio::test]
async fn e2e_me_endpoint_with_malformed_auth_header_returns_401() {
    // Given: Malformed Authorization header (missing "Bearer" prefix)
    let base_url = get_server_url();
    let me_addr = format!("{}/me", base_url);

    // When: Requesting /me with malformed Authorization header
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", "just_a_token_without_bearer_prefix")
        .send()
        .await
        .expect("request failed");

    // Then: Should return 401 Unauthorized
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /me with malformed Authorization header"
    );
}

#[tokio::test]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_me_endpoint_response_contains_required_fields() {
    // Given: A valid JWT token and running server
    let base_url = get_server_url();
    let me_addr = format!("{}/me", base_url);

    // Get valid token from environment
    let token = std::env::var("TEST_JWT_TOKEN")
        .expect("TEST_JWT_TOKEN environment variable must be set for this test");

    // When: Requesting /me with valid authentication
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("request failed");

    // Then: Response should have correct structure
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.json::<serde_json::Value>().await.expect("invalid JSON");

    // Verify required field exists
    assert!(body.get("id").is_some(), "Response must contain 'id' field");

    // Verify id is a string
    assert!(
        body["id"].is_string(),
        "Response 'id' field must be a string"
    );

    // Verify id is not empty
    let id = body["id"].as_str().unwrap();
    assert!(!id.is_empty(), "Response 'id' field must not be empty");
}

// ============================================
// GraphQL E2E Tests
// ============================================

fn get_graphql_url() -> String {
    let base_url = get_server_url();
    format!("{}/graphql", base_url)
}

fn get_token() -> String {
    std::env::var("TEST_JWT_TOKEN")
        .expect("TEST_JWT_TOKEN environment variable must be set for this test")
}

async fn graphql_request(query: &str, token: Option<&str>) -> (u16, serde_json::Value) {
    let client = Client::new();
    let url = get_graphql_url();

    let mut request = client
        .post(&url)
        .json(&serde_json::json!({ "query": query }));

    if let Some(t) = token {
        request = request.header("Authorization", format!("Bearer {}", t));
    }

    let res = request.send().await.expect("request failed");
    let status = res.status().as_u16();
    let body = res.json::<serde_json::Value>().await.expect("invalid JSON");
    (status, body)
}

async fn delete_test_book(book_id: &str) {
    let token = get_token();
    let query = format!(r#"mutation {{ deleteBook(bookId: "{}") }}"#, book_id);
    let (status, response) = graphql_request(&query, Some(&token)).await;
    assert_eq!(status, 200, "deleteBook should return 200");
    let data = response.get("data").expect("data field must exist");
    assert!(data.get("deleteBook").is_some(), "deleteBook should succeed");
}

async fn ensure_user_registered() {
    let token = get_token();
    let query = r#"mutation { registerUser { id } }"#;
    let (status, response) = graphql_request(query, Some(&token)).await;
    if status != 200 {
        // Check if it's a duplicate key error (user already exists)
        if let Some(errors) = response.get("errors") {
            let error_message = errors.as_array()
                .and_then(|arr| arr.first())
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("");
            
            if !error_message.contains("duplicate key") {
                panic!("registerUser failed with non-duplicate error: {}", error_message);
            }
            // Duplicate key means user already exists - that's OK
        } else {
            panic!("registerUser failed with status {} and no errors", status);
        }
    }
}

#[tokio::test]
#[serial]
async fn e2e_graphql_without_auth_returns_error() {
    let query = r#"{ books { id title } }"#;
    let (status, _response) = graphql_request(query, None).await;

    assert_eq!(
        status, 401,
        "Expected 401 Unauthorized when accessing GraphQL without authentication"
    );
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_books_empty() {
    let token = get_token();
    let query = r#"{ books { id title } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await;

    let data = response.get("data").expect("data field must exist");
    let books = data.get("books").expect("books field must exist");
    let books_array = books.as_array().expect("books should be an array");
    assert!(
        books_array.is_empty(),
        "books should be empty after cleanup by other tests"
    );
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_crud_book() {
    let token = get_token();
    ensure_user_registered().await;

    // Create author first
    let author_name = format!("Test Author for CRUD {}", uuid::Uuid::new_v4());
    let create_author_query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        author_name
    );
    let (_, response) = graphql_request(&create_author_query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let author_result = data.get("createAuthor").expect("createAuthor field must exist");
    let author_id = author_result.get("id").expect("id field must exist").as_str().expect("id must be string");

    // Create book with author
    let create_query = format!(
        r#"
        mutation {{
            createBook(bookData: {{
                title: "Test Book"
                authorIds: ["{}"]
                isbn: "9783161484100"
                read: false
                owned: true
                priority: 1
                format: E_BOOK
                store: KINDLE
            }}) {{
                id
                title
                read
                owned
                priority
            }}
        }}
        "#,
        author_id
    );
    let (_, response) = graphql_request(&create_query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let create_result = data.get("createBook").expect("createBook field must exist");
    let book_id = create_result
        .get("id")
        .expect("id field must exist")
        .as_str()
        .expect("id must be string");

    // Update book
    let update_query = format!(
        r#"
        mutation {{
            updateBook(bookData: {{
                id: "{}"
                title: "Updated Test Book"
                authorIds: ["{}"]
                isbn: "9783161484100"
                read: true
                owned: true
                priority: 2
                format: PRINTED
                store: KINDLE
            }}) {{
                id
                title
                read
                priority
            }}
        }}
        "#,
        book_id, author_id
    );
    let (_, response) = graphql_request(&update_query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let update_result = data.get("updateBook").expect("updateBook field must exist");
    assert_eq!(
        update_result
            .get("title")
            .expect("title field must exist")
            .as_str(),
        Some("Updated Test Book")
    );
    assert_eq!(
        update_result
            .get("read")
            .expect("read field must exist")
            .as_bool(),
        Some(true)
    );
    let (_, response) = graphql_request(&update_query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let update_result = data.get("updateBook").expect("updateBook field must exist");
    assert_eq!(
        update_result
            .get("title")
            .expect("title field must exist")
            .as_str(),
        Some("Updated Test Book")
    );
    assert_eq!(
        update_result
            .get("read")
            .expect("read field must exist")
            .as_bool(),
        Some(true)
    );
    assert_eq!(
        update_result
            .get("priority")
            .expect("priority field must exist")
            .as_i64(),
        Some(2)
    );

    // Delete book
    let delete_query = format!(r#"mutation {{ deleteBook(bookId: "{}") }}"#, book_id);
    graphql_request(&delete_query, Some(&token)).await;

    // Verify deletion
    let query = format!(r#"{{ book(id: "{}") {{ id title }} }}"#, book_id);
    let (_, response) = graphql_request(&query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let book = data.get("book").expect("book field must exist");
    assert!(book.is_null(), "book should be null after deletion");
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_book_by_id() {
    let token = get_token();
    ensure_user_registered().await;

    // Create author first
    let author_name = format!("Test Author for BookByID {}", uuid::Uuid::new_v4());
    let create_author_query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        author_name
    );
    let (_, response) = graphql_request(&create_author_query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let author_result = data.get("createAuthor").expect("createAuthor field must exist");
    let author_id = author_result.get("id").expect("id field must exist").as_str().expect("id must be string");

    // Create book with author
    let create_query = format!(
        r#"
        mutation {{
            createBook(bookData: {{
                title: "Book By ID Test"
                authorIds: ["{}"]
                isbn: "9780123456789"
                read: false
                owned: true
                priority: 1
                format: E_BOOK
                store: KINDLE
            }}) {{
                id
                title
            }}
        }}
        "#,
        author_id
    );
    let (_, response) = graphql_request(&create_query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let create_result = data.get("createBook").expect("createBook field must exist");
    let book_id = create_result
        .get("id")
        .expect("id field must exist")
        .as_str()
        .expect("id must be string");

    // Get book by ID
    let query = format!(
        r#"{{ book(id: "{}") {{ id title isbn read owned priority format store }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await;
    let data = response.get("data").expect("data field must exist");
    let book = data.get("book").expect("book field must exist");
    assert_eq!(
        book.get("title").expect("title field must exist").as_str(),
        Some("Book By ID Test")
    );
    assert_eq!(
        book.get("isbn").expect("isbn field must exist").as_str(),
        Some("9780123456789")
    );

    // Clean up
    delete_test_book(book_id).await;
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_authors() {
    let token = get_token();
    let query = r#"{ authors { id name } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await;

    let data = response.get("data").expect("data field must exist");
    let authors = data.get("authors").expect("authors field must exist");
    assert!(authors.is_array(), "authors should be an array");
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_create_author() {
    // Note: Author deletion is not supported in the current GraphQL API.
    // Created authors will remain in the database.
    // Use random names to avoid conflicts.
    let token = get_token();
    ensure_user_registered().await;

    let random_name = format!("Test Author {}", uuid::Uuid::new_v4());

    let query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id name }} }}"#,
        random_name
    );
    let (_, response) = graphql_request(&query, Some(&token)).await;

    let data = response.get("data").expect("data field must exist");
    let create_result = data
        .get("createAuthor")
        .expect("createAuthor field must exist");
    assert!(
        create_result
            .get("id")
            .expect("id field must exist")
            .is_string(),
        "id field must exist"
    );
    assert_eq!(
        create_result
            .get("name")
            .expect("name field must exist")
            .as_str(),
        Some(random_name.as_str())
    );
}
