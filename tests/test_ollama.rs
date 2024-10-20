// tests/test_ollama.rs

use anyhow::Result;
use axum::http::StatusCode;
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
async fn test_generate_api() -> Result<()> {
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

    for (req, expected_status) in test_cases {
        let request_body = serde_json::to_vec(&req)?;

        let request = reqwest::Client::new()
            .post(format!("http://{}/api/generate", addr))
            .header(CONTENT_TYPE, "application/json")
            .body(request_body)
            .build()?;

        let response = client.execute(request).await?;

        let status = response.status();
        let response_body = response.text().await?;
        let response_json: Value = serde_json::from_str(&response_body)?;
        let response_text = response_json["response"].as_str().unwrap();
        assert!(response_text.contains("beijing"));

        assert_eq!(status, expected_status);
    }

    Ok(())
}
