use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

const TEST_AUDIENCE: &str = "test-audience";
const TEST_ISSUER: &str = "https://test-issuer.local/";
const TEST_KID: &str = "test-key-id";

// Embedded test private key (RSA-2048, for testing only — never use in production)
const TEST_PRIVATE_KEY_PEM: &str = include_str!("../../testdata/test_private_key.pem");

#[derive(Debug, Serialize)]
struct TestClaims {
    sub: String,
    aud: String,
    iss: String,
    exp: u64,
}

/// Generate a test JWT token signed with the test RSA private key.
///
/// The token is valid for 1 hour and uses the audience/issuer values that
/// match the test server configuration (JWT_AUDIENCE=test-audience,
/// JWT_DOMAIN=test-issuer.local).
pub fn generate_test_token(user_id: &str) -> String {
    let exp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;

    let claims = TestClaims {
        sub: user_id.to_string(),
        aud: TEST_AUDIENCE.to_string(),
        iss: TEST_ISSUER.to_string(),
        exp,
    };

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(TEST_KID.to_string());

    let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes())
        .expect("Failed to load test private key");

    encode(&header, &claims, &key).expect("Failed to generate test JWT")
}
