// E2E tests that run against a real Postgres instance.
// This lives in a dedicated crate inside the workspace so it can be run independently.

#![cfg(test)]

use reqwest::{Client, StatusCode};
use serde::Deserialize;

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
