use anyhow::Context as _;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    pub port: u16,
    pub client_origin_url: String,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

impl Config {
    pub fn from_env() -> Result<Self, anyhow::Error> {
        envy::from_env::<Self>().context("missing environment variables for Config")
    }
}

#[derive(Serialize)]
pub struct ErrorMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    pub message: String,
}
