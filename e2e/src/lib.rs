use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use reqwest::{Client, StatusCode};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

const TEST_AUDIENCE: &str = "test-audience";
const TEST_ISSUER: &str = "https://test-issuer.local/";
const TEST_KID: &str = "test-key-id";

// Embedded test private key (RSA-2048, for testing only — never use in production)
const TEST_PRIVATE_KEY_PEM: &str = include_str!("../../testdata/test_private_key.pem");

#[derive(Debug, Serialize)]
struct TestClaims {
    sub: String,
    aud: String,
    iss: String,
    exp: u64,
}

/// Generate a test JWT token signed with the test RSA private key.
///
/// The token is valid for 1 hour and uses the audience/issuer values that
/// match the test server configuration (JWT_AUDIENCE=test-audience,
/// JWT_DOMAIN=test-issuer.local).
pub fn generate_test_token(user_id: &str) -> Result<String> {
    let exp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() + 3600;

    let claims = TestClaims {
        sub: user_id.to_string(),
        aud: TEST_AUDIENCE.to_string(),
        iss: TEST_ISSUER.to_string(),
        exp,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());

    let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes())?;
    let token = encode(&header, &claims, &key)?;
    Ok(token)
}

// Shared E2E test helpers.
pub fn get_server_url() -> Result<String> {
    let url = std::env::var("TEST_SERVER_URL")
        .context("TEST_SERVER_URL environment variable must be set. Please set it to the external server URL (e.g., http://localhost:8080)")?;
    Ok(url.trim_end_matches('/').to_owned())
}

pub fn get_graphql_url() -> Result<String> {
    let base_url = get_server_url()?;
    Ok(format!("{}/graphql", base_url))
}

pub async fn graphql_request(query: &str, token: Option<&str>) -> Result<(u16, serde_json::Value)> {
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

pub async fn delete_test_author(author_id: &str, token: &str) -> Result<()> {
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

pub async fn delete_test_book(book_id: &str, token: &str) -> Result<()> {
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

pub async fn ensure_user_registered(token: &str) -> Result<()> {
    let query = r#"mutation { registerUser { id } }"#;
    let (status, response) = graphql_request(query, Some(token)).await?;

    // Always check for GraphQL errors, regardless of HTTP status
    if let Some(errors) = response.get("errors") {
        let is_conflict = errors.as_array().is_some_and(|errors| {
            errors
                .iter()
                .any(|error| graphql_error_code(error) == Some("CONFLICT"))
        });

        if !is_conflict {
            anyhow::bail!("registerUser failed with error: {}", errors);
        }
        // A conflict means user already exists - that's OK.
        return Ok(());
    }

    // If no GraphQL errors, HTTP status should be 200
    if status != 200 {
        anyhow::bail!("registerUser failed with status {}", status);
    }

    Ok(())
}

/// Creates a fresh, registered user and returns its `(user_id, token)`.
pub async fn create_test_user() -> Result<(String, String)> {
    let user_id = uuid::Uuid::new_v4().to_string();
    let token = generate_test_token(&user_id)?;
    ensure_user_registered(&token).await?;
    Ok((user_id, token))
}

pub async fn create_test_author(name: &str, token: &str) -> Result<String> {
    let query = format!(
        r#"mutation {{ createAuthor(authorData: {{ name: "{}" }}) {{ id }} }}"#,
        name
    );
    let (_, response) = graphql_request(&query, Some(token)).await?;
    bail_on_graphql_errors(&response, "createAuthor")?;
    let id = response["data"]["createAuthor"]["id"]
        .as_str()
        .context("createAuthor id should be a string")?
        .to_owned();
    Ok(id)
}

pub async fn create_test_book(title: &str, author_id: &str, token: &str) -> Result<String> {
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
    bail_on_graphql_errors(&response, "createBook")?;
    let id = response["data"]["createBook"]["id"]
        .as_str()
        .context("createBook id should be a string")?
        .to_owned();
    Ok(id)
}

pub async fn update_test_book(
    book_id: &str,
    title: &str,
    author_id: &str,
    isbn: &str,
    read: bool,
    owned: bool,
    priority: i32,
    format: &str,
    store: &str,
    token: &str,
) -> Result<serde_json::Value> {
    let query = format!(
        r#"
        mutation {{
            updateBook(bookData: {{
                id: "{}"
                title: "{}"
                authorIds: ["{}"]
                isbn: "{}"
                read: {}
                owned: {}
                priority: {}
                format: {}
                store: {}
            }}) {{
                id
                title
                authors {{
                    id
                    name
                }}
                isbn
                read
                owned
                priority
                format
                store
            }}
        }}
        "#,
        book_id, title, author_id, isbn, read, owned, priority, format, store
    );
    let (_, response) = graphql_request(&query, Some(token)).await?;
    bail_on_graphql_errors(&response, "updateBook")?;
    Ok(response["data"]["updateBook"].clone())
}

pub async fn find_event_set_by_operation(
    operation: &str,
    token: &str,
) -> Result<serde_json::Value> {
    let (_, response) = graphql_request(r#"{ eventSets { id operation } }"#, Some(token)).await?;
    bail_on_graphql_errors(&response, "eventSets")?;
    let event_set_id = response["data"]["eventSets"]
        .as_array()
        .context("eventSets should be an array")?
        .iter()
        .find(|set| set["operation"].as_str() == Some(operation))
        .and_then(|set| set["id"].as_str())
        .context("matching event set should exist")?;

    let event_set_query = format!(
        r#"{{ eventSet(id: "{}") {{
            operation
            bookEvents {{ bookId operation }}
            authorEvents {{ name operation }}
        }} }}"#,
        event_set_id
    );
    let (_, response) = graphql_request(&event_set_query, Some(token)).await?;
    bail_on_graphql_errors(&response, "eventSet")?;
    let event_set = response["data"]["eventSet"].clone();
    if event_set.is_null() {
        anyhow::bail!("matching event set should be found");
    }
    Ok(event_set)
}

pub fn assert_graphql_errors(response: &serde_json::Value, context: &str) {
    assert!(
        response.get("errors").is_some(),
        "{context} should return GraphQL errors: {response:?}"
    );
}

pub fn assert_no_graphql_errors(response: &serde_json::Value, context: &str) {
    assert!(
        response.get("errors").is_none(),
        "{context} should not return GraphQL errors: {:?}",
        response.get("errors")
    );
}

pub fn bail_on_graphql_errors(response: &serde_json::Value, context: &str) -> Result<()> {
    if let Some(errors) = response.get("errors") {
        anyhow::bail!("{context} returned GraphQL errors: {errors}");
    }
    Ok(())
}

fn graphql_error_code(error: &serde_json::Value) -> Option<&str> {
    error
        .get("extensions")
        .and_then(|extensions| extensions.get("code"))
        .and_then(|code| code.as_str())
}
