use super::extractor::claims::Auth0Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pub auth0_config: Auth0Config,
}
