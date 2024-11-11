use anyhow::Result;
use lumos::config::{check_model_name, Config};

#[test]
fn test_config_from_file() -> Result<()> {
    // 直接从 keys.toml 读取
    let config_path = "keys.toml";

    // 读取配置文件并解析为 Config 结构体
    let config = Config::from_file(config_path)?;

    // 验证解析结果
    assert_eq!(config.models().len(), 4);
    assert!(config.models().contains_key("glm-4-plus"));
    assert!(config.models().contains_key("glm-4-long"));
    assert!(config.models().contains_key("deepseek-chat"));
    assert!(config.models().contains_key("qwen25-72b-instuct"));

    // 验证 check_model_name 函数
    assert!(check_model_name("glm-4-plus", config_path));
    assert!(!check_model_name("non-existent-model", config_path));

    Ok(())
}
