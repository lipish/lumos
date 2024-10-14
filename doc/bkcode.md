```rust
use crate::define::{ChatRequest, Provider, ProviderName};
use anyhow::Result;
use async_trait::async_trait;
use axum::response::sse::Event;
use axum::response::{IntoResponse, Sse};
use eventsource_stream::Eventsource;
use futures::stream::{self, Stream};
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest::header;
use serde_json::json;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

enum ChatService {
    GLM(Provider),
    DeepSeek(Provider),
}

// Add ChatService trait
#[async_trait]
trait ChatServiceTrait: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<impl IntoResponse + '_, anyhow::Error>;
    fn get_provider(&self) -> &Provider;
}

impl ChatServiceTrait for ChatService {
    async fn chat(&self, request: ChatRequest) -> Result<impl IntoResponse + '_, anyhow::Error> {
        let (tx, rx) = mpsc::channel(1024);
        let provider = self.get_provider().clone();

        tokio::spawn(async move {
            let mut headers = header::HeaderMap::new();

            // get api key
            headers.insert(
                "Authorization",
                format!("Bearer {}", provider.api_key).parse().unwrap(),
            );

            // Send request to the provider API
            let mut stream = match &*self {
                ChatService::GLM(_) => {
                    // GLM的实现逻辑
                    todo!("实现GLM的流式请求")
                }
                ChatService::DeepSeek(_) => {
                    let client = reqwest::Client::new();
                    let request_body = json!({
                        "messages": request.messages,
                        "model": "deepseek-chat",
                        "stream": true,
                        "max_tokens": 4096,
                    });

                    client
                        .post(&provider.url)
                        .headers(headers)
                        .json(&request_body)
                        .send()
                        .await
                        .unwrap()
                        .bytes_stream()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
                        .eventsource()
                }
            };

            while let Some(event) = stream.next().await {
                match event {
                    Ok(event) => {
                        let axum_event = Event::default().data(event.data);
                        if tx.send(Ok(axum_event)).await.is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        if tx.send(Err(axum::Error::new(err))).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });

        Ok(Sse::new(ReceiverStream::new(rx)))
    }

    fn get_provider(&self) -> &Provider {
        match self {
            ChatService::GLM(provider) => provider,
            ChatService::DeepSeek(provider) => provider,
        }
    }
}

impl ChatService {
    pub fn new(provider: Provider) -> Self {
        match provider.name {
            ProviderName::GLM => ChatService::GLM(provider),
            ProviderName::DeepSeek => ChatService::DeepSeek(provider),
        }
    }

    pub async fn handle_chat(
        self: Arc<Self>,
        request: ChatRequest,
    ) -> Result<impl IntoResponse, anyhow::Error> {
        self.chat(request).await
    }
}

// 添加这个实现
unsafe impl Send for ChatService {}
unsafe impl Sync for ChatService {}

```