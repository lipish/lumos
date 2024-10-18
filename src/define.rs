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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>, // For multimodal models
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>, // Only if stream is false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ChatOptions>,
    #[serde(default = "default_stream")]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    // Add other options as needed (e.g., top_p, n, etc.)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    pub type_: String,
    pub function: ToolFunction,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // Can be any JSON value
}

fn default_stream() -> bool {
    true
}

impl Default for ChatRequest {
    fn default() -> Self {
        ChatRequest {
            model: String::new(),
            messages: Vec::new(),
            stream: false,
            tools: None,
            format: None,
            options: None,
            keep_alive: None,
        }
    }
}

impl Default for ChatMessage {
    fn default() -> Self {
        ChatMessage {
            role: String::new(),
            content: String::new(),
            images: None,
            tool_calls: None,
        }
    }
}
