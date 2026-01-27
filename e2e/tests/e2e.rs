// E2E tests that run against a real Postgres instance.
// This lives in a dedicated crate inside the workspace so it can be run independently.

#![cfg(test)]

use reqwest::Client;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

fn spawn_server() -> Child {
    // Start the application binary from the workspace by specifying the package
    Command::new("cargo")
        .args(&["run", "-p", "bookshelf-api", "--bin", "bookshelf-api"])
        .spawn()
        .expect("failed to spawn app via cargo run")
}

async fn wait_for_server(url: &str) {
    let client = Client::new();
    for _ in 0..150 {
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_millis(200)).await;
    }
    panic!("server did not become ready");
}

#[tokio::test]
async fn e2e_health_check() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("http://127.0.0.1:{}/health", port);

    let mut child = spawn_server();

    wait_for_server(&addr).await;

    let client = Client::new();
    let res = client.get(&addr).send().await.expect("request failed");
    assert!(res.status().is_success());

    let _ = child.kill();
}
