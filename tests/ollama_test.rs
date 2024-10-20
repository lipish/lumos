// tests/test_ollama.rs

use anyhow::Result;
use axum::http::StatusCode;
use futures_util::StreamExt;
use reqwest::header::CONTENT_TYPE;
use serde_json::Value;

use lumos::structs::app::AppState;
use lumos::structs::ollama::GenerateRequest;

use reqwest::Client;
use std::sync::Arc;

async fn spawn_app(app_state: Arc<AppState>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:11434")
        .await
        .unwrap();

    let app = lumos::app::create_app(app_state.clone()).await;

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap(); // Use axum::serve
    });
}

#[tokio::test]
async fn test_generate() -> Result<()> {
    // Setup app state for testing
    let app_state = Arc::new(AppState {
        model_name: "deepseek-chat".to_string(),
        config_path: "keys.toml".to_string(),
    });

    spawn_app(app_state).await;

    let client = Client::new();

    let test_cases = vec![(
        GenerateRequest {
            model: "deepseek-chat".to_string(),
            prompt: Some("What is the capital of China?".to_string()),
            ..Default::default()
        },
        StatusCode::OK,
    )];

    let addr = "127.0.0.1:11434";

    for (req, _) in test_cases {
        let request_body = serde_json::to_vec(&req)?;

        let request = reqwest::Client::new()
            .post(format!("http://{}/api/generate", addr))
            .header(CONTENT_TYPE, "application/json")
            .body(request_body)
            .build()?;

        let mut stream = client.execute(request).await?.bytes_stream();
        let mut response_text = String::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            println!("chunk: {:?}", chunk);
            let chunk_str = std::str::from_utf8(&chunk)?;
            println!("chunk_str: {}", chunk_str);
            let chunk_json: Value = serde_json::from_str(chunk_str)?;
            println!("chunk_json: {:?}", chunk_json);
            if let Some(response) = chunk_json["response"].as_str() {
                println!("response: {}", response);
                response_text.push_str(response);
            }
        }

        assert!(response_text.contains("beijing"));
    }

    Ok(())
}