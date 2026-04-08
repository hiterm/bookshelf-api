// Regression check for the CA-certificate fix (PR #187).
// Verifies that reqwest can establish an HTTPS connection using the system trust store.
// Run inside the production container image to confirm ca-certificates is installed.
#[tokio::main]
async fn main() {
    reqwest::get("https://example.com")
        .await
        .expect("HTTPS check failed: CA certificates may not be installed");
    println!("TLS OK");
}
