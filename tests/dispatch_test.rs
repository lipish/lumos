use anyhow::Context;
use axum::body::Body;
use axum::response::IntoResponse;
use axum::response::Response;
use futures_util::StreamExt;
use lumos::config::Config;
use lumos::ollama::dispatch;
use lumos::structs::ollama::ChatType;
use lumos::structs::ollama::{ChatRequest, Message};

#[tokio::test]
async fn test_dispatch() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "keys.toml";
    let config = Config::from_file(config_path)
        .context("无法加载配置文件")
        .map_err(|e| axum::Error::new(e))?;

    let test_cases = vec![
        ("deepseek-chat", "Where is the capital of China?", "Beijing"),
        ("glm-4-plus", "who is the boss of SpaceX?", "Elon Musk"),
    ];

    for (model_name, prompt, expected_substring) in test_cases {
        let provider = config
            .models
            .get(model_name)
            .context(format!("未找到模型提供者: {}", model_name))?;

        let req = ChatRequest {
            model: model_name.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
                images: None,
                tool_calls: None,
            }],
            stream: true,
            ..Default::default()
        };

        let response: Response<Body> = dispatch(model_name, req.messages, provider, ChatType::Chat)
            .await
            .map_err(|e| axum::Error::new(e))?
            .into_response();
        let mut stream = response.into_body().into_data_stream();

        let mut collected_chunks = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    println!("Model: {}, Received chunk: {:?}", model_name, chunk);
                    collected_chunks.push(chunk);
                }
                Err(err) => {
                    println!("Model: {}, Error reading from stream: {}", model_name, err);
                    return Err(err.into());
                }
            }
        }

        let reply_string = collected_chunks
            .iter()
            .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
            .collect::<String>();

        assert!(
            reply_string.contains(expected_substring),
            "Model {} did not return expected substring '{}'. Actual reply: {}",
            model_name,
            expected_substring,
            reply_string
        );

        assert!(collected_chunks.len() > 0);
    }

    Ok(())
}
