// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::{generate_test_token, get_server_url};
use reqwest::{Client, StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MeResponse {
    id: String,
}

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
