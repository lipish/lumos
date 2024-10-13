use std::collections::HashMap;
use std::fs;

use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::ollama::OllamaService;

#[derive(Debug, Deserialize)]
pub struct ModelConfig {
    pub provider: Option<String>,
    pub url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)] // Important: allows for arbitrary model names as keys
    pub models: HashMap<String, ModelConfig>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

pub async fn init_model_service(config: &Config, model_name: &str) -> Result<OllamaService> {
    let model_config = config
        .models
        .get(model_name)
        .ok_or_else(|| anyhow!("Model {} not found in config", model_name))?;

    // Use defaults if values aren't provided in the config
    let provider = model_config.provider.as_deref();
    let service_url = model_config.url.as_deref();
    let api_key = model_config.api_key.as_deref();

    OllamaService::new(provider, service_url, api_key).await // Assuming you have a new() method on OllamaService
}

pub fn check_model_name(model_name: &str, config_path: &str) -> bool {
    let config_result = Config::from_file(config_path);

    match config_result {
        Ok(config) => config.models.contains_key(model_name),
        Err(_) => false, // Handle the error as needed, e.g., log the error
    }
}
