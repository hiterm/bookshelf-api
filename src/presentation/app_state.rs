use super::extractor::claims::JwtConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    pub jwt_config: JwtConfig,
}
