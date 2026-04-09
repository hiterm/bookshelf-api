use jsonwebtoken::jwk::JwkSet;
use moka::future::Cache;
use std::sync::Arc;

use super::extractor::claims::JwtConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    pub jwt_config: JwtConfig,
    pub jwks_cache: Cache<String, Arc<JwkSet>>,
}
