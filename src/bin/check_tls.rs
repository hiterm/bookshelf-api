// Regression check for the CA-certificate fix (PR #187).
// Verifies that the HTTP client used in production can establish an HTTPS
// connection using the system trust store.
// Run inside the production container image to confirm ca-certificates is installed.
use bookshelf_api::common::http::build_http_client;

#[tokio::main]
async fn main() {
    build_http_client()
        .expect("failed to build HTTP client")
        .get("https://example.com")
        .send()
        .await
        .expect("HTTPS check failed: CA certificates may not be installed");
    println!("TLS OK");
}
