use anyhow::{Context, Result};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::config::Config;
use crate::ollama::dispatch;
use crate::structs::app::AppState;
use crate::structs::ollama::GenerateRequest;
use crate::structs::ollama::Message;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GenerateRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    generate(State(state), Json(request))
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn generate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<GenerateRequest>,
) -> Result<impl IntoResponse, anyhow::Error> {
    let model_name = &state.model_name;
    let config_path = &state.config_path;

    let config = Config::from_file(config_path).context("Failed to load config")?;
    let provider = config
        .models
        .get(model_name)
        .context("Provider not found")?;

    let messages = vec![Message {
        role: "user".to_string(),
        content: req.prompt.unwrap(),
        ..Default::default()
    }];

    // Dispatch the request to the provider service and get the stream
    dispatch(model_name, messages, provider, None, None).await
}