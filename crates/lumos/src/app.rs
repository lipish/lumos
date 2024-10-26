use std::sync::Arc;

use axum::extract::State;
use serde_json::json;

use crate::ollama::chat_handler as chat;
use crate::ollama::generate_handler as generate;
use crate::ollama::models;

use crate::structs::app::AppState;
use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

pub async fn create_app(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/chat", post(chat))
        .route("/api/tags", get(models)) //  或 /api/models
        .route("/api/ping", get(ping))
        .route("/api/generate", post(generate))
        .with_state(app_state)
        .layer(CorsLayer::new().allow_origin(Any))
}

async fn ping(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({"model_name": state.model_name.clone()}))
}
