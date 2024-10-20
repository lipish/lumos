use anyhow::{Context, Result};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::config::Config;
use crate::ollama::dispatch;
use crate::structs::app::AppState;
use crate::structs::ollama::ChatRequest;
use crate::structs::ollama::ChatType;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    chat(State(state), Json(request))
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Result<impl IntoResponse, anyhow::Error> {
    let model = &req.model.replacen(":", "-", 1); // deepseek:chat -> deepseek-chat

    // check model name if match in app state
    if model != &state.model_name {
        return Err(anyhow::anyhow!(
            "Model name not match in app state:{} != {}",
            model,
            state.model_name
        ));
    }

    let config_path = &state.config_path;

    let config = Config::from_file(config_path).context("Failed to load config")?;
    let provider = config.models.get(model).context("Provider not found")?;

    // Dispatch the request to the provider service and get the stream
    dispatch(model, req.messages, provider, ChatType::Chat).await
}
