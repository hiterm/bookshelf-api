use std::time::Duration;

pub fn build_http_client() -> reqwest::Result<reqwest::Client> {
    reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
}
