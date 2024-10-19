use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ProviderName {
    #[serde(rename = "zhipu")]
    Zhipu,
    #[serde(rename = "deepseek")]
    DeepSeek,
}

impl FromStr for ProviderName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zhipu" => Ok(ProviderName::Zhipu),
            "deepseek" => Ok(ProviderName::DeepSeek),

            _ => Err(anyhow::anyhow!("Invalid provider name: {}", s)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Provider {
    #[serde(deserialize_with = "deserialize_provider_name", alias = "provider")]
    pub name: ProviderName,
    pub api_key: String,
    pub url: String,
}

fn deserialize_provider_name<'de, D>(deserializer: D) -> Result<ProviderName, D::Error>
where
    D: Deserializer<'de>,
{
    let provider_str = String::deserialize(deserializer)?;
    ProviderName::from_str(&provider_str).map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Model {
    name: String,
    provider_config: Provider,
}
