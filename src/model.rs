use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde_json::{json, Value};

pub enum LLMProvider {
    OpenAI,
    DeepSeek,
}

struct ProviderConfig {
    provider: LLMProvider,
    api_key: String,
    url: String,
}

pub struct Model {
    name: String,
    provider_config: ProviderConfig,
}

pub fn list_models(model_name: &str) -> Result<Value, anyhow::Error> {
    let model_data = json!({
        "name": parse_model_name(model_name),
        "modified_at": Utc::now().to_rfc3339(),
        "size": 1000000000i64,
        "digest": generate_random_digest(),
        "details": {
            "format": "gguf",
            "family": "llama",
            "families": serde_json::Value::Null,
            "parameter_size": "14b",
            "quantization_level": "Q4_0",
        },
    });

    Ok(json!({"models": [model_data]}))
}

fn parse_model_name(model_name: &str) -> &str {
    model_name
}

fn generate_random_digest() -> String {
    let rand_string: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();
    rand_string
}
