/// List models that are available locally.
/// https://github.com/ollama/ollama/blob/main/docs/api.md#list-local-models
use crate::structs::app::AppState;
use axum::extract::State;
use axum::response::Json;
use chrono::Utc;
use hex::encode as hex_encode;
use rand::Rng;
use serde_json::json;
use std::sync::Arc;

pub async fn models(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let model_name = &state.model_name;

    let model_data = json!({
        "name": parse(model_name),
        "modified_at": Utc::now().to_rfc3339(),
        "size": 3825819519i64,
        "digest": format!("sha256:{}", hex_encode(rand::thread_rng().gen::<[u8; 32]>())),
        "details": {
            "format": "gguf",
            "family": "llama",
            "families": serde_json::Value::Null,
            "parameter_size": "7B",
            "quantization_level": "Q4_0",
        },
    });

    Json(json!({
        "models": [model_data]
    }))
}

/// Parse the model name to a format that ollama expects
/// like `deepseek:chat` or `glm:4-plus`
fn parse(model_name: &str) -> String {
    let parts: Vec<&str> = model_name.split('-').collect();
    if parts.len() > 1 {
        format!("{}:{}", parts[0], parts[1..].join("-"))
    } else {
        model_name.to_string()
    }
}
