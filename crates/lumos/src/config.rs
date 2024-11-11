use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::structs::config::Model;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config(HashMap<String, Model>);

impl Config {
    pub fn get_model(&self, name: &str) -> Option<&Model> {
        self.0.get(name)
    }

    pub fn models(&self) -> &HashMap<String, Model> {
        &self.0
    }

    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn contains_model(&self, model_name: &str) -> bool {
        self.0.iter().any(|(model, _)| model == model_name)
    }
}

pub fn check_model_name(model_name: &str, config_path: &str) -> bool {
    let config_result = Config::from_file(config_path);
    match config_result {
        Ok(config) => config.contains_model(model_name),
        Err(error) => {
            println!("Error reading config file: {}", error);
            false
        }
    }
}
