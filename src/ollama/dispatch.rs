use anyhow::Result;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};

use crate::structs::config::Provider;
use crate::structs::ollama::ChatType;
use crate::structs::ollama::Message;
use futures_util::Stream;
use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub async fn dispatch(
    model: &str,
    messages: Vec<Message>,
    provider: &Provider,
    chat_type: ChatType,
) -> Result<impl IntoResponse, anyhow::Error> {
    // 将 model 转换为 String
    let model = model.to_string();

    // 发送请求到提供者服务并获取流
    let stream = send(model, messages, provider, chat_type).await?;

    // Convert the stream to a Body
    let body = Body::from_stream(stream);

    // Construct the response
    let response = Response::builder()
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap();

    Ok(response)
}

async fn send(
    model: String,
    messages: Vec<Message>,
    provider: &Provider,
    chat_type: ChatType,
) -> Result<impl Stream<Item = Result<String, anyhow::Error>> + Unpin, anyhow::Error> {
    let api_key = &provider.api_key;
    let messages = messages
        .into_iter()
        .map(|msg| {
            json!({
                "role": msg.role,
                "content": msg.content
            })
        })
        .collect::<Vec<_>>();
    let ollama_model = model.replacen('-', ":", 1);

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

    let done_flag = Arc::new(AtomicBool::new(false));
    let done_flag_clone = done_flag.clone();

    let stream = response
        .bytes_stream()
        .map(|result| -> Result<String, anyhow::Error> {
            let bytes = result.map_err(anyhow::Error::from)?;
            let text = String::from_utf8_lossy(&bytes).to_string();
            Ok(text)
        })
        .flat_map(move |line_result| {
            let done_flag = done_flag.clone();
            futures_util::stream::iter(line_result.map(move |line| {
                if line.trim() == "data: [DONE]" {
                    done_flag.store(true, Ordering::SeqCst);
                    Ok(Some("".to_string())) // keep the stream alive
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
                    Ok(Some("".to_string())) // keep the stream alive
                }
            }))
        })
        .filter_map(|result| futures_util::future::ready(result.transpose())) // filter out the empty string and flatten the result and option, keep the async feature
        .map(move |result| {
            let done_flag = done_flag_clone.clone();

            result.and_then(|content| {
                if content.contains("[DONE]") {
                    done_flag.store(true, Ordering::SeqCst);
                    let done_json = json!({
                        "model": ollama_model,
                        "created_at": chrono::Utc::now().to_rfc3339(),
                        "response": "",
                        "done": true,
                        "context": [1, 2, 3],
                        "total_duration": 122112,
                        "load_duration": 123112,
                        "prompt_eval_count": 26,
                        "prompt_eval_duration": 130079000,
                        "eval_count": 259,
                        "eval_duration": 2433122
                    });
                    Ok(done_json.to_string())
                } else {
                    let mut json_content = json!({
                        "model": ollama_model,
                        "created_at": chrono::Utc::now().to_rfc3339(),
                        "done": false
                    });

                    if chat_type == ChatType::Chat {
                        json_content["message"] = json!({
                            "role": "assistant",
                            "content": content,
                            "images": null
                        });
                    } else {
                        json_content["response"] = json!(content);
                    }

                    Ok(json_content.to_string())
                }
            })
        })
        .take_while(|result| {
            futures_util::future::ready(
                !result
                    .as_ref()
                    .map_or(false, |s| s.contains("\"done\": true")),
            )
        });

    Ok(stream)
}
