use anyhow::Result;
use axum::{
    body::Body,
    response::{IntoResponse, Response},
};

use crate::structs::config::Provider;
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
    fn_content: Option<Arc<dyn Fn(&str) -> Result<String, anyhow::Error> + Send + Sync>>,
    fn_done: Option<Arc<dyn Fn() -> Result<(), anyhow::Error> + Send + Sync>>,
) -> Result<impl IntoResponse, anyhow::Error> {
    // Send request to the provider service and get the stream
    let stream = send(model, messages, provider, fn_content, fn_done).await?;

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
    model: &str,
    messages: Vec<Message>,
    provider: &Provider,
    fn_content: Option<Arc<dyn Fn(&str) -> Result<String, anyhow::Error> + Send + Sync>>,
    fn_done: Option<Arc<dyn Fn() -> Result<(), anyhow::Error> + Send + Sync>>,
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
    let model = &model;

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
            let fn_content = fn_content.clone();
            let fn_done = fn_done.clone();
            let done_flag = done_flag_clone.clone();

            result.and_then(|content| {
                if done_flag.load(Ordering::SeqCst) {
                    if let Some(ref fn_done) = fn_done {
                        fn_done().map(|_| "[DONE]".to_string())
                    } else {
                        Ok("[DONE]".to_string())
                    }
                } else if let Some(ref fn_content) = fn_content {
                    fn_content(&content)
                } else {
                    Ok(content)
                }
            })
        })
        .take_while(|result| {
            futures_util::future::ready(result.as_ref().map_or(true, |s| s != "[DONE]"))
        });

    Ok(stream)
}
