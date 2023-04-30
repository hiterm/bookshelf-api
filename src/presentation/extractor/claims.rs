use std::collections::HashSet;
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::State;
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::{extract::FromRequestParts, TypedHeader};
use axum::{Json, RequestPartsExt};
use derive_more::Display;
use http::{StatusCode, Uri};
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Clone, Deserialize)]
pub struct Auth0Config {
    audience: String,
    domain: String,
}

impl Default for Auth0Config {
    fn default() -> Self {
        envy::prefixed("AUTH0_")
            .from_env()
            .expect("Provide missing environment variables for Auth0Client")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub _permissions: Option<HashSet<String>>,
}

pub struct AppState {
    auth0_config: Auth0Config,
}

#[async_trait]
impl FromRequestParts<State<Arc<AppState>>> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        State(state): &State<Arc<AppState>>,
    ) -> Result<Self, Self::Rejection> {
        let config = state.auth0_config.clone();
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        let token = bearer.token();

        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        let kid = header.kid.ok_or_else(|| AuthError::InvalidToken)?;
        let domain = config.domain.as_str();
        // TODO: サンプル実装の通りに戻せそうなら戻す
        // https://github.com/auth0-developer-hub/api_actix-web_rust_hello-world/blob/c86861763a4a4f2ad5f0e39bb3c15a7216d3fdba/src/extractors/claims.rs#L107-L119
        let jwks = fetch_jwks(domain).await.unwrap(); // TODO
        let jwk = jwks.find(&kid).ok_or_else(|| AuthError::InvalidToken)?;
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
                    .map_err(|_| AuthError::InvalidToken)?;
                let token = decode::<Claims>(token, &key, &validation)
                    .map_err(|_| AuthError::InvalidToken)?;
                Ok(token.claims)
            }
            algorithm => Err(AuthError::InvalidToken),
        }
    }
}

#[derive(Debug, Display, derive_more::Error)]
#[display(fmt = "my error: {}", message)]
struct MyError {
    message: String,
}

async fn fetch_jwks(domain: &str) -> Result<JwkSet, MyError> {
    let uri = Uri::builder()
        .scheme("https")
        .authority(domain)
        .path_and_query("/.well-known/jwks.json")
        .build()
        .unwrap();
    let response = reqwest::get(uri.to_string()).await;
    let response = match response {
        Ok(response) => response,
        Err(e) => {
            return Err(MyError {
                message: format!("TODO1: {}", e),
            })
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
