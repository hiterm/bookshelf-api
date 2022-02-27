use crate::types::ErrorMessage;
use actix_web::{
    error::ResponseError,
    http::{StatusCode, Uri},
    Error, FromRequest, HttpResponse,
};
use actix_web_httpauth::{
    extractors::bearer::BearerAuth, headers::www_authenticate::bearer::Bearer,
};
use awc::Client;
use derive_more::Display;
use jsonwebtoken::{
    decode, decode_header,
    jwk::{AlgorithmParameters, JwkSet},
    Algorithm, DecodingKey, Validation,
};
use serde::Deserialize;
use std::{collections::HashSet, future::Future, pin::Pin};

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

#[derive(Debug, Display)]
enum ClientError {
    #[display(fmt = "authentication")]
    Authentication(actix_web_httpauth::extractors::AuthenticationError<Bearer>),
    #[display(fmt = "decode")]
    Decode(jsonwebtoken::errors::Error),
    #[display(fmt = "not_found")]
    NotFound(String),
    #[display(fmt = "unsupported_algorithm")]
    UnsupportedAlgortithm(AlgorithmParameters),
}

impl ResponseError for ClientError {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Authentication(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: None,
                error_description: None,
                message: "Requires authentication".to_string(),
            }),
            Self::Decode(_) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(
                    "Authorization header value must follow this format: Bearer access-token"
                        .to_string(),
                ),
                message: "Bad credentials".to_string(),
            }),
            Self::NotFound(msg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(msg.to_string()),
                message: "Bad credentials".to_string(),
            }),
            Self::UnsupportedAlgortithm(alg) => HttpResponse::Unauthorized().json(ErrorMessage {
                error: Some("invalid_token".to_string()),
                error_description: Some(format!(
                    "Unsupported encryption algortithm expected RSA got {:?}",
                    alg
                )),
                message: "Bad credentials".to_string(),
            }),
        }
    }

    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }
}

#[derive(Debug, Deserialize)]
pub struct Claims {
    _permissions: Option<HashSet<String>>,
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let config = req.app_data::<Auth0Config>().unwrap().clone();
        let extractor = BearerAuth::extract(req);
        Box::pin(async move {
            let credentials = extractor.await.map_err(ClientError::Authentication)?;
            let token = credentials.token();
            let header = decode_header(token).map_err(ClientError::Decode)?;
            let kid = header.kid.ok_or_else(|| {
                ClientError::NotFound("kid not found in token header".to_string())
            })?;
            let domain = config.domain.as_str();
            // TODO: サンプル実装の通りに戻せそうなら戻す
            // https://github.com/auth0-developer-hub/api_actix-web_rust_hello-world/blob/c86861763a4a4f2ad5f0e39bb3c15a7216d3fdba/src/extractors/claims.rs#L107-L119
            let jwks = fetch_jwks(domain).await?;
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
                algorithm => Err(ClientError::UnsupportedAlgortithm(algorithm).into()),
            }
        })
    }
}

#[derive(Debug, Display, derive_more::Error)]
#[display(fmt = "my error: {}", name)]
struct MyError {
    name: &'static str,
}

// Use default implementation for `error_response()` method
impl ResponseError for MyError {}

async fn fetch_jwks(domain: &str) -> Result<JwkSet, MyError> {
    let response = Client::new()
        .get(
            Uri::builder()
                .scheme("https")
                .authority(domain)
                .path_and_query("/.well-known/jwks.json")
                .build()
                .unwrap(),
        )
        .send()
        .await;
    let mut response = match response {
        Ok(response) => response,
        Err(_) => return Err(MyError {name: "TODO"}),
    };
    match response.json().await {
        Ok(jwks) => Ok(jwks),
        Err(_) => Err(MyError {name: "TODO"}),
    }
}
