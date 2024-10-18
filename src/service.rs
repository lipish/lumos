use crate::define::ChatRequest;
use crate::define::Provider;
use futures_util::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;
use serde_json::Value;

pub async fn send(
    req: ChatRequest,
    provider: &Provider,
) -> Result<impl Stream<Item = Result<String, anyhow::Error>> + Unpin, anyhow::Error> {
    let api_key = &provider.api_key;
    let messages = req
        .messages
        .into_iter()
        .map(|msg| {
            json!({
                "role": msg.role,
                "content": msg.content
            })
        })
        .collect::<Vec<_>>();
    let model = &req.model;

    let client = Client::new();

    let request_body = json!({
        "model": model,
        "messages": messages,
        "stream": true
    });

    let response = client
        .post(&provider.url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("API请求失败: {}", response.status()));
    }

    let stream = response
        .bytes_stream()
        .map(|result| -> Result<String, anyhow::Error> {
            let bytes = result.map_err(anyhow::Error::from)?;
            let text = String::from_utf8_lossy(&bytes).to_string();
            Ok(text)
        })
        .flat_map(|line_result| {
            futures_util::stream::iter(line_result.map(|line| {
                if line.trim() == "data: [DONE]" {
                    Ok(None)
                } else if line.starts_with("data: ") {
                    let json_str = line.trim_start_matches("data: ");
                    match serde_json::from_str::<Value>(json_str) {
                        Ok(json) => {
                            let content = json["choices"][0]["delta"]["content"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            if content.is_empty() {
                                Ok(None)
                            } else {
                                Ok(Some(content))
                            }
                        }
                        Err(_) => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }))
        })
        .filter_map(|result| futures_util::future::ready(result.transpose()))
        .take_while(|result| {
            futures_util::future::ready(result.as_ref().map_or(true, |s| s != "[DONE]"))
        });

    Ok(stream)
}

// Test deepseek
#[tokio::test]
async fn test_deepseek() -> Result<(), anyhow::Error> {
    use crate::config::Config;
    use crate::define::{ChatMessage, ChatRequest};
    use anyhow::Context;

    // read api key from keys.toml
    let config_path = "keys.toml";
    println!("config_path: {}", config_path);
    let config = Config::from_file(config_path)?;
    println!("config: {:?}", config);

    let provider = config
        .models
        .get("deepseek-chat")
        .context("未找到模型提供者")?;

    let req = ChatRequest {
        model: "deepseek-chat".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
            images: None,
            tool_calls: None,
        }],
        stream: true,
        ..Default::default()
    };

    let stream = send(req, &provider).await?;

    let lines = stream.collect::<Vec<_>>().await;

    // print the first 5 lines
    for line in lines.iter().take(5) {
        println!("{:?}", line);
    }

    assert!(lines.len() > 1);
    assert!(lines[0].as_ref().unwrap().contains("Hello"));

    Ok(())
}
