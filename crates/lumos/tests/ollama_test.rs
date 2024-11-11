// tests/ollama_test.rs

use anyhow::Result;
use futures_util::StreamExt;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;

use lumos::structs::app::AppState;
use lumos::structs::ollama::{ChatRequest, GenerateRequest, Message};

async fn spawn_app(app_state: Arc<AppState>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:11434")
        .await
        .unwrap();
    let app = lumos::app::create_app(app_state.clone()).await;
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
}

async fn process_request<T: serde::Serialize>(
    client: &Client,
    addr: &str,
    endpoint: &str,
    req: T,
    content_key: &str,
) -> Result<String> {
    let request_body = serde_json::to_vec(&req)?;
    let request = client
        .post(format!("http://{}/api/{}", addr, endpoint))
        .header(CONTENT_TYPE, "application/json")
        .body(request_body)
        .build()?;

    let mut stream = client.execute(request).await?.bytes_stream();
    let mut response_text = String::new();
    let mut buffer = String::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let chunk_str = std::str::from_utf8(&chunk)?;
        buffer.push_str(chunk_str);

        while let Some(json_end) = buffer.find("}\n") {
            let json_str = &buffer[..=json_end];
            println!("Processing JSON: {}", json_str);

            if let Ok(chunk_json) = serde_json::from_str::<Value>(json_str) {
                if let Some(response) = chunk_json[content_key].as_str() {
                    response_text.push_str(response);
                } else if let Some(content) = chunk_json["message"][content_key].as_str() {
                    response_text.push_str(content);
                }
            } else {
                eprintln!("Failed to parse JSON: {}", json_str);
            }

            buffer = buffer[json_end + 2..].to_string();
        }
    }

    // 处理可能剩余在缓冲区中的最后一个 JSON 对象
    if !buffer.is_empty() {
        if let Ok(chunk_json) = serde_json::from_str::<Value>(&buffer) {
            if let Some(response) = chunk_json[content_key].as_str() {
                response_text.push_str(response);
            } else if let Some(content) = chunk_json["message"][content_key].as_str() {
                response_text.push_str(content);
            }
        } else {
            eprintln!("Failed to parse remaining JSON: {}", buffer);
        }
    }

    println!("response_text: {}", response_text);
    Ok(response_text)
}

#[tokio::test]
async fn test_generate() -> Result<()> {
    let model_name = "glm-4-plus";
    let app_state = Arc::new(AppState {
        model_name: model_name.to_string(),
        config_path: "keys.toml".to_string(),
    });

    spawn_app(app_state).await;

    let client = Client::new();
    let addr = "localhost:11434";

    let req = GenerateRequest {
        model: model_name.replacen("-", ":", 1).to_string(),
        prompt: Some("What is the capital of China?".to_string()),
        keep_alive: Some(serde_json::Value::Number(serde_json::Number::from(-1))),
        ..Default::default()
    };

    let response_text = process_request(&client, addr, "generate", req, "response").await?;
    assert!(response_text.to_lowercase().contains("beijing"));

    Ok(())
}

#[tokio::test]
async fn test_chat() -> Result<()> {
    let model_name = "glm-4-plus";
    let app_state = Arc::new(AppState {
        model_name: model_name.to_string(),
        config_path: "keys.toml".to_string(),
    });

    spawn_app(app_state).await;

    let client = Client::new();
    let addr = "localhost:11434";

    let req = ChatRequest {
        model: model_name.replacen(":", "-", 1).to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: "What is the capital of China?".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };

    let response_text = process_request(&client, addr, "chat", req, "content").await?;
    assert!(response_text.to_lowercase().contains("beijing"));

    Ok(())
}
