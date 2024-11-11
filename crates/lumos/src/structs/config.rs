use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ProviderName {
    #[serde(rename = "zhipu")]
    Zhipu,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "xinference")]
    Xinference,
}

impl fmt::Display for ProviderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderName::Zhipu => write!(f, "zhipu"),
            ProviderName::DeepSeek => write!(f, "deepseek"),
            ProviderName::Xinference => write!(f, "xinference"),
        }
    }
}

impl FromStr for ProviderName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zhipu" => Ok(ProviderName::Zhipu),
            "deepseek" => Ok(ProviderName::DeepSeek),
            "xinference" => Ok(ProviderName::Xinference),

            _ => Err(anyhow::anyhow!("Invalid provider name: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Model {
    pub model_name: String,
    pub provider: ProviderName,
    pub api_key: String,
    pub url: String,
}
