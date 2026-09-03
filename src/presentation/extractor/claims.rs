use crate::common::http::build_http_client;
use crate::presentation::app_state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::{Json, RequestPartsExt};
use axum_extra::TypedHeader;
use axum_extra::headers::Authorization;
use axum_extra::headers::authorization::Bearer;
use http::{StatusCode, Uri};
use jsonwebtoken::{
    Algorithm, DecodingKey, Validation, decode, decode_header,
    jwk::{AlgorithmParameters, Jwk, JwkSet},
};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashSet, sync::Arc};

use anyhow::{Context as _, anyhow};

#[derive(Debug, Clone)]
pub struct JwtConfig {
    pub(crate) audience: String,
    pub(crate) domain: String,
    pub(crate) jwks_url: String,
}

impl JwtConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        #[derive(Deserialize)]
        struct EnvironmentConfig {
            audience: String,
            domain: String,
        }

        let config = envy::prefixed("JWT_")
            .from_env::<EnvironmentConfig>()
            .context("missing JWT environment variables (JWT_AUDIENCE, JWT_DOMAIN)")?;
        let jwks_url = std::env::var("JWKS_URL")
            .unwrap_or_else(|_| format!("https://{}/.well-known/jwks.json", config.domain));
        Ok(Self {
            audience: config.audience,
            domain: config.domain,
            jwks_url,
        })
    }
}

#[derive(Debug)]
pub enum ClientError {
    Authentication,
    InvalidToken,
    JwksFetch(Arc<anyhow::Error>),
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
            Self::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                Some("invalid_token".to_string()),
                Some("The access token is invalid".to_string()),
                "Bad credentials".to_string(),
            ),
            Self::JwksFetch(error) => {
                tracing::error!(error = ?error, "JWKS infrastructure failure");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Some("server_error".to_string()),
                    Some("Authentication service is temporarily unavailable".to_string()),
                    "Service temporarily unavailable".to_string(),
                )
            }
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

        let header = decode_header(token).map_err(|_| ClientError::InvalidToken)?;
        let kid = header.kid.ok_or(ClientError::InvalidToken)?;
        let domain = config.domain.as_str();

        // Fetch JWKS from cache; on cache miss, fetch_jwks is called exactly once
        // (try_get_with deduplicates concurrent requests for the same key)
        let jwks = state
            .jwks_cache
            .try_get_with((), fetch_jwks(&config.jwks_url))
            .await
            .map_err(ClientError::JwksFetch)?;

        // Validate token if the matching key is found in the cached JWKS
        if let Some(jwk) = jwks.find(&kid) {
            return validate_claims(jwk, token, domain, &config.audience);
        }

        // kid not found: the provider may have rotated keys; invalidate the cache and
        // re-fetch once. try_get_with ensures only one in-flight fetch even under concurrency.
        state.jwks_cache.invalidate(&()).await;
        let jwks = state
            .jwks_cache
            .try_get_with((), fetch_jwks(&config.jwks_url))
            .await
            .map_err(ClientError::JwksFetch)?;

        let jwk = jwks.find(&kid).ok_or(ClientError::InvalidToken)?;
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
            let issuer = Uri::builder()
                .scheme("https")
                .authority(domain)
                .path_and_query("/")
                .build()
                .context("invalid JWT_DOMAIN issuer configuration")
                .map_err(|error| ClientError::JwksFetch(Arc::new(error)))?;
            validation.set_issuer(&[issuer]);
            let key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                .map_err(|_| ClientError::InvalidToken)?;
            let token_data = decode::<Claims>(token, &key, &validation)
                .map_err(|_| ClientError::InvalidToken)?;
            Ok(token_data.claims)
        }
        _ => Err(ClientError::InvalidToken),
    }
}

/// Validates that the JWKS URL is safe to fetch: `http://` is only permitted
/// for loopback addresses.
fn validate_jwks_url(url: &str) -> Result<(), anyhow::Error> {
    let uri: Uri = url.parse().context("invalid JWKS_URL configuration")?;
    if uri.scheme_str() == Some("http") {
        let host = uri.host().unwrap_or("");
        // uri.host() preserves brackets for IPv6 addresses (e.g. "[::1]"), so we strip them
        // manually before parsing into IpAddr. Using the `url` crate would avoid this via its
        // Host enum, but adding that dependency solely for this function is not warranted.
        let bare_host = host
            .strip_prefix('[')
            .and_then(|h| h.strip_suffix(']'))
            .unwrap_or(host);
        let is_loopback = bare_host == "localhost"
            || bare_host
                .parse::<std::net::IpAddr>()
                .map(|ip| ip.is_loopback())
                .unwrap_or(false);
        if !is_loopback {
            return Err(anyhow!(
                "http:// JWKS_URL is only permitted for loopback addresses"
            ));
        }
    }
    Ok(())
}

async fn fetch_jwks(url: &str) -> Result<Arc<JwkSet>, anyhow::Error> {
    validate_jwks_url(url)?;
    let client = build_http_client().context("failed to build JWKS HTTP client")?;
    let response = client
        .get(url)
        .send()
        .await
        .context("JWKS request failed")?;
    let response = response
        .error_for_status()
        .context("JWKS endpoint returned an error status")?;
    let jwks = response
        .json::<JwkSet>()
        .await
        .context("failed to decode JWKS response body")?;
    Ok(Arc::new(jwks))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::to_bytes, extract::State, http::Request, routing::get};
    use jsonwebtoken::{EncodingKey, Header, encode};
    use moka::future::Cache;
    use serde::Serialize;
    use serde_json::{Value, json};
    use std::{
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        },
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    const TEST_AUDIENCE: &str = "test-audience";
    const TEST_DOMAIN: &str = "test-issuer.local";
    const TEST_ISSUER: &str = "https://test-issuer.local/";
    const TEST_KID: &str = "test-key-id";
    const TEST_PRIVATE_KEY: &str = include_str!("../../../testdata/test_private_key.pem");
    const TEST_JWKS: &str = include_str!("../../../testdata/test_jwks.json");

    type JwksServerState = (Arc<Vec<(StatusCode, String)>>, Arc<AtomicUsize>);

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        aud: String,
        iss: String,
        exp: u64,
    }

    fn test_config(jwks_url: String) -> JwtConfig {
        JwtConfig {
            audience: TEST_AUDIENCE.to_owned(),
            domain: TEST_DOMAIN.to_owned(),
            jwks_url,
        }
    }

    fn empty_state(jwks_url: String) -> Arc<AppState> {
        Arc::new(AppState {
            jwt_config: test_config(jwks_url),
            jwks_cache: Cache::builder()
                .max_capacity(1)
                .time_to_live(Duration::from_hours(1))
                .build(),
        })
    }

    async fn state_with_jwks(jwks: JwkSet) -> Arc<AppState> {
        let state = empty_state("https://unused.example/jwks.json".to_owned());
        state.jwks_cache.insert((), Arc::new(jwks)).await;
        state
    }

    fn test_jwks_with_kid(kid: &str) -> JwkSet {
        let mut value: Value = serde_json::from_str(TEST_JWKS).unwrap();
        value["keys"][0]["kid"] = Value::String(kid.to_owned());
        serde_json::from_value(value).unwrap()
    }

    fn token(header: Header, audience: &str, issuer: &str, exp: u64) -> String {
        let claims = TestClaims {
            sub: "test-user".to_owned(),
            aud: audience.to_owned(),
            iss: issuer.to_owned(),
            exp,
        };
        let key = EncodingKey::from_rsa_pem(TEST_PRIVATE_KEY.as_bytes()).unwrap();
        encode(&header, &claims, &key).unwrap()
    }

    fn valid_token() -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(TEST_KID.to_owned());
        token(
            header,
            TEST_AUDIENCE,
            TEST_ISSUER,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 3600,
        )
    }

    async fn extract(token: Option<&str>, state: &Arc<AppState>) -> Result<Claims, ClientError> {
        let mut builder = Request::builder().uri("/me");
        if let Some(token) = token {
            builder = builder.header("Authorization", format!("Bearer {token}"));
        }
        let request = builder.body(()).unwrap();
        let (mut parts, _) = request.into_parts();
        Claims::from_request_parts(&mut parts, state).await
    }

    async fn response_json(error: ClientError) -> (StatusCode, Value) {
        let response = error.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    async fn spawn_jwks_server(
        responses: Vec<(StatusCode, String)>,
    ) -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
        let responses = Arc::new(responses);
        let requests = Arc::new(AtomicUsize::new(0));
        let app = Router::new()
            .route(
                "/.well-known/jwks.json",
                get(
                    |State((responses, requests)): State<JwksServerState>| async move {
                        let index = requests.fetch_add(1, Ordering::SeqCst);
                        responses[index.min(responses.len() - 1)].clone()
                    },
                ),
            )
            .with_state((responses, requests.clone()));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        (
            format!("http://{address}/.well-known/jwks.json"),
            requests,
            server,
        )
    }

    #[test]
    fn http_localhost_is_allowed() {
        assert!(validate_jwks_url("http://localhost/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_127_0_0_1_is_allowed() {
        assert!(validate_jwks_url("http://127.0.0.1/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_ipv6_loopback_is_allowed() {
        assert!(validate_jwks_url("http://[::1]/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_ipv6_loopback_full_form_is_allowed() {
        assert!(validate_jwks_url("http://[0:0:0:0:0:0:0:1]/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn http_non_loopback_ip_is_rejected() {
        assert!(validate_jwks_url("http://192.168.1.1/.well-known/jwks.json").is_err());
    }

    #[test]
    fn http_non_loopback_host_is_rejected() {
        assert!(validate_jwks_url("http://example.com/.well-known/jwks.json").is_err());
    }

    #[test]
    fn https_non_loopback_is_allowed() {
        assert!(validate_jwks_url("https://example.auth0.com/.well-known/jwks.json").is_ok());
    }

    #[test]
    fn invalid_url_is_rejected() {
        assert!(validate_jwks_url("not a url").is_err());
    }

    #[tokio::test]
    async fn missing_authorization_is_unauthorized() {
        let state = empty_state("https://unused.example/jwks.json".to_owned());

        let (status, body) = response_json(extract(None, &state).await.unwrap_err()).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["message"], "Requires authentication");
    }

    #[tokio::test]
    async fn malformed_and_missing_kid_tokens_are_unauthorized() {
        let state = empty_state("https://unused.example/jwks.json".to_owned());
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let without_kid = token(
            Header::new(Algorithm::RS256),
            TEST_AUDIENCE,
            TEST_ISSUER,
            now + 3600,
        );

        for candidate in ["not-a-jwt", without_kid.as_str()] {
            let (status, body) =
                response_json(extract(Some(candidate), &state).await.unwrap_err()).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(body["error"], "invalid_token");
            assert_eq!(body["error_description"], "The access token is invalid");
        }
    }

    #[tokio::test]
    async fn jwt_validation_failures_are_unauthorized() {
        let jwks = test_jwks_with_kid(TEST_KID);
        let state = state_with_jwks(jwks).await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(TEST_KID.to_owned());
        let expired = token(header.clone(), TEST_AUDIENCE, TEST_ISSUER, now - 3600);
        let wrong_audience = token(header.clone(), "wrong-audience", TEST_ISSUER, now + 3600);
        let wrong_issuer = token(
            header.clone(),
            TEST_AUDIENCE,
            "https://wrong-issuer.example/",
            now + 3600,
        );
        let mut invalid_signature = valid_token();
        let last = invalid_signature.pop().unwrap();
        invalid_signature.push(if last == 'A' { 'B' } else { 'A' });

        for candidate in [expired, wrong_audience, wrong_issuer, invalid_signature] {
            let (status, body) =
                response_json(extract(Some(&candidate), &state).await.unwrap_err()).await;
            assert_eq!(status, StatusCode::UNAUTHORIZED);
            assert_eq!(body["error"], "invalid_token");
        }
    }

    #[tokio::test]
    async fn unsupported_jwk_algorithm_is_unauthorized() {
        let jwks = serde_json::from_value(json!({
            "keys": [{
                "kty": "oct",
                "k": "c2VjcmV0",
                "kid": TEST_KID
            }]
        }))
        .unwrap();
        let state = state_with_jwks(jwks).await;

        let (status, body) =
            response_json(extract(Some(&valid_token()), &state).await.unwrap_err()).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"], "invalid_token");
    }

    #[tokio::test]
    async fn valid_token_uses_cached_jwks() {
        let jwks = serde_json::to_string(&test_jwks_with_kid(TEST_KID)).unwrap();
        let (url, requests, server) = spawn_jwks_server(vec![(StatusCode::OK, jwks)]).await;
        let state = empty_state(url);

        assert!(extract(Some(&valid_token()), &state).await.is_ok());
        assert!(extract(Some(&valid_token()), &state).await.is_ok());

        assert_eq!(requests.load(Ordering::SeqCst), 1);
        server.abort();
    }

    #[tokio::test]
    async fn unknown_kid_refreshes_once_and_accepts_rotated_key() {
        let old = serde_json::to_string(&test_jwks_with_kid("old-key")).unwrap();
        let rotated = serde_json::to_string(&test_jwks_with_kid(TEST_KID)).unwrap();
        let (url, requests, server) =
            spawn_jwks_server(vec![(StatusCode::OK, old), (StatusCode::OK, rotated)]).await;
        let state = empty_state(url);

        assert!(extract(Some(&valid_token()), &state).await.is_ok());
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        server.abort();
    }

    #[tokio::test]
    async fn unknown_kid_stops_after_one_refresh() {
        let old = serde_json::to_string(&test_jwks_with_kid("old-key")).unwrap();
        let (url, requests, server) = spawn_jwks_server(vec![(StatusCode::OK, old)]).await;
        let state = empty_state(url);

        let (status, _) =
            response_json(extract(Some(&valid_token()), &state).await.unwrap_err()).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(requests.load(Ordering::SeqCst), 2);
        server.abort();
    }

    #[tokio::test]
    async fn jwks_http_and_decode_failures_return_sanitized_service_unavailable() {
        for (response_status, response_body, forbidden) in [
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "upstream secret HTTP failure",
                "500",
            ),
            (StatusCode::OK, "invalid JWKS response", "invalid JWKS"),
        ] {
            let (url, _, server) =
                spawn_jwks_server(vec![(response_status, response_body.to_owned())]).await;
            let state = empty_state(url);
            let (status, body) =
                response_json(extract(Some(&valid_token()), &state).await.unwrap_err()).await;
            let encoded = body.to_string();

            assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
            assert_eq!(body["error"], "server_error");
            assert_eq!(
                body["error_description"],
                "Authentication service is temporarily unavailable"
            );
            assert_eq!(body["message"], "Service temporarily unavailable");
            assert!(!encoded.contains(forbidden));
            assert!(!encoded.contains("reqwest"));
            server.abort();
        }
    }

    #[tokio::test]
    async fn jwks_connection_failure_returns_sanitized_service_unavailable() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let state = empty_state(format!("http://{address}/.well-known/jwks.json"));

        let (status, body) =
            response_json(extract(Some(&valid_token()), &state).await.unwrap_err()).await;
        let encoded = body.to_string();

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body["error"], "server_error");
        assert!(!encoded.contains("connection refused"));
        assert!(!encoded.contains(&address.to_string()));
    }

    #[tokio::test]
    async fn invalid_domain_returns_sanitized_service_unavailable() {
        let state = state_with_jwks(test_jwks_with_kid(TEST_KID)).await;
        let state = Arc::new(AppState {
            jwt_config: JwtConfig {
                domain: "invalid domain".to_owned(),
                ..state.jwt_config.clone()
            },
            jwks_cache: state.jwks_cache.clone(),
        });

        let (status, body) =
            response_json(extract(Some(&valid_token()), &state).await.unwrap_err()).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            body["error_description"],
            "Authentication service is temporarily unavailable"
        );
        assert!(!body.to_string().contains("invalid domain"));
    }
}
