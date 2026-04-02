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
    jwk::{AlgorithmParameters, JwkSet},
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
    #[display("not_found")]
    NotFound(String),
    #[display("unsupported_algorithm")]
    UnsupportedAlgortithm(AlgorithmParameters),
}

impl IntoResponse for ClientError {
    fn into_response(self) -> Response {
        let (error, error_description, message) = match self {
            Self::Authentication => (None, None, "Requires authentication".to_string()),
            Self::Decode(_) => (
                Some("invalid_token".to_string()),
                Some(
                    "Authorization header value must follow this format: Bearer access-token"
                        .to_string(),
                ),
                "Bad credentials".to_string(),
            ),
            Self::NotFound(msg) => (
                Some("invalid_token".to_string()),
                Some(msg),
                "Bad credentials".to_string(),
            ),
            Self::UnsupportedAlgortithm(alg) => (
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
        (StatusCode::UNAUTHORIZED, body).into_response()
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
        let jwks = fetch_jwks(domain).await.unwrap(); // TODO
        let jwk = jwks
            .find(&kid)
            .ok_or_else(|| ClientError::NotFound("No JWK found for kid".to_string()))?;
        match jwk.clone().algorithm {
            AlgorithmParameters::RSA(ref rsa) => {
                let mut validation = Validation::new(Algorithm::RS256);
                validation.set_audience(&[config.audience]);
                validation.set_issuer(&[Uri::builder()
                    .scheme("https")
                    .authority(domain)
                    .path_and_query("/")
                    .build()
                    .unwrap()]);
                let key = DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                    .map_err(ClientError::Decode)?;
                let token =
                    decode::<Claims>(token, &key, &validation).map_err(ClientError::Decode)?;
                Ok(token.claims)
            }
            algorithm => Err(ClientError::UnsupportedAlgortithm(algorithm)),
        }
    }
}

#[derive(Debug, Display, derive_more::Error)]
#[display("my error: {message}")]
struct MyError {
    message: String,
}

async fn fetch_jwks(domain: &str) -> Result<JwkSet, MyError> {
    let uri = std::env::var("JWKS_URL")
        .unwrap_or_else(|_| format!("https://{}/.well-known/jwks.json", domain));
    let client = reqwest::ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .unwrap();
    let response = client.get(uri).send().await;
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            return Err(MyError {
                message: format!("TODO1: {}", e),
            });
        }
    };
    match response.json().await {
        Ok(jwks) => Ok(jwks),
        Err(_) => Err(MyError {
            message: "TODO2".to_string(),
        }),
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[cfg(test)]
mod test {
    use crate::presentation::extractor::claims::MyError;

    #[test]
    fn my_error_to_string() {
        assert_eq!(
            MyError {
                message: "test".to_string()
            }
            .to_string(),
            "my error: test"
        );
    }
}
