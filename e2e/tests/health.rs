// E2E tests that run against a real Postgres instance.

#![cfg(test)]

use anyhow::{Context, Result};
use bookshelf_e2e::get_server_url;
use reqwest::Client;

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
