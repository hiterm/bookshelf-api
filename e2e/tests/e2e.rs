// E2E tests that run against a real Postgres instance.
// This lives in a dedicated crate inside the workspace so it can be run independently.

#![cfg(test)]

use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::time::Duration;
use tokio::process::{Child, Command};
use tokio::time::sleep;

/// Response payload for GET /me endpoint
#[derive(Debug, Deserialize)]
struct MeResponse {
    id: String,
}

fn spawn_server() -> Child {
    // Start the application binary from the workspace by specifying the package
    // Use tokio::process::Command and enable kill_on_drop so the child is
    // killed automatically if the test task panics or times out.
    Command::new("cargo")
        .args(&["run", "-p", "bookshelf-api", "--bin", "bookshelf-api"])
        .kill_on_drop(true)
        .spawn()
        .expect("failed to spawn app via cargo run")
}

async fn wait_for_server(url: &str) {
    let client = Client::new();
    for _ in 0..300 {
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    panic!("server did not become ready");
}

#[tokio::test]
async fn e2e_health_check() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("http://127.0.0.1:{}/health", port);

    let _child = spawn_server();

    wait_for_server(&addr).await;

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let health_addr = format!("http://127.0.0.1:{}/health", port);
    let me_addr = format!("http://127.0.0.1:{}/me", port);

    let _child = spawn_server();
    wait_for_server(&health_addr).await;

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let health_addr = format!("http://127.0.0.1:{}/health", port);
    let me_addr = format!("http://127.0.0.1:{}/me", port);

    // Get valid token from environment
    let token = std::env::var("TEST_JWT_TOKEN")
        .expect("TEST_JWT_TOKEN environment variable must be set for this test");

    let _child = spawn_server();
    wait_for_server(&health_addr).await;

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let health_addr = format!("http://127.0.0.1:{}/health", port);
    let me_addr = format!("http://127.0.0.1:{}/me", port);

    let _child = spawn_server();
    wait_for_server(&health_addr).await;

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let health_addr = format!("http://127.0.0.1:{}/health", port);
    let me_addr = format!("http://127.0.0.1:{}/me", port);

    let _child = spawn_server();
    wait_for_server(&health_addr).await;

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
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let health_addr = format!("http://127.0.0.1:{}/health", port);
    let me_addr = format!("http://127.0.0.1:{}/me", port);

    // Get valid token from environment
    let token = std::env::var("TEST_JWT_TOKEN")
        .expect("TEST_JWT_TOKEN environment variable must be set for this test");

    let _child = spawn_server();
    wait_for_server(&health_addr).await;

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
