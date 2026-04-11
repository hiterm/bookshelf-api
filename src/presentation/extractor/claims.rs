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

impl JwtConfig {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        use anyhow::Context as _;
        envy::prefixed("JWT_")
            .from_env::<Self>()
            .context("missing JWT environment variables (JWT_AUDIENCE, JWT_DOMAIN)")
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
    UnsupportedAlgorithm(AlgorithmParameters),
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
            Self::UnsupportedAlgorithm(alg) => (
                StatusCode::UNAUTHORIZED,
                Some("invalid_token".to_string()),
                Some(format!(
                    "Unsupported encryption algorithm expected RSA got {:?}",
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

        // Fetch JWKS from cache; on cache miss, fetch_jwks is called exactly once
        // (try_get_with deduplicates concurrent requests for the same key)
        let jwks = state
            .jwks_cache
            .try_get_with((), fetch_jwks(domain))
            .await
            .map_err(|e| ClientError::JwksFetch(format!("JWKS fetch failed: {e}")))?;

        // Validate token if the matching key is found in the cached JWKS
        if let Some(jwk) = jwks.find(&kid) {
            return validate_claims(jwk, token, domain, &config.audience);
        }

        // kid not found: the provider may have rotated keys; invalidate the cache and
        // re-fetch once. try_get_with ensures only one in-flight fetch even under concurrency.
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
            let issuer = Uri::builder()
                .scheme("https")
                .authority(domain)
                .path_and_query("/")
                .build()
                .map_err(|e| ClientError::JwksFetch(format!("invalid domain: {e}")))?;
            validation.set_issuer(&[issuer]);
            let key =
                DecodingKey::from_rsa_components(&rsa.n, &rsa.e).map_err(ClientError::Decode)?;
            let token_data =
                decode::<Claims>(token, &key, &validation).map_err(ClientError::Decode)?;
            Ok(token_data.claims)
        }
        algorithm => Err(ClientError::UnsupportedAlgorithm(algorithm.clone())),
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
            return Err(ClientError::JwksFetch(
                "http:// JWKS_URL is only permitted for loopback addresses".to_string(),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
