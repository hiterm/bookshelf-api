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

        // キャッシュから取得（miss時は fetch_jwks を1回だけ実行）
        let jwks = state
            .jwks_cache
            .try_get_with((), fetch_jwks(domain))
            .await
            .map_err(|e| ClientError::JwksFetch(format!("JWKS fetch failed: {e}")))?;

        // kid が見つかれば検証して返す
        if let Some(jwk) = jwks.find(&kid) {
            return validate_claims(jwk, token, domain, &config.audience);
        }

        // kid miss: キャッシュを無効化して1回だけ再フェッチ（鍵ローテーション対応）
        // try_get_with を使うことで並行リクエストのフェッチを1回に集約する
        state.jwks_cache.invalidate(&()).await;
        let jwks = state
            .jwks_cache
            .try_get_with((), fetch_jwks(domain))
            .await
            .map_err(|e| ClientError::JwksFetch(format!("JWKS fetch failed: {e}")))?;

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
        let is_loopback = host == "localhost"
            || host
                .parse::<std::net::IpAddr>()
                .map(|ip| ip.is_loopback())
                .unwrap_or(false);
        if !is_loopback {
            return Err(ClientError::JwksFetch(
                "http:// JWKS_URL is only permitted for loopback addresses".to_string(),
            ));
        }
    }
    Ok(())
}

async fn fetch_jwks(domain: &str) -> Result<Arc<JwkSet>, ClientError> {
    let url = std::env::var("JWKS_URL")
        .unwrap_or_else(|_| format!("https://{}/.well-known/jwks.json", domain));
    validate_jwks_url(&url)?;
    let client = build_http_client()
        .map_err(|e| ClientError::JwksFetch(format!("failed to build HTTP client: {e}")))?;
    let response = client
        .get(&url)
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

    fn make_valid_token(sub: &str, aud: &str, iss: &str, exp_offset_secs: i64) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let exp = if exp_offset_secs >= 0 {
            now + exp_offset_secs as u64
        } else {
            now.saturating_sub((-exp_offset_secs) as u64)
        };
        let claims = TestClaims {
            sub: sub.to_string(),
            aud: aud.to_string(),
            iss: iss.to_string(),
            exp,
        };
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(TEST_KID.to_string());
        let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY_PEM.as_bytes()).unwrap();
        encode(&header, &claims, &key).unwrap()
    }

    fn load_test_jwks() -> JwkSet {
        serde_json::from_str(TEST_JWKS_JSON).unwrap()
    }

    fn rsa_jwk() -> Jwk {
        load_test_jwks().keys.into_iter().next().unwrap()
    }

    fn ec_jwk() -> Jwk {
        // Valid-format P-256 EC JWK (RFC 7517 Appendix A example key)
        serde_json::from_str(
            r#"{
                "kty": "EC",
                "crv": "P-256",
                "x": "f83OJ3D2xF1Bg8vub9tLe1gHMzV76e8Tus9uPHvRVEU",
                "y": "x_FEzRu9m36HLN_tue659LNpXW6pCyStikYjKIWI5a0"
            }"#,
        )
        .unwrap()
    }

    fn symmetric_jwk() -> Jwk {
        serde_json::from_str(
            r#"{
                "kty": "oct",
                "k": "dGhlIHNlY3JldA"
            }"#,
        )
        .unwrap()
    }

    // ============================================================
    // validate_jwks_url tests
    // ============================================================

    #[test]
    fn https_url_is_always_valid() {
        assert!(validate_jwks_url("https://example.com/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn https_url_with_port_is_valid() {
        assert!(validate_jwks_url("https://example.com:443/jwks.json").is_ok());
    }

    #[test]
    fn http_localhost_is_valid() {
        assert!(validate_jwks_url("http://localhost/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_localhost_with_port_is_valid() {
        assert!(validate_jwks_url("http://localhost:8080/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_loopback_ipv4_127_0_0_1_is_valid() {
        assert!(validate_jwks_url("http://127.0.0.1/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_loopback_ipv4_127_0_0_2_is_valid() {
        // The entire 127.x.x.x range is loopback per is_loopback(); this was not
        // handled by the old hardcoded-string check.
        assert!(validate_jwks_url("http://127.0.0.2/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_loopback_ipv6_bracket_notation_is_valid() {
        assert!(validate_jwks_url("http://[::1]/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_non_loopback_ip_is_rejected() {
        let result = validate_jwks_url("http://192.168.1.1/.well-known/jwks.json");
        assert!(matches!(result, Err(ClientError::JwksFetch(_))));
        if let Err(ClientError::JwksFetch(msg)) = result {
            assert!(msg.contains("loopback"));
        }
    }

    #[test]
    fn http_external_domain_is_rejected() {
        let result = validate_jwks_url("http://evil.com/.well-known/jwks.json");
        assert!(matches!(result, Err(ClientError::JwksFetch(_))));
        if let Err(ClientError::JwksFetch(msg)) = result {
            assert!(msg.contains("loopback"));
        }
    }

    #[test]
    fn http_public_ip_with_port_is_rejected() {
        let result = validate_jwks_url("http://10.0.0.1:9000/jwks.json");
        assert!(matches!(result, Err(ClientError::JwksFetch(_))));
    }

    #[test]
    fn invalid_url_is_rejected() {
        let result = validate_jwks_url("://not-valid");
        assert!(matches!(result, Err(ClientError::JwksFetch(_))));
        if let Err(ClientError::JwksFetch(msg)) = result {
            assert!(msg.contains("invalid JWKS_URL"));
        }
    }

    #[test]
    fn empty_url_has_no_http_scheme_so_passes_loopback_guard() {
        // An empty string parses as a relative URI with no scheme, so the
        // http-scheme guard does not trigger and validate_jwks_url returns Ok.
        // This documents the boundary: callers must supply a full URL.
        assert!(validate_jwks_url("").is_ok());
    }

    #[test]
    fn http_url_with_no_path_to_loopback_is_valid() {
        assert!(validate_jwks_url("http://127.0.0.1").is_ok());
    }

    // ============================================================
    // validate_claims tests
    // ============================================================

    #[test]
    fn validate_claims_with_ec_jwk_returns_unsupported_algorithm_error() {
        // Given: an EC JWK (not RSA)
        let jwk = ec_jwk();
        let token = "dummy.token.value";

        // When: validating claims
        let result = validate_claims(&jwk, token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should return UnsupportedAlgortithm error
        assert!(
            matches!(result, Err(ClientError::UnsupportedAlgortithm(_))),
            "Expected UnsupportedAlgortithm, got {:?}",
            result
        );
    }

    #[test]
    fn validate_claims_with_symmetric_jwk_returns_unsupported_algorithm_error() {
        // Given: a symmetric (oct) JWK
        let jwk = symmetric_jwk();
        let token = "dummy.token.value";

        // When: validating claims
        let result = validate_claims(&jwk, token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should return UnsupportedAlgortithm error
        assert!(
            matches!(result, Err(ClientError::UnsupportedAlgortithm(_))),
            "Expected UnsupportedAlgortithm, got {:?}",
            result
        );
    }

    #[test]
    fn validate_claims_with_rsa_jwk_and_invalid_token_returns_decode_error() {
        // Given: a valid RSA JWK but an invalid (malformed) token string
        let jwk = rsa_jwk();
        let token = "not.a.jwt";

        // When: validating claims
        let result = validate_claims(&jwk, token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should return a Decode error
        assert!(
            matches!(result, Err(ClientError::Decode(_))),
            "Expected Decode error, got {:?}",
            result
        );
    }

    #[test]
    fn validate_claims_with_rsa_jwk_and_invalid_n_returns_decode_error() {
        // Given: RSA JWK with an invalid modulus (base64url-looking but cryptographically invalid)
        let jwk: Jwk = serde_json::from_str(
            r#"{
                "kty": "RSA",
                "n": "AAAA",
                "e": "AQAB"
            }"#,
        )
        .unwrap();
        let token = "header.payload.signature";

        // When: validating claims
        let result = validate_claims(&jwk, token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should return a Decode error (either from key construction or token decoding)
        assert!(
            matches!(result, Err(ClientError::Decode(_))),
            "Expected Decode error, got {:?}",
            result
        );
    }

    #[test]
    fn validate_claims_with_valid_rsa_jwk_and_valid_token_returns_claims() {
        // Given: the real RSA public JWK and a freshly minted valid JWT
        let jwk = rsa_jwk();
        let issuer = format!("https://{}/", TEST_DOMAIN);
        let token = make_valid_token("auth0|user1", TEST_AUDIENCE, &issuer, 3600);

        // When: validating claims
        let result = validate_claims(&jwk, &token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should succeed and return Claims with the correct sub
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
        let claims = result.unwrap();
        assert_eq!(claims.sub, "auth0|user1");
    }

    #[test]
    fn validate_claims_with_wrong_audience_returns_decode_error() {
        // Given: a valid JWT signed for "test-audience" but validated against "other-audience"
        let jwk = rsa_jwk();
        let issuer = format!("https://{}/", TEST_DOMAIN);
        let token = make_valid_token("auth0|user2", TEST_AUDIENCE, &issuer, 3600);

        // When: validating with wrong audience
        let result = validate_claims(&jwk, &token, TEST_DOMAIN, "other-audience");

        // Then: should return a Decode error
        assert!(
            matches!(result, Err(ClientError::Decode(_))),
            "Expected Decode error for wrong audience, got {:?}",
            result
        );
    }

    #[test]
    fn validate_claims_with_wrong_issuer_returns_decode_error() {
        // Given: a valid JWT signed with issuer "https://test-issuer.local/" but
        //        validated against domain "different-domain.local"
        let jwk = rsa_jwk();
        let issuer = format!("https://{}/", TEST_DOMAIN);
        let token = make_valid_token("auth0|user3", TEST_AUDIENCE, &issuer, 3600);

        // When: validating with wrong domain
        let result = validate_claims(&jwk, &token, "different-domain.local", TEST_AUDIENCE);

        // Then: should return a Decode error
        assert!(
            matches!(result, Err(ClientError::Decode(_))),
            "Expected Decode error for wrong issuer, got {:?}",
            result
        );
    }

    #[test]
    fn validate_claims_with_expired_token_returns_decode_error() {
        // Given: a JWT that expired 10 seconds ago
        let jwk = rsa_jwk();
        let issuer = format!("https://{}/", TEST_DOMAIN);
        let token = make_valid_token("auth0|user4", TEST_AUDIENCE, &issuer, -10);

        // When: validating the expired token
        let result = validate_claims(&jwk, &token, TEST_DOMAIN, TEST_AUDIENCE);

        // Then: should return a Decode error (ExpiredSignature)
        assert!(
            matches!(result, Err(ClientError::Decode(_))),
            "Expected Decode error for expired token, got {:?}",
            result
        );
    }
}