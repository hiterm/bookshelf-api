// E2E tests that run against a real Postgres instance.
// Enabled with `--features test-with-database`.
#![cfg(feature = "test-with-database")]

use reqwest::Client;
use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;

fn spawn_server() -> Child {
    // Start the application binary via `cargo run`.
    // The test process inherits env vars (PORT, DATABASE_URL, ALLOWED_ORIGINS).
    Command::new("cargo")
        .args(&["run", "--bin", "bookshelf-api"])
        .spawn()
        .expect("failed to spawn app via cargo run")
}

async fn wait_for_server(url: &str) {
    let client = Client::new();
    // Wait up to ~30 seconds for the server to become ready. Building the binary
    // and starting it inside tests can take a while, so allow a longer timeout.
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
    // Expect the caller (CI/local) to set these env vars
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("http://127.0.0.1:{}/health", port);

    let mut child = spawn_server();

    wait_for_server(&addr).await;

    let client = Client::new();
    let res = client.get(&addr).send().await.expect("request failed");
    assert!(res.status().is_success());

    // cleanup
    let _ = child.kill();
}
