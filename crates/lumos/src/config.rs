use anyhow::Result;
use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use crate::structs::config::Provider;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub models: HashMap<String, Provider>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }
}

pub fn check_model_name(model_name: &str, config_path: &str) -> bool {
    let config_result = Config::from_file(config_path);

    match config_result {
        Ok(config) => config.models.contains_key(model_name),
        Err(_) => false, // Handle the error as needed, e.g., log the error
    }
}
