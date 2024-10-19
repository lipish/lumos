use anyhow::{Context, Result};
use axum::{
    extract::{Json, State},
    response::IntoResponse,
};
use std::sync::Arc;

use crate::config::Config;
use crate::ollama::dispatch::dispatch;
use crate::structs::app::AppState;
use crate::structs::ollama::ChatRequest;

pub async fn handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ChatRequest>,
) -> Result<impl IntoResponse, (axum::http::StatusCode, String)> {
    chat(State(state), Json(request))
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> Result<impl IntoResponse, anyhow::Error> {
    let model = &state.model_name;
    let config_path = &state.config_path;

    let config = Config::from_file(config_path).context("Failed to load config")?;
    let provider = config.models.get(model).context("Provider not found")?;

    // Dispatch the request to the provider service and get the stream
    dispatch(model, req.messages, provider).await
}
