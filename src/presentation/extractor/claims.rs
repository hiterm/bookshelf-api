use crate::common::http::build_http_client;
use crate::presentation::app_state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::{Json, RequestPartsExt};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use derive_more::Display;
use http::{StatusCode, Uri};
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{AlgorithmParameters, Jwk, JwkSet},
};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashSet, sync::Arc};

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub(crate) audience: String,
    pub(crate) domain: String,
}

impl Default for JwtConfig {
    fn default() -> Self {
        envy::prefixed("JWT_")
            .from_env()
            .expect("Provide missing environment variables for JWT (JWT_AUDIENCE, JWT_DOMAIN)")
    }
}

#[derive(Debug, Display)]
pub enum ClientError {
    #[display("authentication")]
    Authentication,
    #[display("decode")]
    Decode(jsonwebtoken::errors::Error),
    #[display("jwks_fetch")]
    JwksFetch(String),
    #[display("not_found")]
    NotFound(String),
    #[display("unsupported_algorithm")]
    UnsupportedAlgortithm(AlgorithmParameters),
}

impl IntoResponse for ClientError {
    fn into_response(self) -> Response {
        let (status, error, error_description, message) = match self {
            Self::Authentication => (
                StatusCode::UNAUTHORIZED,
                None,
                None,
                "Requires authentication".to_string(),
            ),
            Self::Decode(_) => (
                StatusCode::UNAUTHORIZED,
                Some("invalid_token".to_string()),
                Some(
                    "Authorization header value must follow this format: Bearer access-token"
                        .to_string(),
                ),
                "Bad credentials".to_string(),
            ),
            Self::JwksFetch(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Some("server_error".to_string()),
                Some(msg),
                "Service temporarily unavailable".to_string(),
            ),
            Self::NotFound(msg) => (
                StatusCode::UNAUTHORIZED,
                Some("invalid_token".to_string()),
                Some(msg),
                "Bad credentials".to_string(),
            ),
            Self::UnsupportedAlgortithm(alg) => (
                StatusCode::UNAUTHORIZED,
                Some("invalid_token".to_string()),
                Some(format!(
                    "Unsupported encryption algortithm expected RSA got {:?}",
                    alg
                )),
                "Bad credentials".to_string(),
            ),
        };
        let body = Json(json!({
            "error": error,
            "error_description": error_description,
            "message": message
        }));
        (status, body).into_response()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub _permissions: Option<HashSet<String>>,
}

impl FromRequestParts<Arc<AppState>> for Claims {
    type Rejection = ClientError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let config = state.jwt_config.clone();
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| ClientError::Authentication)?;
        let token = bearer.token();

        let header = decode_header(token).map_err(ClientError::Decode)?;
        let kid = header
            .kid
            .ok_or_else(|| ClientError::NotFound("kid not found in token header".to_string()))?;
        let domain = config.domain.as_str();
        let jwks_url = std::env::var("JWKS_URL")
            .unwrap_or_else(|_| format!("https://{}/.well-known/jwks.json", domain));
        validate_jwks_url(&jwks_url)?;

        // キャッシュから取得（miss時は fetch_jwks を1回だけ実行）
        let jwks = state
            .jwks_cache
            .try_get_with(jwks_url.clone(), fetch_jwks(&jwks_url))
            .await
            .map_err(|e| ClientError::JwksFetch(format!("JWKS fetch failed: {e}")))?;

        // kid が見つかれば検証して返す
        if let Some(jwk) = jwks.find(&kid) {
            return validate_claims(jwk, token, domain, &config.audience);
        }

        // kid miss: キャッシュを無効化して1回だけ再フェッチ（鍵ローテーション対応）
        state.jwks_cache.invalidate(&jwks_url).await;
        let jwks = fetch_jwks(&jwks_url).await?;
        state.jwks_cache.insert(jwks_url, jwks.clone()).await;

        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| ClientError::NotFound("No JWK found for kid".to_string()))?;
        validate_claims(jwk, token, domain, &config.audience)
    }
}

fn validate_claims(
    jwk: &Jwk,
    token: &str,
    domain: &str,
    audience: &str,
) -> Result<Claims, ClientError> {
    match &jwk.algorithm {
        AlgorithmParameters::RSA(rsa) => {
            let mut validation = Validation::new(Algorithm::RS256);
            validation.set_audience(&[audience]);
            validation.set_issuer(&[Uri::builder()
                .scheme("https")
                .authority(domain)
                .path_and_query("/")
                .build()
                .unwrap()]);
            let key =
                DecodingKey::from_rsa_components(&rsa.n, &rsa.e).map_err(ClientError::Decode)?;
            let token_data =
                decode::<Claims>(token, &key, &validation).map_err(ClientError::Decode)?;
            Ok(token_data.claims)
        }
        algorithm => Err(ClientError::UnsupportedAlgortithm(algorithm.clone())),
    }
}

/// Validates that the JWKS URL is safe to fetch: `http://` is only permitted
/// for loopback addresses.
fn validate_jwks_url(url: &str) -> Result<(), ClientError> {
    let uri: Uri = url
        .parse()
        .map_err(|_| ClientError::JwksFetch(format!("invalid JWKS_URL: {url}")))?;
    if uri.scheme_str() == Some("http") {
        let host = uri.host().unwrap_or("");
        if host != "localhost" && host != "127.0.0.1" && host != "::1" {
            return Err(ClientError::JwksFetch(
                "http:// JWKS_URL is only permitted for loopback addresses".to_string(),
            ));
        }
    }
    Ok(())
}

async fn fetch_jwks(url: &str) -> Result<Arc<JwkSet>, ClientError> {
    let client = build_http_client()
        .map_err(|e| ClientError::JwksFetch(format!("failed to build HTTP client: {e}")))?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| ClientError::JwksFetch(format!("request failed: {e}")))?;
    let jwks = response
        .json::<JwkSet>()
        .await
        .map_err(|e| ClientError::JwksFetch(format!("invalid JWKS response: {e}")))?;
    Ok(Arc::new(jwks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
    use serde::Serialize;
    use std::time::{SystemTime, UNIX_EPOCH};

    // ── Test fixtures ──────────────────────────────────────────────────────────

    const TEST_PRIVATE_KEY_PEM: &str =
        include_str!("../../../testdata/test_private_key.pem");
    const TEST_JWKS_JSON: &str = include_str!("../../../testdata/test_jwks.json");

    const TEST_KID: &str = "test-key-id";
    const TEST_AUDIENCE: &str = "test-audience";
    const TEST_DOMAIN: &str = "test-issuer.local";

    #[derive(Debug, Serialize)]
    struct TestClaims {
        sub: String,
        aud: String,
        iss: String,
        exp: u64,
    }

    /// Generates a valid RS256 JWT signed with the test RSA private key.
    fn generate_test_token(sub: &str) -> String {
        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        let claims = TestClaims {
            sub: sub.to_string(),
            aud: TEST_AUDIENCE.to_string(),
            iss: format!("https://{}/", TEST_DOMAIN),
            exp,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(TEST_KID.to_string());
        let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap();
        encode(&header, &claims, &key).unwrap()
    }

    /// Loads the test JWK Set from testdata.
    fn test_jwks() -> JwkSet {
        serde_json::from_str(TEST_JWKS_JSON).unwrap()
    }

    // ── validate_jwks_url ─────────────────────────────────────────────────────

    #[test]
    fn validate_jwks_url_accepts_https_url() {
        // Given: a standard HTTPS JWKS URL
        // When/Then: should succeed
        assert!(validate_jwks_url("https://example.com/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn validate_jwks_url_accepts_http_localhost() {
        // Given: an HTTP URL pointing to localhost
        assert!(validate_jwks_url("http://localhost/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn validate_jwks_url_accepts_http_127_0_0_1() {
        // Given: an HTTP URL pointing to 127.0.0.1 loopback
        assert!(validate_jwks_url("http://127.0.0.1/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn validate_jwks_url_accepts_http_ipv6_loopback() {
        // Given: an HTTP URL pointing to ::1 (IPv6 loopback)
        assert!(validate_jwks_url("http://[::1]/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn validate_jwks_url_rejects_http_external_host() {
        // Given: an HTTP URL pointing to a non-loopback host
        // When/Then: should fail because http:// is not allowed for external hosts
        let result = validate_jwks_url("http://example.com/.well-known/jwks.json");
        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::JwksFetch(msg) => {
                assert!(msg.contains("loopback"), "Expected loopback error, got: {msg}");
            }
            e => panic!("Expected JwksFetch error, got: {e}"),
        }
    }

    #[test]
    fn validate_jwks_url_rejects_http_private_ip() {
        // Given: an HTTP URL with a private (non-loopback) IP address
        let result = validate_jwks_url("http://192.168.1.1/.well-known/jwks.json");
        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::JwksFetch(msg) => {
                assert!(msg.contains("loopback"), "Expected loopback error, got: {msg}");
            }
            e => panic!("Expected JwksFetch error, got: {e}"),
        }
    }

    #[test]
    fn validate_jwks_url_rejects_invalid_url() {
        // Given: a string that cannot be parsed as a URI
        let result = validate_jwks_url("not a valid url !!!");
        assert!(result.is_err());
        match result.unwrap_err() {
            ClientError::JwksFetch(msg) => {
                assert!(
                    msg.contains("invalid JWKS_URL"),
                    "Expected invalid URL error, got: {msg}"
                );
            }
            e => panic!("Expected JwksFetch error, got: {e}"),
        }
    }

    #[test]
    fn validate_jwks_url_accepts_https_with_port() {
        // Given: an HTTPS URL that includes a non-standard port
        assert!(validate_jwks_url("https://auth.example.com:8443/jwks").is_ok());
    }

    // ── validate_claims ───────────────────────────────────────────────────────

    #[test]
    fn validate_claims_accepts_valid_rsa_token() {
        // Given: a valid RSA JWK and a correctly signed JWT
        let jwks = test_jwks();
        let jwk = jwks.find(TEST_KID).unwrap();
        let token = generate_test_token("auth0|user123");

        // When: validating claims
        let result = validate_claims(jwk, &token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should return the extracted claims
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result.unwrap_err());
        let claims = result.unwrap();
        assert_eq!(claims.sub, "auth0|user123");
    }

    #[test]
    fn validate_claims_rejects_tampered_token() {
        // Given: a valid JWT that has been tampered with
        let jwks = test_jwks();
        let jwk = jwks.find(TEST_KID).unwrap();
        let mut token = generate_test_token("auth0|user123");
        // Flip one character in the signature (last segment)
        let last_char = token.pop().unwrap();
        let replacement = if last_char == 'a' { 'b' } else { 'a' };
        token.push(replacement);

        // When: validating claims
        let result = validate_claims(jwk, &token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should fail with a decode error
        assert!(matches!(result, Err(ClientError::Decode(_))));
    }

    #[test]
    fn validate_claims_rejects_wrong_audience() {
        // Given: a valid JWT but the wrong expected audience
        let jwks = test_jwks();
        let jwk = jwks.find(TEST_KID).unwrap();
        let token = generate_test_token("auth0|user123");

        // When: validating with a different audience
        let result = validate_claims(jwk, &token, TEST_DOMAIN, "wrong-audience");

        // Then: should fail
        assert!(matches!(result, Err(ClientError::Decode(_))));
    }

    #[test]
    fn validate_claims_rejects_wrong_domain_issuer() {
        // Given: a valid JWT but the wrong expected domain/issuer
        let jwks = test_jwks();
        let jwk = jwks.find(TEST_KID).unwrap();
        let token = generate_test_token("auth0|user123");

        // When: validating with a different domain (different issuer)
        let result = validate_claims(jwk, &token, "wrong-domain.example.com", TEST_AUDIENCE);

        // Then: should fail
        assert!(matches!(result, Err(ClientError::Decode(_))));
    }

    #[test]
    fn validate_claims_rejects_non_rsa_jwk() {
        // Given: a JWK using an EC (non-RSA) algorithm
        let ec_jwk_json = serde_json::json!({
            "kty": "EC",
            "alg": "ES256",
            "use": "sig",
            "kid": "ec-key",
            "crv": "P-256",
            "x": "f83OJ3D2xF1Bg8vub9tLe1gHMzV76e8Tus9uPHvRVEU",
            "y": "x_FEzRu9m36HLN_tue659LNpXW6pCyStikYjKIWI5a0"
        });
        let ec_jwk: Jwk = serde_json::from_value(ec_jwk_json).unwrap();
        let token = generate_test_token("auth0|user123");

        // When: validating with an EC JWK
        let result = validate_claims(&ec_jwk, &token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should return UnsupportedAlgortithm error
        assert!(matches!(result, Err(ClientError::UnsupportedAlgortithm(_))));
    }

    #[test]
    fn validate_claims_rejects_malformed_token_string() {
        // Given: a JWK and a completely malformed token string
        let jwks = test_jwks();
        let jwk = jwks.find(TEST_KID).unwrap();

        // When: validating with a non-JWT string
        let result = validate_claims(jwk, "not.a.jwt", TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should fail with a decode error
        assert!(matches!(result, Err(ClientError::Decode(_))));
    }

    #[test]
    fn validate_claims_preserves_sub_value() {
        // Given: a token generated for a user with a specific sub value
        let jwks = test_jwks();
        let jwk = jwks.find(TEST_KID).unwrap();
        let expected_sub = "google-oauth2|9876543210";
        let token = generate_test_token(expected_sub);

        // When: validating claims
        let result = validate_claims(jwk, &token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: the claims should carry the original sub
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sub, expected_sub);
    }
}