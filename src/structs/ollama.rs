use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

fn is_false(value: &bool) -> bool {
    !(*value)
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GenerateRequest {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>, // base64 encoded images
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>, // "json"
    pub options: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<u8>>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default, skip_serializing_if = "is_false")] // Treat missing as false
    pub raw: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
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

fn default_stream() -> bool {
    true
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

impl Default for Message {
    fn default() -> Self {
        Message {
            role: String::new(),
            content: String::new(),
            images: None,
            tool_calls: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    pub type_: String,
    pub function: ToolFunction,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // Can be any JSON value
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChatOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    // Add other options as needed (e.g., top_p, n, etc.)
}
