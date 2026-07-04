// E2E tests that run against a real Postgres instance.
// This lives in a dedicated crate inside the workspace so it can be run independently.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::generate_test_token;
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
async fn e2e_me_endpoint_with_valid_token_returns_user_info() -> Result<()> {
    // Given: A valid JWT token
    let base_url = get_server_url()?;
    let me_addr = format!("{}/me", base_url);
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = generate_test_token(&user_id)?;

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
    assert_eq!(
        body.id, user_id,
        "Response ID should match the token subject"
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
async fn e2e_me_endpoint_response_contains_required_fields() -> Result<()> {
    // Given: A valid JWT token and running server
    let base_url = get_server_url()?;
    let me_addr = format!("{}/me", base_url);
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = generate_test_token(&user_id)?;

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
    if !res.status().is_success() && res.status() != StatusCode::UNAUTHORIZED {
        return Err(anyhow::anyhow!("HTTP error: {}", res.status()));
    }
    let body = res
        .json::<serde_json::Value>()
        .await
        .context("invalid JSON")?;
    Ok((status, body))
}

async fn delete_test_author(author_id: &str, token: &str) -> Result<()> {
    let query = format!(r#"mutation {{ deleteAuthor(authorId: "{}") }}"#, author_id);
    let (status, response) = graphql_request(&query, Some(token)).await?;

    if status != 200 {
        anyhow::bail!("deleteAuthor should return 200, got {}", status);
    }

    if let Some(errors) = response.get("errors") {
        anyhow::bail!("deleteAuthor has errors: {:?}", errors);
    }

    let data = response.get("data").context("data field must exist")?;
    let delete_result = data
        .get("deleteAuthor")
        .context("deleteAuthor field must exist")?;

    let delete_result_str = delete_result
        .as_str()
        .context("deleteAuthor result should be a string")?;

    assert_eq!(
        delete_result_str, author_id,
        "deleted author id should match the requested author_id"
    );

    Ok(())
}

async fn delete_test_book(book_id: &str, token: &str) -> Result<()> {
    let query = format!(r#"mutation {{ deleteBook(bookId: "{}") }}"#, book_id);
    let (status, response) = graphql_request(&query, Some(token)).await?;

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

async fn ensure_user_registered(token: &str) -> Result<()> {
    let query = r#"mutation { registerUser { id } }"#;
    let (status, response) = graphql_request(query, Some(token)).await?;

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

/// Creates a fresh, registered user and returns its `(user_id, token)`.
async fn create_test_user() -> Result<(String, String)> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = generate_test_token(&user_id)?;
    ensure_user_registered(&token).await?;
    Ok((user_id, token))
}

async fn create_test_author(name: &str, token: &str) -> Result<String> {
    let query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        name
    );
    let (_, response) = graphql_request(&query, Some(token)).await?;
    let id = response["data"]["createAuthor"]["id"]
        .as_str()
        .context("createAuthor id should be a string")?
        .to_owned();
    Ok(id)
}

async fn create_test_book(title: &str, author_id: &str, token: &str) -> Result<String> {
    let query = format!(
        r#"
        mutation {{
            createBook(bookData: {{
                title: "{}"
                authorIds: ["{}"]
                isbn: ""
                read: false
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }}) {{ id }}
        }}
        "#,
        title, author_id
    );
    let (_, response) = graphql_request(&query, Some(token)).await?;
    let id = response["data"]["createBook"]["id"]
        .as_str()
        .context("createBook id should be a string")?
        .to_owned();
    Ok(id)
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
async fn e2e_graphql_books_empty() -> Result<()> {
    // Use a fresh user ID so the books list is always empty for this user
    let (_user_id, token) = create_test_user().await?;

    let query = r#"{ books { id title } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let books = data.get("books").context("books field must exist")?;
    let books_array = books.as_array().context("books should be an array")?;
    assert!(
        books_array.is_empty(),
        "books should be empty for a new user"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_crud_book() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

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
    delete_test_book(book_id, &token).await?;

    // Verify deletion
    let query = format!(r#"{{ book(id: "{}") {{ id title }} }}"#, book_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert!(book.is_null(), "book should be null after deletion");

    // Clean up the author (book already deleted above)
    delete_test_author(author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_book_by_id() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

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

    // Clean up: delete book first, then author
    delete_test_book(book_id, &token).await?;
    delete_test_author(author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_authors() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // The authors list is user-scoped, so a fresh user starts empty.
    let query = r#"{ authors { id name } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;
    let authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    assert!(authors.is_empty(), "a fresh user should have no authors");

    // After creating one author, it should appear in the list.
    let author_name = format!("Listed Author {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    let (_, response) = graphql_request(query, Some(&token)).await?;
    let authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    assert_eq!(authors.len(), 1, "should list exactly the created author");
    assert_eq!(authors[0]["id"].as_str(), Some(author_id.as_str()));
    assert_eq!(authors[0]["name"].as_str(), Some(author_name.as_str()));

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_create_author() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Use a random name to keep the author unique across runs.
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
    let author_id = create_result
        .get("id")
        .context("id field must exist")?
        .as_str()
        .context("id should be string")?;
    assert_eq!(
        create_result
            .get("name")
            .context("name field must exist")?
            .as_str(),
        Some(random_name.as_str())
    );

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
        author_name_from_query, random_name,
        "author name should match"
    );

    delete_test_author(author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_book_with_invalid_id() -> Result<()> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = generate_test_token(&user_id)?;

    let query = r#"{ book(id: "00000000-0000-0000-0000-000000000000") { id title } }"#;
    let (_, response) = graphql_request(query, Some(&token)).await?;

    let data = response.get("data").context("data field must exist")?;
    let book = data.get("book").context("book field must exist")?;
    assert!(book.is_null(), "book with invalid ID should be null");
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_delete_author_without_books_succeeds() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let random_name = format!("Author To Delete {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&random_name, &token).await?;

    delete_test_author(&author_id, &token).await?;

    // Verify author no longer exists
    let query = format!(r#"{{ author(id: "{}") {{ id }} }}"#, author_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response["data"]["author"].is_null(),
        "author should be null after deletion"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_delete_author_with_associated_books_fails() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Create author
    let random_name = format!("Author With Book {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&random_name, &token).await?;

    // Create book associated with the author
    let book_id = create_test_book("Book Blocking Author Delete", &author_id, &token).await?;

    // Attempt to delete the author — must fail
    let delete_author_query = format!(r#"mutation {{ deleteAuthor(authorId: "{}") }}"#, author_id);
    let (_, response) = graphql_request(&delete_author_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "deleteAuthor should return errors when author has associated books"
    );

    // Verify the author still exists
    let query = format!(r#"{{ author(id: "{}") {{ id }} }}"#, author_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        !response["data"]["author"].is_null(),
        "author should still exist after failed deletion"
    );

    // Clean up: delete book first, then author
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_update_author() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Create author
    let original_name = format!("Author Before Update {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&original_name, &token).await?;

    // Create book associated with the author
    let book_id = create_test_book("Book For Author Update Test", &author_id, &token).await?;

    // Update author name while the author has an associated book
    let updated_name = format!("Author After Update {}", uuid::Uuid::new_v4());
    let update_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}" }}) {{ id name }} }}"#,
        author_id, updated_name
    );
    let (_, response) = graphql_request(&update_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "updateAuthor should not return errors"
    );
    let update_result = &response["data"]["updateAuthor"];
    assert_eq!(
        update_result["id"].as_str(),
        Some(author_id.as_str()),
        "updated author id should match"
    );
    assert_eq!(
        update_result["name"].as_str(),
        Some(updated_name.as_str()),
        "updated author name should match"
    );

    // Verify update by fetching the author
    let query = format!(r#"{{ author(id: "{}") {{ id name }} }}"#, author_id);
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert_eq!(
        response["data"]["author"]["name"].as_str(),
        Some(updated_name.as_str()),
        "author name should reflect the update"
    );

    // Clean up: delete book first, then author
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_update_nonexistent_author_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "Ghost" }}) {{ id name }} }}"#,
        nonexistent_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "updateAuthor should return errors for a non-existent author"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_graphql_delete_nonexistent_author_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let nonexistent_id = uuid::Uuid::new_v4().to_string();
    let query = format!(
        r#"mutation {{ deleteAuthor(authorId: "{}") }}"#,
        nonexistent_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "deleteAuthor should return errors for a non-existent author"
    );
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

// ============================================
// Change History E2E Tests
// ============================================

#[tokio::test]
#[serial]
async fn e2e_book_events_records_create_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("History Book Create", &author_id, &token).await?;

    let query = format!(
        r#"{{ bookEvents(bookId: "{}") {{
            eventId eventSetId operation bookId
            title authorIds isbn read owned priority format store
            bookCreatedAt bookUpdatedAt changedAt extra
        }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "bookEvents should not return errors: {:?}",
        response.get("errors")
    );

    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(entries.len(), 1, "should have exactly 1 event entry");

    let entry = &entries[0];
    assert!(!entry["eventId"].is_null(), "eventId should not be null");
    assert!(
        !entry["eventSetId"].is_null(),
        "eventSetId should not be null"
    );
    assert_eq!(entry["operation"].as_str(), Some("create"));
    assert_eq!(
        entry["bookId"].as_str(),
        Some(book_id.as_str()),
        "bookId should match"
    );
    assert_eq!(entry["title"].as_str(), Some("History Book Create"));
    assert_eq!(
        entry["authorIds"].as_array().and_then(|a| a[0].as_str()),
        Some(author_id.as_str()),
        "authorIds should contain the author"
    );
    assert_eq!(
        entry["isbn"].as_str(),
        Some(""),
        "isbn should be empty string"
    );
    assert_eq!(entry["read"].as_bool(), Some(false), "read should be false");
    assert_eq!(
        entry["owned"].as_bool(),
        Some(false),
        "owned should be false"
    );
    assert_eq!(
        entry["priority"].as_i64(),
        Some(50),
        "priority should be 50"
    );
    assert_eq!(
        entry["format"].as_str(),
        Some("E_BOOK"),
        "format should be E_BOOK"
    );
    assert_eq!(
        entry["store"].as_str(),
        Some("KINDLE"),
        "store should be KINDLE"
    );
    assert!(
        !entry["bookCreatedAt"].is_null(),
        "bookCreatedAt should not be null"
    );
    assert!(
        !entry["bookUpdatedAt"].is_null(),
        "bookUpdatedAt should not be null"
    );
    assert!(
        !entry["changedAt"].is_null(),
        "changedAt should not be null"
    );
    assert!(
        entry["extra"].is_null(),
        "create event extra should be null"
    );

    // Cleanup
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_book_events_records_update_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("Original Title", &author_id, &token).await?;

    // Update the book
    let update_query = format!(
        r#"
        mutation {{
            updateBook(bookData: {{
                id: "{}"
                title: "Updated Title"
                authorIds: ["{}"]
                isbn: ""
                read: false
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }}) {{ id title }}
        }}
        "#,
        book_id, author_id
    );
    let (_, response) = graphql_request(&update_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "updateBook should not return errors"
    );

    let query = format!(
        r#"{{ bookEvents(bookId: "{}") {{
            eventId eventSetId operation bookId
            title authorIds isbn read owned priority format store
            bookCreatedAt bookUpdatedAt changedAt extra
        }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        entries.len(),
        2,
        "should have 2 event entries (create + update)"
    );

    // Entries are ordered by changed_at DESC, so the most recent (update) is first.
    // Each entry records the post-state after the operation was applied.
    let update_entry = &entries[0];
    assert_eq!(update_entry["operation"].as_str(), Some("update"));
    assert!(
        !update_entry["eventId"].is_null(),
        "update eventId should not be null"
    );
    assert!(
        !update_entry["eventSetId"].is_null(),
        "update eventSetId should not be null"
    );
    assert_eq!(update_entry["bookId"].as_str(), Some(book_id.as_str()));
    assert_eq!(update_entry["title"].as_str(), Some("Updated Title"));
    assert_eq!(
        update_entry["authorIds"]
            .as_array()
            .and_then(|a| a[0].as_str()),
        Some(author_id.as_str()),
    );
    assert_eq!(update_entry["isbn"].as_str(), Some(""));
    assert_eq!(update_entry["read"].as_bool(), Some(false));
    assert_eq!(update_entry["owned"].as_bool(), Some(false));
    assert_eq!(update_entry["priority"].as_i64(), Some(50));
    assert_eq!(update_entry["format"].as_str(), Some("E_BOOK"));
    assert_eq!(update_entry["store"].as_str(), Some("KINDLE"));
    assert!(!update_entry["bookCreatedAt"].is_null());
    assert!(!update_entry["bookUpdatedAt"].is_null());
    assert!(!update_entry["changedAt"].is_null());
    assert!(
        update_entry["extra"].is_null(),
        "update event extra should be null"
    );

    let create_entry = &entries[1];
    assert_eq!(create_entry["operation"].as_str(), Some("create"));
    assert_eq!(create_entry["title"].as_str(), Some("Original Title"));

    // Cleanup
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_book_reverts_to_snapshot() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("Before Restore", &author_id, &token).await?;

    // Update so history has 2 entries; record create event's event_id
    let history_query = format!(
        r#"{{ bookEvents(bookId: "{}") {{ eventId operation title }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Update the book
    let update_query = format!(
        r#"
        mutation {{
            updateBook(bookData: {{
                id: "{}"
                title: "After Update"
                authorIds: ["{}"]
                isbn: ""
                read: true
                owned: false
                priority: 50
                format: E_BOOK
                store: KINDLE
            }}) {{ id title }}
        }}
        "#,
        book_id, author_id
    );
    graphql_request(&update_query, Some(&token)).await?;

    // Verify current title is "After Update"
    let book_query = format!(r#"{{ book(id: "{}") {{ title read }} }}"#, book_id);
    let (_, response) = graphql_request(&book_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["book"]["title"].as_str(),
        Some("After Update"),
        "title should be 'After Update' before restore"
    );

    // Restore to the create event state
    let restore_query = format!(
        r#"mutation {{ restoreBook(eventId: "{}") {{ id title read }} }}"#,
        create_event_id
    );
    let (_, response) = graphql_request(&restore_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "restoreBook should not return errors: {:?}",
        response.get("errors")
    );
    assert_eq!(
        response["data"]["restoreBook"]["title"].as_str(),
        Some("Before Restore"),
        "restored book should have create-event title"
    );
    assert_eq!(
        response["data"]["restoreBook"]["read"].as_bool(),
        Some(false),
        "restored book should have create-event read flag"
    );

    // Verify that the book in the DB reflects the restored state
    let (_, response) = graphql_request(&book_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["book"]["title"].as_str(),
        Some("Before Restore"),
        "book should reflect restored title"
    );

    // Cleanup
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_book_events_records_delete_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id =
        create_test_author(&format!("History Author {}", uuid::Uuid::new_v4()), &token).await?;
    let book_id = create_test_book("Book To Delete", &author_id, &token).await?;

    // Grab history before delete so we know the book_id for later query
    let history_query = format!(
        r#"{{ bookEvents(bookId: "{}") {{ eventId operation title }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries_before = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        entries_before.len(),
        1,
        "should have 1 history entry before delete"
    );

    // Delete the book
    delete_test_book(&book_id, &token).await?;

    // Even after deletion, history should be queryable and include the delete entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries_after = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        entries_after.len(),
        2,
        "should have 2 history entries after delete (create + delete)"
    );
    assert_eq!(
        entries_after[0]["operation"].as_str(),
        Some("delete"),
        "most recent entry should be 'delete'"
    );

    // Cleanup author
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_author_events_records_create_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_name = format!("Author History Create {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    let query = format!(
        r#"{{ authorEvents(authorId: "{}") {{
            eventId eventSetId operation authorId
            name yomi authorCreatedAt authorUpdatedAt changedAt extra
        }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "authorEvents should not return errors: {:?}",
        response.get("errors")
    );

    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(entries.len(), 1, "should have exactly 1 event entry");

    let entry = &entries[0];
    assert!(!entry["eventId"].is_null(), "eventId should not be null");
    assert!(
        !entry["eventSetId"].is_null(),
        "eventSetId should not be null"
    );
    assert_eq!(entry["operation"].as_str(), Some("create"));
    assert_eq!(
        entry["authorId"].as_str(),
        Some(author_id.as_str()),
        "authorId should match"
    );
    assert_eq!(entry["name"].as_str(), Some(author_name.as_str()));
    assert_eq!(
        entry["yomi"].as_str(),
        Some(""),
        "yomi defaults to empty string"
    );
    assert!(
        !entry["authorCreatedAt"].is_null(),
        "authorCreatedAt should not be null"
    );
    assert!(
        !entry["authorUpdatedAt"].is_null(),
        "authorUpdatedAt should not be null"
    );
    assert!(
        !entry["changedAt"].is_null(),
        "changedAt should not be null"
    );
    assert!(
        entry["extra"].is_null(),
        "create event extra should be null"
    );

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_author_events_records_update_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let original_name = format!("Author Before Update History {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&original_name, &token).await?;

    let updated_name = format!("Author After Update History {}", uuid::Uuid::new_v4());
    let update_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}" }}) {{ id name }} }}"#,
        author_id, updated_name
    );
    let (_, response) = graphql_request(&update_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "updateAuthor should not return errors"
    );

    let query = format!(
        r#"{{ authorEvents(authorId: "{}") {{
            eventId eventSetId operation authorId
            name yomi authorCreatedAt authorUpdatedAt changedAt extra
        }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        entries.len(),
        2,
        "should have 2 event entries (create + update)"
    );

    // Each entry records the post-state after the operation was applied.
    let update_entry = &entries[0];
    assert_eq!(update_entry["operation"].as_str(), Some("update"));
    assert!(!update_entry["eventId"].is_null());
    assert!(!update_entry["eventSetId"].is_null());
    assert_eq!(update_entry["authorId"].as_str(), Some(author_id.as_str()));
    assert_eq!(update_entry["name"].as_str(), Some(updated_name.as_str()));
    assert_eq!(
        update_entry["yomi"].as_str(),
        Some(""),
        "yomi should remain empty string"
    );
    assert!(!update_entry["authorCreatedAt"].is_null());
    assert!(!update_entry["authorUpdatedAt"].is_null());
    assert!(!update_entry["changedAt"].is_null());
    assert!(
        update_entry["extra"].is_null(),
        "update event extra should be null"
    );

    let create_entry = &entries[1];
    assert_eq!(create_entry["operation"].as_str(), Some("create"));
    assert_eq!(create_entry["name"].as_str(), Some(original_name.as_str()));

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_author_reverts_to_snapshot() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let original_name = format!("Restore Author Original {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&original_name, &token).await?;

    // Capture the create event's event_id
    let history_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ eventId operation name }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Update the author
    let updated_name = format!("Restore Author Updated {}", uuid::Uuid::new_v4());
    let update_query = format!(
        r#"mutation {{ updateAuthor(authorData: {{ id: "{}", name: "{}" }}) {{ id name }} }}"#,
        author_id, updated_name
    );
    let (_, update_response) = graphql_request(&update_query, Some(&token)).await?;
    assert!(
        update_response.get("errors").is_none(),
        "updateAuthor should not return errors"
    );
    assert_eq!(
        update_response["data"]["updateAuthor"]["name"].as_str(),
        Some(updated_name.as_str()),
        "updateAuthor should return updated name"
    );

    // Restore to the create event state
    let restore_query = format!(
        r#"mutation {{ restoreAuthor(eventId: "{}") {{ id name }} }}"#,
        create_event_id
    );
    let (_, response) = graphql_request(&restore_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "restoreAuthor should not return errors: {:?}",
        response.get("errors")
    );
    assert_eq!(
        response["data"]["restoreAuthor"]["name"].as_str(),
        Some(original_name.as_str()),
        "restored author should have create-event name"
    );

    // Verify DB reflects the restored state
    let author_query = format!(r#"{{ author(id: "{}") {{ name }} }}"#, author_id);
    let (_, response) = graphql_request(&author_query, Some(&token)).await?;
    assert_eq!(
        response["data"]["author"]["name"].as_str(),
        Some(original_name.as_str()),
        "author in DB should reflect restored name"
    );

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_author_events_records_delete_operation() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_name = format!("Author History Delete {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    // Confirm 1 history entry before delete
    let history_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ eventId operation name }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let before = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(before.len(), 1, "should have 1 history entry before delete");

    // Delete the author
    delete_test_author(&author_id, &token).await?;

    // History should survive deletion and include the delete entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let after = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        after.len(),
        2,
        "should have 2 history entries after delete (create + delete)"
    );
    assert_eq!(
        after[0]["operation"].as_str(),
        Some("delete"),
        "most recent entry should be 'delete'"
    );
    assert!(
        after[0]["name"].is_null(),
        "delete event records only author_id; name is null"
    );
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_book_records_restore_event() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_id = create_test_author(
        &format!("Restore Event Author {}", uuid::Uuid::new_v4()),
        &token,
    )
    .await?;
    let book_id = create_test_book("Original Title", &author_id, &token).await?;

    // Get the create event_id
    let history_query = format!(
        r#"{{ bookEvents(bookId: "{}") {{ eventId operation extra }} }}"#,
        book_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Restore to the create event
    let restore_query = format!(
        r#"mutation {{ restoreBook(eventId: "{}") {{ id title }} }}"#,
        create_event_id
    );
    graphql_request(&restore_query, Some(&token)).await?;

    // History should now include a restore event as the most recent entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["bookEvents"]
        .as_array()
        .context("bookEvents should be an array after restore")?;

    assert!(
        entries.len() >= 2,
        "should have at least 2 entries after restore"
    );
    assert_eq!(
        entries[0]["operation"].as_str(),
        Some("restore"),
        "most recent entry should be 'restore'"
    );
    // extra should contain source_event_id
    let extra = &entries[0]["extra"];
    assert!(!extra.is_null(), "restore event extra should not be null");
    let create_event_id_i64: i64 = create_event_id.parse().context("event_id should be i64")?;
    assert_eq!(
        extra["source_event_id"].as_i64(),
        Some(create_event_id_i64),
        "restore event extra should contain the source event id"
    );
    assert_eq!(
        extra["version"].as_i64(),
        Some(1),
        "restore event extra should contain version 1"
    );

    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_restore_author_records_restore_event() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let author_name = format!("Restore Event Author {}", uuid::Uuid::new_v4());
    let author_id = create_test_author(&author_name, &token).await?;

    // Get the create event_id
    let history_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ eventId operation extra }} }}"#,
        author_id
    );
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    let create_event_id = entries[0]["eventId"]
        .as_str()
        .context("eventId should be a string")?
        .to_owned();

    // Restore to the create event
    let restore_query = format!(
        r#"mutation {{ restoreAuthor(eventId: "{}") {{ id name }} }}"#,
        create_event_id
    );
    graphql_request(&restore_query, Some(&token)).await?;

    // History should now include a restore event as the most recent entry
    let (_, response) = graphql_request(&history_query, Some(&token)).await?;
    let entries = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array after restore")?;

    assert!(
        entries.len() >= 2,
        "should have at least 2 entries after restore"
    );
    assert_eq!(
        entries[0]["operation"].as_str(),
        Some("restore"),
        "most recent entry should be 'restore'"
    );
    let extra = &entries[0]["extra"];
    assert!(!extra.is_null(), "restore event extra should not be null");
    let create_event_id_i64: i64 = create_event_id.parse().context("event_id should be i64")?;
    assert_eq!(
        extra["source_event_id"].as_i64(),
        Some(create_event_id_i64),
        "restore event extra should contain the source event id"
    );
    assert_eq!(
        extra["version"].as_i64(),
        Some(1),
        "restore event extra should contain version 1"
    );

    delete_test_author(&author_id, &token).await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Pre-create an author
    let existing_author_id = create_test_author("Existing Author", &token).await?;

    // Call importBooks
    let import_query = r#"
        mutation {
            importBooks(books: [
                {
                    title: "Book One"
                    authorNames: ["Existing Author"]
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                },
                {
                    title: "Book Two"
                    authorNames: ["New Author"]
                    isbn: ""
                    read: false
                    owned: false
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }
            ]) {
                id
                title
                authors {
                    name
                }
            }
        }
    "#;
    let (_, response) = graphql_request(import_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "importBooks should not return errors: {:?}",
        response.get("errors")
    );

    let imported_books = response["data"]["importBooks"]
        .as_array()
        .context("importBooks should return an array")?;
    assert_eq!(imported_books.len(), 2, "should import exactly 2 books");

    let book_one_id = imported_books[0]["id"]
        .as_str()
        .context("book id should be string")?;
    let book_one_title = imported_books[0]["title"]
        .as_str()
        .context("book title should be string")?;
    let book_one_authors = imported_books[0]["authors"]
        .as_array()
        .context("book one authors should be an array")?;
    let book_two_id = imported_books[1]["id"]
        .as_str()
        .context("book id should be string")?;
    let book_two_title = imported_books[1]["title"]
        .as_str()
        .context("book title should be string")?;
    let book_two_authors = imported_books[1]["authors"]
        .as_array()
        .context("book two authors should be an array")?;

    assert_eq!(book_one_title, "Book One");
    assert_eq!(book_one_authors.len(), 1);
    assert_eq!(
        book_one_authors[0]["name"].as_str(),
        Some("Existing Author")
    );
    assert_eq!(book_two_title, "Book Two");
    assert_eq!(book_two_authors.len(), 1);
    assert_eq!(book_two_authors[0]["name"].as_str(), Some("New Author"));
    assert!(
        !book_one_id.is_empty(),
        "book one should have a non-empty id"
    );
    assert!(
        !book_two_id.is_empty(),
        "book two should have a non-empty id"
    );
    assert_ne!(book_one_id, book_two_id, "book ids should be distinct");

    // Locate the single event set produced by the import via eventSets, then
    // inspect it directly. This replaces the old workaround of comparing the
    // eventSetId across separate per-book bookEvents queries.
    let (_, response) = graphql_request(r#"{ eventSets { id operation } }"#, Some(&token)).await?;
    let event_sets = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?;
    let import_set_id = event_sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("import_books"))
        .and_then(|s| s["id"].as_str())
        .context("there should be an import_books event set")?;

    // The shared event set groups both book creates and the new author create.
    let event_set_query = format!(
        r#"{{ eventSet(id: "{}") {{
            operation
            bookEvents {{ bookId operation }}
            authorEvents {{ name operation }}
        }} }}"#,
        import_set_id
    );
    let (_, response) = graphql_request(&event_set_query, Some(&token)).await?;
    let event_set = &response["data"]["eventSet"];
    assert!(!event_set.is_null(), "import event set should be found");
    assert_eq!(
        event_set["operation"].as_str(),
        Some("import_books"),
        "import event set operation should be import_books"
    );
    let grouped_book_events = event_set["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        grouped_book_events.len(),
        2,
        "import event set should group both book create events"
    );
    assert!(
        grouped_book_events
            .iter()
            .all(|e| e["operation"].as_str() == Some("create")),
        "all grouped book events should be create events"
    );
    let grouped_book_ids: Vec<&str> = grouped_book_events
        .iter()
        .filter_map(|e| e["bookId"].as_str())
        .collect();
    assert!(grouped_book_ids.contains(&book_one_id));
    assert!(grouped_book_ids.contains(&book_two_id));
    let grouped_author_events = event_set["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        grouped_author_events.len(),
        1,
        "import event set should group the newly created author event"
    );
    assert_eq!(
        grouped_author_events[0]["name"].as_str(),
        Some("New Author"),
        "grouped author event should be the new author"
    );
    assert_eq!(
        grouped_author_events[0]["operation"].as_str(),
        Some("create")
    );

    // Verify Book Two has "New Author"
    let book_two_query = format!(
        r#"{{ book(id: "{}") {{ authors {{ name }} }} }}"#,
        book_two_id
    );
    let (_, response) = graphql_request(&book_two_query, Some(&token)).await?;
    let authors = response["data"]["book"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    assert_eq!(authors.len(), 1, "book two should have 1 author");
    assert_eq!(
        authors[0]["name"].as_str(),
        Some("New Author"),
        "book two author should be New Author"
    );

    // Verify "Existing Author" was not duplicated
    let authors_query = r#"{ authors { id name } }"#;
    let (_, response) = graphql_request(authors_query, Some(&token)).await?;
    let all_authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    let existing_count = all_authors
        .iter()
        .filter(|a| a["name"].as_str() == Some("Existing Author"))
        .count();
    assert_eq!(
        existing_count, 1,
        "Existing Author should appear exactly once"
    );

    // Verify New Author has a create event in authorEvents
    let new_author = all_authors
        .iter()
        .find(|a| a["name"].as_str() == Some("New Author"));
    let new_author_id = new_author
        .and_then(|a| a["id"].as_str())
        .context("new author should have an id")?;
    let author_events_query = format!(
        r#"{{ authorEvents(authorId: "{}") {{ operation name }} }}"#,
        new_author_id
    );
    let (_, response) = graphql_request(&author_events_query, Some(&token)).await?;
    let author_events = response["data"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        author_events.len(),
        1,
        "new author should have exactly 1 event entry"
    );
    assert_eq!(
        author_events[0]["operation"].as_str(),
        Some("create"),
        "new author event should be create"
    );
    assert_eq!(
        author_events[0]["name"].as_str(),
        Some("New Author"),
        "new author event name should match"
    );

    // Cleanup
    delete_test_book(book_one_id, &token).await?;
    delete_test_book(book_two_id, &token).await?;
    delete_test_author(&existing_author_id, &token).await?;

    // Delete New Author
    delete_test_author(new_author_id, &token).await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books_many_entries() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;
    let run_id = uuid::Uuid::new_v4();

    let existing_author_names: Vec<String> = (0..3)
        .map(|i| format!("Bulk Existing Author {run_id} {i}"))
        .collect();
    let mut author_ids = Vec::new();
    for name in &existing_author_names {
        author_ids.push(create_test_author(name, &token).await?);
    }

    let import_count = 25;
    let imported_entries = (0..import_count)
        .map(|i| {
            let title = format!("Bulk Import Book {run_id} {i:02}");
            let existing_author_name = &existing_author_names[i % existing_author_names.len()];
            let new_author_name = format!("Bulk New Author {run_id} {i:02}");
            format!(
                r#"{{
                    title: "{title}"
                    authorNames: ["{existing_author_name}", "{new_author_name}"]
                    isbn: "978000000{i:04}"
                    read: false
                    owned: true
                    priority: 50
                    format: E_BOOK
                    store: KINDLE
                }}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    let import_query = format!(
        r#"
        mutation {{
            importBooks(books: [{imported_entries}]) {{
                id
                title
                authors {{ name }}
            }}
        }}
        "#
    );
    let (_, response) = graphql_request(&import_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_none(),
        "bulk importBooks should not return errors: {:?}",
        response.get("errors")
    );

    let imported_books = response["data"]["importBooks"]
        .as_array()
        .context("importBooks should return an array")?;
    assert_eq!(
        imported_books.len(),
        import_count,
        "bulk import should return every requested book"
    );

    let mut imported_book_ids = Vec::new();
    for (i, book) in imported_books.iter().enumerate() {
        let book_id = book["id"]
            .as_str()
            .context("imported book id should be a string")?;
        imported_book_ids.push(book_id.to_owned());
        assert_eq!(
            book["title"].as_str(),
            Some(format!("Bulk Import Book {run_id} {i:02}").as_str())
        );
        let authors = book["authors"]
            .as_array()
            .context("imported book authors should be an array")?;
        assert_eq!(authors.len(), 2, "each bulk imported book has 2 authors");
        assert!(authors.iter().any(|author| {
            author["name"].as_str()
                == Some(existing_author_names[i % existing_author_names.len()].as_str())
        }));
        assert!(authors.iter().any(|author| {
            author["name"].as_str() == Some(format!("Bulk New Author {run_id} {i:02}").as_str())
        }));
    }

    let (_, response) = graphql_request(r#"{ eventSets { id operation } }"#, Some(&token)).await?;
    let event_sets = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?;
    let import_set_id = event_sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("import_books"))
        .and_then(|s| s["id"].as_str())
        .context("there should be an import_books event set")?;

    let event_set_query = format!(
        r#"{{ eventSet(id: "{}") {{
            operation
            bookEvents {{ bookId operation }}
            authorEvents {{ name operation }}
        }} }}"#,
        import_set_id
    );
    let (_, response) = graphql_request(&event_set_query, Some(&token)).await?;
    let event_set = &response["data"]["eventSet"];
    assert_eq!(event_set["operation"].as_str(), Some("import_books"));
    let grouped_book_events = event_set["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(
        grouped_book_events.len(),
        import_count,
        "bulk import event set should group every book create event"
    );
    assert!(
        grouped_book_events
            .iter()
            .all(|event| event["operation"].as_str() == Some("create")),
        "all bulk import book events should be create events"
    );
    for book_id in &imported_book_ids {
        assert!(
            grouped_book_events
                .iter()
                .any(|event| event["bookId"].as_str() == Some(book_id.as_str())),
            "bulk import event set should contain book id {book_id}"
        );
    }

    let grouped_author_events = event_set["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        grouped_author_events.len(),
        import_count,
        "bulk import event set should only group newly created author events"
    );
    for i in 0..import_count {
        let new_author_name = format!("Bulk New Author {run_id} {i:02}");
        assert!(
            grouped_author_events.iter().any(|event| {
                event["operation"].as_str() == Some("create")
                    && event["name"].as_str() == Some(new_author_name.as_str())
            }),
            "bulk import event set should contain author event for {new_author_name}"
        );
    }

    let (_, response) = graphql_request(r#"{ authors { id name } }"#, Some(&token)).await?;
    let all_authors = response["data"]["authors"]
        .as_array()
        .context("authors should be an array")?;
    for existing_author_name in &existing_author_names {
        assert_eq!(
            all_authors
                .iter()
                .filter(|author| author["name"].as_str() == Some(existing_author_name.as_str()))
                .count(),
            1,
            "existing bulk import authors should not be duplicated"
        );
    }
    for i in 0..import_count {
        let new_author_name = format!("Bulk New Author {run_id} {i:02}");
        let new_author_id = all_authors
            .iter()
            .find(|author| author["name"].as_str() == Some(new_author_name.as_str()))
            .and_then(|author| author["id"].as_str())
            .context("new bulk import author should exist")?;
        author_ids.push(new_author_id.to_owned());
    }

    for book_id in &imported_book_ids {
        delete_test_book(book_id, &token).await?;
    }
    for author_id in &author_ids {
        delete_test_author(author_id, &token).await?;
    }

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_import_books_empty_returns_error() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    let import_query = r#"mutation { importBooks(books: []) { id } }"#;
    let (_, response) = graphql_request(import_query, Some(&token)).await?;
    assert!(
        response.get("errors").is_some(),
        "importBooks with empty list should return errors: {:?}",
        response.get("errors")
    );

    Ok(())
}

#[tokio::test]
#[serial]
async fn e2e_event_sets() -> Result<()> {
    let (_user_id, token) = create_test_user().await?;

    // Two separate operations => two event sets (create_author, then create_book).
    let author_id = create_test_author(
        &format!("Event Set Author {}", uuid::Uuid::new_v4()),
        &token,
    )
    .await?;
    let book_id = create_test_book("Event Set Book", &author_id, &token).await?;

    // List event sets: newest first, exactly the two operations above.
    let (_, response) =
        graphql_request(r#"{ eventSets { id operation createdAt } }"#, Some(&token)).await?;
    let sets = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?;
    assert_eq!(sets.len(), 2, "fresh user should have exactly 2 event sets");
    // Ordered by createdAt descending (newest first).
    let created0 = sets[0]["createdAt"].as_i64().context("createdAt is i64")?;
    let created1 = sets[1]["createdAt"].as_i64().context("createdAt is i64")?;
    assert!(created0 >= created1, "event sets should be newest first");
    let operations: Vec<&str> = sets
        .iter()
        .filter_map(|s| s["operation"].as_str())
        .collect();
    assert!(
        operations.contains(&"create_book"),
        "should contain a create_book event set"
    );
    assert!(
        operations.contains(&"create_author"),
        "should contain a create_author event set"
    );

    // Drill into the create_book event set: it groups the book's create event.
    let create_book_set_id = sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("create_book"))
        .and_then(|s| s["id"].as_str())
        .context("create_book event set should have an id")?;
    let detail_query = format!(
        r#"{{ eventSet(id: "{}") {{
            id operation createdAt
            bookEvents {{ bookId operation title changedAt }}
            authorEvents {{ name operation }}
        }} }}"#,
        create_book_set_id
    );
    let (_, response) = graphql_request(&detail_query, Some(&token)).await?;
    let detail = &response["data"]["eventSet"];
    assert!(!detail.is_null(), "eventSet should be found");
    assert_eq!(
        detail["id"].as_str(),
        Some(create_book_set_id),
        "eventSet id should round-trip the queried id"
    );
    assert_eq!(detail["operation"].as_str(), Some("create_book"));
    assert!(
        detail["createdAt"].as_i64().is_some(),
        "createdAt should be an Int"
    );
    let book_events = detail["bookEvents"]
        .as_array()
        .context("bookEvents should be an array")?;
    assert_eq!(book_events.len(), 1, "create_book set has one book event");
    assert_eq!(book_events[0]["bookId"].as_str(), Some(book_id.as_str()));
    assert_eq!(book_events[0]["operation"].as_str(), Some("create"));
    // The nested entry must carry the full BookEventEntry data, not just ids
    // (guards the find_by_event_set projection).
    assert_eq!(
        book_events[0]["title"].as_str(),
        Some("Event Set Book"),
        "nested book event should carry the title"
    );
    assert!(
        !book_events[0]["changedAt"].is_null(),
        "nested book event changedAt should not be null"
    );
    assert!(
        detail["authorEvents"]
            .as_array()
            .context("authorEvents should be an array")?
            .is_empty(),
        "create_book set has no author events"
    );

    // Drill into the create_author event set: it groups the author's create event.
    let create_author_set_id = sets
        .iter()
        .find(|s| s["operation"].as_str() == Some("create_author"))
        .and_then(|s| s["id"].as_str())
        .context("create_author event set should have an id")?;
    let detail_query = format!(
        r#"{{ eventSet(id: "{}") {{ authorEvents {{ name operation }} bookEvents {{ bookId }} }} }}"#,
        create_author_set_id
    );
    let (_, response) = graphql_request(&detail_query, Some(&token)).await?;
    let author_events = response["data"]["eventSet"]["authorEvents"]
        .as_array()
        .context("authorEvents should be an array")?;
    assert_eq!(
        author_events.len(),
        1,
        "create_author set has one author event"
    );
    assert_eq!(author_events[0]["operation"].as_str(), Some("create"));

    // Unknown id returns null.
    let unknown_query = format!(r#"{{ eventSet(id: "{}") {{ id }} }}"#, uuid::Uuid::new_v4());
    let (_, response) = graphql_request(&unknown_query, Some(&token)).await?;
    assert!(
        response["data"]["eventSet"].is_null(),
        "unknown event set id should return null"
    );

    // User isolation: a second user cannot see the first user's event set.
    let (_other_user_id, other_token) = create_test_user().await?;
    let isolation_query = format!(r#"{{ eventSet(id: "{}") {{ id }} }}"#, create_book_set_id);
    let (_, response) = graphql_request(&isolation_query, Some(&other_token)).await?;
    assert!(
        response["data"]["eventSet"].is_null(),
        "other user must not see another user's event set"
    );
    let (_, response) = graphql_request(r#"{ eventSets { id } }"#, Some(&other_token)).await?;
    assert!(
        response["data"]["eventSets"]
            .as_array()
            .context("eventSets should be an array")?
            .is_empty(),
        "a fresh user should have no event sets"
    );

    // Cleanup.
    delete_test_book(&book_id, &token).await?;
    delete_test_author(&author_id, &token).await?;

    Ok(())
}
