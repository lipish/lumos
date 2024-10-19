use anyhow::Context;
use futures_util::StreamExt;
use lumos::config::Config;
use lumos::define::{ChatMessage, ChatRequest};
use lumos::service::send;

#[tokio::test]
async fn test_models() -> Result<(), anyhow::Error> {
    let config_path = "keys.toml";
    let config = Config::from_file(config_path)?;

    let test_cases = vec![
        ("deepseek-chat", "Where is the capital of China?", "Beijing"),
        ("glm-4-plus", "who is the boss of SpaceX?", "Elon Musk"),
    ];

    for (model_name, prompt, expected_substring) in test_cases {
        let provider = config
            .models
            .get(model_name)
            .with_context(|| format!("未找到模型提供者: {}", model_name))?;

        let req = ChatRequest {
            model: model_name.to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
                images: None,
                tool_calls: None,
            }],
            stream: true,
            ..Default::default()
        };

        let mut stream = send(req, provider).await?;

        let mut collected_chunks = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    println!("Model: {}, Received chunk: {}", model_name, chunk);
                    collected_chunks.push(chunk);
                }
                Err(err) => {
                    println!("Model: {}, Error reading from stream: {}", model_name, err);
                    return Err(err);
                }
            }
        }

        let reply_string = collected_chunks.join("");

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
