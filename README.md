# lumos
Ollama 代理，通过 Ollama 接口，调用 GLM、Deepseek、Xinference 等后台大模型服务

## 使用说明

### 基本用法

```bash
lumos --help

Usage: lumos [OPTIONS] <MODEL_NAME>

Arguments:
  <MODEL_NAME>  模型名称

Options:
      --host <HOST>                服务器地址[default: localhost]
  -p, --port <PORT>                服务端口[default: 11434]
  -c, --config-file <CONFIG_FILE>  配置文件路径[default: keys.toml]
  -h, --help                       帮助
  -V, --version                    版本
```

### 配置文件
配置文件例子如下：
```toml
[glm4-plus]
model_name = "glm-4-plus"
provider = "zhipu"
url = "https://open.bigmodel.cn/api/paas/v4/chat/completions"
api_key = ""

[deepseek]
model_name = "deepseek-chat"
provider = "deepseek"
url = "https://api.deepseek.com/chat/completions"
api_key = ""

[qwen25-32b]
model_name = "Qwen2.5-32B-Instruct"
provider = "xinference"
url = "https://inference.top/api/v1/chat/completions"
api_key = ""
```
[glm4-plus] 是模型的一个别名，可以自定义，model_name 是模型名称，provider 是模型提供商，url 是模型服务地址，api_key 是模型服务的 API Key。
model_name 一定要和后台大模型服务能够支持的模型名称一致，而且区分大小写

启动的时候，可以通过 `--config-file` 参数指定配置文件路径，如果不指定，默认会读取当前目录下的 `keys.toml` 文件

启动例子：
```bash
./lumos glm4-plus -c ./config/models.toml

2024-11-11T00:10:48.788438Z  INFO lumos: listening on localhost:11434
```

因为 ollama 默认是启动在 localhost:11434。但是有些应用调用 Ollama 可能在 127.0.0.1，所以可以通过 `--host` 和 `--port` 参数指定 ollama 的地址和端口
```bash
./lumos glm4-plus --host 127.0.0.1 --port 11434 -c ./config/models.toml

2024-11-11T00:13:30.026509Z  INFO lumos: listening on 127.0.0.1:11434
```
