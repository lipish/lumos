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

    println!("request_body:{}", request_body);
    println!("provider.url:{}", provider.url);
    println!("api_key:{}", api_key);

    let response = client
        .post(&provider.url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;

    println!("response.status:{}", response.status());

    if !response.status().is_success() {
        return Err(anyhow::anyhow!("API请求失败: {}", response.status()));
    }

    let done_flag = Arc::new(AtomicBool::new(false));
    let done_flag_clone = done_flag.clone();

    let buffer = String::new();
    let stream = response
        .bytes_stream()
        .map(|result| -> Result<String, anyhow::Error> {
            let bytes = result.map_err(anyhow::Error::from)?;
            let text = String::from_utf8_lossy(&bytes).to_string();
            Ok(text)
        })
        .flat_map(move |line_result| {
            let done_flag = done_flag.clone();
            let buffer = buffer.clone();
            futures_util::stream::iter(line_result.map(move |line| {
                let mut buffer = buffer.clone();
                buffer.push_str(&line);

                if buffer.ends_with("\n\n") {
                    // remove the last "\n\n"
                    buffer.pop();
                    buffer.pop();

                    let full_line = std::mem::take(&mut buffer);

                    // 将full_line按"\n\n"分割成多个数据块
                    let data_blocks: Vec<&str> = full_line.split("\n\n").collect();

                    for block in data_blocks {
                        let trimmed_block = block.trim();
                        if trimmed_block == "data: [DONE]" {
                            done_flag.store(true, Ordering::SeqCst);
                        } else if trimmed_block.starts_with("data: ") {
                            let json_str = trimmed_block.trim_start_matches("data: ");
                            match serde_json::from_str::<Value>(json_str) {
                                Ok(json) => {
                                    let content = json["choices"][0]["delta"]["content"]
                                        .as_str()
                                        .unwrap_or("")
                                        .to_string();
                                    if !content.is_empty() {
                                        return Ok(Some(content));
                                    }
                                }
                                Err(e) => {
                                    println!("JSON解析错误: {}", e);
                                    // ignore the error
                                }
                            }
                        } else {
                            println!("未知数据块: {}", trimmed_block);
                        }
                    }
                    // 如果所有块都处理完毕但没有返回内容
                    if done_flag.load(Ordering::SeqCst) {
                        Ok(Some("".to_string())) // 保持流活跃
                    } else {
                        Ok(None) // 没有内容可返回
                    }
                } else {
                    Ok(None)
                }
            }))
        })
        .filter_map(|result| futures_util::future::ready(result.transpose())) // filter out the empty string and flatten the result and option, keep the async feature
        .map(move |result| {
            let done_flag = done_flag_clone.clone();

            result.and_then(|content| {
                if done_flag.load(Ordering::SeqCst) {
                    // end of stream
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
                    println!("done_json:{}", done_json);

                    // add "\n" to the end of the string
                    let mut done_json_str = done_json.to_string();
                    done_json_str.push_str("\n");

                    Ok(done_json_str)
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

                    println!("json_content:{}", json_content);

                    // add "\n" to the end of the string
                    let mut json_content_str = json_content.to_string();
                    json_content_str.push_str("\n");

                    Ok(json_content_str)
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
