use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::serve;
use serde_json::json;

use lumos::config::check_model_name;
use lumos::ollama::chat::handler as chat;
use lumos::ollama::generate::handler as generate;

use axum::{
    response::Json,
    routing::{get, post},
    Router,
};
use clap::Parser;
use lumos::structs::app::AppState;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

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

    // Save the model name and config path in the app state
    let app_state = Arc::new(AppState {
        model_name: cli.model_name,
        config_path: cli.config_file,
    });

    let app = Router::new()
        .route("/api/chat", post(chat))
        .route("/api/tags", get(list_model))
        .route("/api/ping", get(ping))
        .route("/api/generate", post(generate))
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
