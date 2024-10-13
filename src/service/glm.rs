use anyhow::Result;
use async_trait::async_trait;
use axum::response::StreamBody;
use futures::stream::Stream;
use serde_json::json;
use tokio_stream::wrappers::ReceiverStream;

use crate::base::BaseModelService;
use crate::models::ChatRequest;

pub struct GLMModelService {
    // ... your fields for GLM (e.g., ZhipuAI client)
    api_key: String, // Example: You'll likely need an API key
}

impl GLMModelService {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            // ... initialize your ZhipuAI client or other components
        }
    }
}
