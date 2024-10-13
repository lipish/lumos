use std::sync::Arc;

use anyhow::{Context, Result};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use axum::serve;
use serde_json::json;

use lumos::config::{check_model_name, Config};
use lumos::define::{ChatRequest, ProviderName};

use axum::{
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(Clone)]
struct AppState {
    model_name: String,
    config_path: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Name of the model to use
    model_name: String,

    /// Server host address
    #[arg(short, long, default_value = "localhost")]
    host: String,

    /// Server port
    #[arg(short, long, default_value_t = 11434)]
    port: u16,

    /// Path to the Toml configuration file
    #[arg(short, long, default_value = "keys.toml")]
    config_file: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    if !check_model_name(&cli.model_name, &cli.config_file) {
        eprintln!(
            "Model name {} is not available in config file {}",
            cli.model_name, cli.config_file
        );
        std::process::exit(1);
    }

    let config = Config::from_file(&cli.config_file)?;

    // Save the model name and config path in the app state
    let app_state = Arc::new(AppState {
        model_name: cli.model_name,
        config_path: cli.config_file,
    });

    let app = Router::new()
        .route("/api/chat", post(chat))
        .route("/api/tags", get(list_model))
        .route("/api/ping", get(ping))
        .with_state(app_state)
        .layer(CorsLayer::new().allow_origin(Any));

    let addr = format!("{}:{}", cli.host, cli.port);
    info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    serve(listener, app.into_make_service()).await?;

    Ok(())
}

async fn list_model(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({"model_name": state.model_name.clone()}))
}

async fn ping(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(json!({"model_name": state.model_name.clone()}))
}

async fn chat(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ChatRequest>,
) -> impl IntoResponse {
    match _chat(state, req).await {
        Ok(response) => response.into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

async fn _chat(state: Arc<AppState>, req: ChatRequest) -> anyhow::Result<Response> {
    let model_name = &state.model_name;
    let config_path = &state.config_path;

    let config = Config::from_file(config_path).context("无法加载配置文件")?;
    let model_config = config.models.get(model_name);

    match model_config {
        Some(provider) => match provider.name {
            ProviderName::OpenAI => {
                todo!()
            }
            ProviderName::DeepSeek => {
                todo!()
            }
        },
        None => {
            let error_message = format!("错误：未找到模型 {} 的提供者", model_name);
            Err(anyhow::anyhow!(error_message))
        }
    }
}
