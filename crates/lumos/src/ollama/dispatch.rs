use anyhow::Result;
use async_stream::try_stream;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use bytes::BytesMut;
use chrono::Utc;
use futures_util::stream::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::structs::config::Model;
use crate::structs::ollama::{ChatType, Message};

pub async fn dispatch(
    model: &str,
    messages: Vec<Message>,
    provider: &Model,
    chat_type: ChatType,
) -> Result<impl IntoResponse, anyhow::Error> {
    let stream = send(model.to_string(), messages, provider, chat_type).await?;
    let body = Body::from_stream(stream);
    let response = Response::builder()
        .header("Content-Type", "text/plain")
        .body(body)
        .unwrap();
    Ok(response)
}

async fn send(
    model: String,
    messages: Vec<Message>,
    provider: &Model,
    chat_type: ChatType,
) -> Result<impl Stream<Item = Result<String, anyhow::Error>> + Unpin + Send, anyhow::Error> {
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

    let client = Client::new();

    let request_body = json!({
        "model": model,
        "messages": messages,
        "stream": true
    });

    // 将模型名称中的 "-" 替换为 ":"
    let model = model.replacen('-', ":", 1);

    let response = client
        .post(&provider.url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        let error_message = response.text().await?;
        return Err(anyhow::anyhow!("API请求失败: {}:{}", status, error_message));
    }

    let done_flag = Arc::new(AtomicBool::new(false));
    let done_flag_clone = done_flag.clone();
    let model_clone = model.clone();
    let chat_type_clone = chat_type.clone();

    let stream = try_stream! {
        let mut buf = BytesMut::new();
        let mut stream_bytes = response.bytes_stream();

        while let Some(result) = stream_bytes.next().await {
            let bytes = result?;
            buf.extend_from_slice(&bytes);

            while let Some(position) = buf.windows(2).position(|window| window == b"\n\n") {
                let line_bytes = buf.split_to(position + 2);
                let line = String::from_utf8_lossy(&line_bytes).trim().to_string();
                if !line.is_empty() {
                    if let Some(content) = process_line(&line, &model_clone, &chat_type_clone, &done_flag_clone) {
                        // trim \n\n from the start or end of the content and add \n\n to the end of the content
                        let mut content_with_newline = content.clone();
                        content_with_newline = content_with_newline.trim_start_matches("\n\n").to_string();
                        content_with_newline.push_str("\n");
                        yield content_with_newline;
                    }
                }
            }

            if done_flag_clone.load(Ordering::SeqCst) {
                // contruct a chat message
                // this is zed.dev format, not in ollama format
                let message = json!({
                    "role": "assistant",
                    "content": "",
                    "images": null
                });

                let done = json!({
                    "model": model_clone,
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "response": "",
                    "message": message,
                    "done": true,
                    "context": [1, 2, 3],
                    "total_duration": 122112,
                    "load_duration": 123112,
                    "prompt_eval_count": 26,
                    "prompt_eval_duration": 130079000,
                    "eval_count": 259,
                    "eval_duration": 2433122
                });
                // trim \n\n from the start or end of the content and add \n\n to the end of the content
                let mut done_with_newline = done.to_string();
                done_with_newline = done_with_newline.trim_start_matches("\n\n").to_string();
                done_with_newline.push_str("\n");
                yield done_with_newline;
                break;
            }
        }
    };

    Ok(Box::pin(stream))
}

fn process_line(
    line: &str,
    model: &str,
    chat_type: &ChatType,
    done_flag: &Arc<AtomicBool>,
) -> Option<String> {
    if line.trim() == "data: [DONE]" {
        done_flag.store(true, Ordering::SeqCst);
        None
    } else if line.starts_with("data: ") {
        let json_str = line.trim_start_matches("data: ").trim();
        match serde_json::from_str::<Value>(json_str) {
            Ok(json) => {
                let content = json["choices"][0]["delta"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                if !content.is_empty() {
                    let mut json_content = json!({
                        "model": model,
                        "created_at": Utc::now().to_rfc3339(),
                        "done": false
                    });

                    if *chat_type == ChatType::Chat {
                        json_content["message"] = json!({
                            "role": "assistant",
                            "content": content,
                            "images": null
                        });
                    } else {
                        json_content["response"] = json!(content);
                    }

                    Some(json_content.to_string())
                } else {
                    None
                }
            }
            Err(e) => {
                eprintln!("JSON解析错误: {}", e);
                None
            }
        }
    } else {
        None
    }
}
