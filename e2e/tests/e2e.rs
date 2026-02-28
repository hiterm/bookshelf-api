// E2E tests that run against a real Postgres instance.
// This lives in a dedicated crate inside the workspace so it can be run independently.

#![cfg(test)]

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use serial_test::serial;

/// Response payload for GET /me endpoint
#[derive(Debug, Deserialize)]
struct MeResponse {
    id: String,
}

fn get_server_url() -> Result<String> {
    let url = std::env::var("TEST_SERVER_URL")
        .context("TEST_SERVER_URL environment variable must be set. Please set it to the external server URL (e.g., http://localhost:8080)")?;
    Ok(url.trim_end_matches('/').to_owned())
}

#[tokio::test]
async fn e2e_health_check() -> Result<()> {
    let base_url = get_server_url()?;
    let addr = format!("{}/health", base_url);

    let client = Client::new();
    let res = client.get(&addr).send().await.context("request failed")?;
    assert!(res.status().is_success(), "health check should succeed");
    Ok(())
}

// ============================================
// GET /me Endpoint E2E Tests
// ============================================

#[tokio::test]
async fn e2e_me_endpoint_without_auth_returns_401() -> Result<()> {
    // Given: No authentication token provided
    let base_url = get_server_url()?;
    let me_addr = format!("{}/me", base_url);

    // When: Requesting /me without authentication
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .send()
        .await
        .context("request failed")?;

    // Then: Should return 401 Unauthorized
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /me without authentication"
    );

    // Verify error response structure
    let body = res
        .json::<serde_json::Value>()
        .await
        .context("invalid JSON")?;
    assert!(
        body.get("message").is_some(),
        "Error response should contain 'message' field"
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_me_endpoint_with_valid_token_returns_user_info() -> Result<()> {
    // Given: A valid JWT token
    let base_url = get_server_url()?;
    let me_addr = format!("{}/me", base_url);

    // Get valid token from environment
    let token = std::env::var("TEST_JWT_TOKEN").context("TEST_JWT_TOKEN must be set")?;

    // When: Requesting /me with valid authentication
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("request failed")?;

    // Then: Should return 200 OK with user info
    assert_eq!(
        res.status(),
        StatusCode::OK,
        "Expected 200 OK when accessing /me with valid token"
    );

    let body = res.json::<MeResponse>().await.context("invalid JSON")?;
    assert!(
        !body.id.is_empty(),
        "Response should contain non-empty user ID"
    );
    Ok(())
}

#[tokio::test]
async fn e2e_me_endpoint_with_invalid_token_returns_401() -> Result<()> {
    // Given: An invalid JWT token
    let base_url = get_server_url()?;
    let me_addr = format!("{}/me", base_url);

    // When: Requesting /me with invalid authentication
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", "Bearer invalid_token_here")
        .send()
        .await
        .context("request failed")?;

    // Then: Should return 401 Unauthorized
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /me with invalid token"
    );
    Ok(())
}

#[tokio::test]
async fn e2e_me_endpoint_with_malformed_auth_header_returns_401() -> Result<()> {
    // Given: Malformed Authorization header (missing "Bearer" prefix)
    let base_url = get_server_url()?;
    let me_addr = format!("{}/me", base_url);

    // When: Requesting /me with malformed Authorization header
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", "just_a_token_without_bearer_prefix")
        .send()
        .await
        .context("request failed")?;

    // Then: Should return 401 Unauthorized
    assert_eq!(
        res.status(),
        StatusCode::UNAUTHORIZED,
        "Expected 401 Unauthorized when accessing /me with malformed Authorization header"
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_me_endpoint_response_contains_required_fields() -> Result<()> {
    // Given: A valid JWT token and running server
    let base_url = get_server_url()?;
    let me_addr = format!("{}/me", base_url);

    // Get valid token from environment
    let token = std::env::var("TEST_JWT_TOKEN").context("TEST_JWT_TOKEN must be set")?;

    // When: Requesting /me with valid authentication
    let client = Client::new();
    let res = client
        .get(&me_addr)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .context("request failed")?;

    // Then: Response should have correct structure
    assert_eq!(res.status(), StatusCode::OK);

    let body = res
        .json::<serde_json::Value>()
        .await
        .context("invalid JSON")?;

    // Verify required field exists
    assert!(body.get("id").is_some(), "Response must contain 'id' field");

    // Verify id is a string
    assert!(
        body["id"].is_string(),
        "Response 'id' field must be a string"
    );

    // Verify id is not empty
    let id = body["id"].as_str().context("id should be a string")?;
    assert!(!id.is_empty(), "Response 'id' field must not be empty");
    Ok(())
}

// ============================================
// GraphQL E2E Tests
// ============================================

fn get_graphql_url() -> Result<String> {
    let base_url = get_server_url()?;
    Ok(format!("{}/graphql", base_url))
}

fn get_token() -> Result<String> {
    std::env::var("TEST_JWT_TOKEN")
        .context("TEST_JWT_TOKEN environment variable must be set for this test")
}

async fn graphql_request(query: &str, token: Option<&str>) -> Result<(u16, serde_json::Value)> {
    let client = Client::new();
    let url = get_graphql_url()?;

    let mut request = client
        .post(&url)
        .json(&serde_json::json!({ "query": query }));

    if let Some(t) = token {
        request = request.header("Authorization", format!("Bearer {}", t));
    }

    let res = request.send().await.context("request failed")?;
    let status = res.status().as_u16();

    // For 401, we still want to return the response body (for auth error checking)
    let _ = if !res.status().is_success() && res.status() != StatusCode::UNAUTHORIZED {
        return Err(anyhow::anyhow!("HTTP error: {}", res.status()));
    };
    let body = res
        .json::<serde_json::Value>()
        .await
        .context("invalid JSON")?;
    Ok((status, body))
}

async fn delete_test_book(book_id: &str) -> Result<()> {
    let token = get_token()?;
    let query = format!(r#"mutation {{ deleteBook(bookId: "{}") }}"#, book_id);
    let (status, response) = graphql_request(&query, Some(&token)).await?;

    if status != 200 {
        anyhow::bail!("deleteBook should return 200, got {}", status);
    }

    if let Some(errors) = response.get("errors") {
        anyhow::bail!("deleteBook has errors: {:?}", errors);
    }

    let data = response.get("data").context("data field must exist")?;
    let delete_result = data
        .get("deleteBook")
        .context("deleteBook field must exist")?;

    let delete_result_str = delete_result
        .as_str()
        .context("deleteResult should be a string")?;

    assert_eq!(
        delete_result_str, book_id,
        "deleted book id should match the requested book_id"
    );

    Ok(())
}

async fn ensure_user_registered() -> Result<()> {
    let token = get_token()?;
    let query = r#"mutation { registerUser { id } }"#;
    let (status, response) = graphql_request(query, Some(&token)).await?;

    // Always check for GraphQL errors, regardless of HTTP status
    if let Some(errors) = response.get("errors") {
        let error_message = errors
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("");

        if !error_message.contains("duplicate key") {
            anyhow::bail!("registerUser failed with error: {}", error_message);
        }
        // Duplicate key means user already exists - that's OK
        return Ok(());
    }

    // If no GraphQL errors, HTTP status should be 200
    if status != 200 {
        anyhow::bail!("registerUser failed with status {}", status);
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_without_auth_returns_error() -> Result<()> {
    let query = r#"{ books { id title } }"#;
    let (status, _response) = graphql_request(query, None).await?;

    assert_eq!(
        status, 401,
        "Expected 401 Unauthorized when accessing GraphQL without authentication"
    );
    Ok(())
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_books_empty() -> Result<()> {
    let token = get_token()?;
    let query = r#"{ books { id title } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let books = data.get("books").context("books field must exist")?;
    let books_array = books.as_array().context("books should be an array")?;
    assert!(
        books_array.is_empty(),
        "books should be empty after cleanup by other tests"
    );
    Ok(())
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_crud_book() -> Result<()> {
    let token = get_token()?;
    ensure_user_registered().await?;

    // Create author first
    let author_name = format!("Test Author for CRUD {}", uuid::Uuid::new_v4());
    let create_author_query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        author_name
    );
    let (_, response) = graphql_request(&create_author_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let author_result = data
        .get("createAuthor")
        .context("createAuthor field must exist")?;
    let author_id = author_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

    // Verify author was created by fetching it
    let author_query = format!(r#"{{ author(id: "{}") {{ id name }} }}"#, author_id);
    let (_, response) = graphql_request(&author_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let author = data.get("author").context("author field must exist")?;
    assert!(!author.is_null(), "author should exist after creation");
    let author_name_from_query = author
        .get("name")
        .context("name field must exist")?
        .as_str()
        .context("name should be string")?;
    assert_eq!(
        author_name_from_query, author_name,
        "author name should match"
    );

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
                createdAt
                updatedAt
            }}
        }}
        "#,
        author_id
    );
    let (_, response) = graphql_request(&create_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let create_result = data
        .get("createBook")
        .context("createBook field must exist")?;
    let book_id = create_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

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
    let (_, response) = graphql_request(&update_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let update_result = data
        .get("updateBook")
        .context("updateBook field must exist")?;
    assert_eq!(
        update_result
            .get("title")
            .context("title field must exist")?
            .as_str(),
        Some("Updated Test Book")
    );
    assert_eq!(
        update_result
            .get("read")
            .context("read field must exist")?
            .as_bool(),
        Some(true)
    );

    // Verify update by reading the book
    let query = format!(
        r#"{{ book(id: "{}") {{ id title read priority createdAt updatedAt authors {{ id name }} }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    let created_at = book
        .get("createdAt")
        .context("createdAt field must exist")?
        .as_i64()
        .context("createdAt should be i64")?;
    let updated_at = book
        .get("updatedAt")
        .context("updatedAt field must exist")?
        .as_i64()
        .context("updatedAt should be i64")?;
    assert!(created_at > 0, "created_at should be positive");
    assert!(updated_at > 0, "updated_at should be positive");
    // Verify updated_at >= created_at (update happened after create)
    assert!(
        updated_at >= created_at,
        "updated_at should be >= created_at"
    );
    assert_eq!(
        book.get("title")
            .context("title field must exist")?
            .as_str(),
        Some("Updated Test Book")
    );
    assert_eq!(
        book.get("read").context("read field must exist")?.as_bool(),
        Some(true)
    );
    assert_eq!(
        book.get("priority")
            .context("priority field must exist")?
            .as_i64(),
        Some(2)
    );
    let authors = book
        .get("authors")
        .context("authors field must exist")?
        .as_array()
        .context("authors should be an array")?;
    assert_eq!(authors.len(), 1, "should have 1 author");
    let author_from_book = authors[0]
        .get("id")
        .context("author id field must exist")?
        .as_str()
        .context("author id should be string")?;
    assert_eq!(
        author_from_book, author_id,
        "author id should match the created author"
    );

    // Delete book
    delete_test_book(book_id).await?;

    // Verify deletion
    let query = format!(r#"{{ book(id: "{}") {{ id title }} }}"#, book_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert!(book.is_null(), "book should be null after deletion");
    Ok(())
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_book_by_id() -> Result<()> {
    let token = get_token()?;
    ensure_user_registered().await?;

    // Create author first
    let author_name = format!("Test Author for BookByID {}", uuid::Uuid::new_v4());
    let create_author_query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        author_name
    );
    let (_, response) = graphql_request(&create_author_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let author_result = data
        .get("createAuthor")
        .context("createAuthor field must exist")?;
    let author_id = author_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

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
    let (_, response) = graphql_request(&create_query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let create_result = data
        .get("createBook")
        .context("createBook field must exist")?;
    let book_id = create_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id must be string")?;

    // Get book by ID
    let query = format!(
        r#"{{ book(id: "{}") {{ id title isbn read owned priority format store }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert_eq!(
        book.get("title")
            .context("title field must exist")?
            .as_str(),
        Some("Book By ID Test")
    );
    assert_eq!(
        book.get("isbn").context("isbn field must exist")?.as_str(),
        Some("9780123456789")
    );

    // Clean up
    delete_test_book(book_id).await?;
    Ok(())
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_authors() -> Result<()> {
    let token = get_token()?;
    let query = r#"{ authors { id name } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let authors = data.get("authors").context("authors field must exist")?;
    assert!(authors.is_array(), "authors should be an array");
    Ok(())
}

#[tokio::test]
#[serial]
#[ignore = "requires TEST_JWT_TOKEN environment variable"]
async fn e2e_graphql_create_author() -> Result<()> {
    // Note: Author deletion is not supported in the current GraphQL API.
    // Created authors will remain in the database.
    // Use random names to avoid conflicts.
    let token = get_token()?;
    ensure_user_registered().await?;

    let random_name = format!("Test Author {}", uuid::Uuid::new_v4());

    let query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id name }} }}"#,
        random_name
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let create_result = data
        .get("createAuthor")
        .context("createAuthor field must exist")?;
    assert!(
        create_result
            .get("id")
            .context("id field must exist")?
            .is_string(),
        "id field must exist"
    );
    assert_eq!(
        create_result
            .get("name")
            .context("name field must exist")?
            .as_str(),
        Some(random_name.as_str())
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_book_with_invalid_id() -> Result<()> {
    let token = get_token()?;
    let query = r#"{ book(id: "00000000-0000-0000-0000-000000000000") { id title } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert!(book.is_null(), "book with invalid ID should be null");
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_create_book_without_auth() -> Result<()> {
    let query = r#"
        mutation {
            createBook(bookData: {
                title: "Test Book"
                authorIds: []
                isbn: "9781234567890"
                read: false
                owned: true
                priority: 1
                format: E_BOOK
                store: KINDLE
            }) {
                id
            }
        }
    "#;
    let (status, _response) = graphql_request(query, None).await?;
    assert_eq!(status, 401, "createBook without auth should return 401");
    Ok(())
}
