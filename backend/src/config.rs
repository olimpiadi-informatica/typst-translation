use std::fs;
use std::path::Path;

use color_eyre::eyre::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub jwt_signing_key: String,
    pub admin_password: String,
    pub staff_password: String,
    pub listen_address: String,
    pub gemini_api_key: String,
}

impl AppConfig {
    pub fn from_file(path: &Path) -> Result<Self> {
        let config_content = fs::read_to_string(path)?;
        let config: AppConfig = toml::from_str(&config_content)?;
        Ok(config)
    }
}
