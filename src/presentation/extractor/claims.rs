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
