use axum::{routing::get, Router};

const TEST_JWKS: &str = include_str!("../../../testdata/test_jwks.json");

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("JWKS_SERVER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(9999);

    let app = Router::new()
        .route("/.well-known/jwks.json", get(jwks_handler))
        .route("/health", get(|| async { "OK" }));

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    eprintln!("JWKS server listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}

async fn jwks_handler() -> &'static str {
    TEST_JWKS
}
