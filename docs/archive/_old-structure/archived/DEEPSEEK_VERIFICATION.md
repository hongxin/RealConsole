# Deepseek 模型验证指南

## 方法 1：使用 RealConsole 诊断（最简单）

### 步骤 1：设置 API Key

```bash
# 从 https://platform.deepseek.com 获取 API key
export DEEPSEEK_API_KEY="sk-your-api-key-here"

# 验证环境变量已设置
echo $DEEPSEEK_API_KEY
```

### 步骤 2：使用诊断命令

```bash
# 使用提供的测试配置
./target/debug/realconsole --config test-deepseek.yaml --once "/llm diag primary"
```

**成功输出示例：**
```
✓ Primary LLM: deepseek-chat (deepseek)
Primary LLM 诊断:
端点: https://api.deepseek.com/v1
模型: deepseek-chat
✓ API 连接正常
```

**失败输出示例：**
```
⚠ Primary LLM 初始化失败: Deepseek 需要 api_key
# 或
✗ API 连接失败: HTTP 401: Unauthorized
建议: 检查 API key 和网络连接
```

### 步骤 3：实际提问测试

```bash
# 简单提问测试
./target/debug/realconsole --config test-deepseek.yaml --once "/ask 你好"

# 或进入 REPL 模式
./target/debug/realconsole --config test-deepseek.yaml
> /ask 1+1等于几？
> /ask 用 Rust 写一个 Hello World
```

## 方法 2：使用 curl 直接测试 API（验证 API key）

### 测试 1：列出可用模型

```bash
curl https://api.deepseek.com/v1/models \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json"
```

**成功响应：**
```json
{
  "object": "list",
  "data": [
    {
      "id": "deepseek-chat",
      "object": "model",
      "created": 1234567890,
      "owned_by": "deepseek"
    }
  ]
}
```

**失败响应：**
```json
{
  "error": {
    "message": "Invalid API key",
    "type": "invalid_request_error",
    "code": "invalid_api_key"
  }
}
```

### 测试 2：简单对话测试

```bash
curl https://api.deepseek.com/v1/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-chat",
    "messages": [
      {
        "role": "user",
        "content": "你好，请说Hi"
      }
    ]
  }'
```

**成功响应：**
```json
{
  "id": "chatcmpl-xxxxx",
  "object": "chat.completion",
  "created": 1234567890,
  "model": "deepseek-chat",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hi！很高兴见到你！"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 15,
    "total_tokens": 25
  }
}
```

## 方法 3：使用 Rust 测试程序

创建 `test_deepseek.rs`：

```rust
use realconsole::llm::{DeepseekClient, LlmClient, Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取 API key
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .expect("请设置 DEEPSEEK_API_KEY 环境变量");

    // 创建客户端
    let client = DeepseekClient::new(
        api_key,
        "deepseek-chat",
        "https://api.deepseek.com/v1"
    )?;

    println!("✓ Deepseek 客户端创建成功");
    println!("模型: {}", client.model());

    // 诊断连接
    println!("\n诊断信息:");
    let diag = client.diagnose().await;
    println!("{}", diag);

    // 简单对话测试
    println!("\n对话测试:");
    let messages = vec![Message::user("1+1等于几？请只回答数字。")];

    match client.chat(messages).await {
        Ok(response) => {
            println!("✓ 对话成功");
            println!("响应: {}", response);
        }
        Err(e) => {
            eprintln!("✗ 对话失败: {}", e);
            return Err(Box::new(e));
        }
    }

    Ok(())
}
```

运行测试：
```bash
cargo run --example test_deepseek
```

## 常见错误及解决方案

### 错误 1：API key 未设置

**症状：**
```
⚠ Primary LLM 初始化失败: Deepseek 需要 api_key
```

**解决：**
```bash
export DEEPSEEK_API_KEY="sk-xxxxxxxx"
```

### 错误 2：API key 无效

**症状：**
```
✗ API 连接失败: HTTP 401: Unauthorized
```

**解决：**
1. 检查 API key 是否正确
2. 确认 API key 未过期
3. 登录 https://platform.deepseek.com 检查账号状态

### 错误 3：网络连接问题

**症状：**
```
✗ API 连接失败: Network error: ...
```

**解决：**
```bash
# 检查网络
ping api.deepseek.com

# 使用代理
export HTTPS_PROXY=http://127.0.0.1:7890
export HTTP_PROXY=http://127.0.0.1:7890
```

### 错误 4：速率限制

**症状：**
```
错误: Rate limit exceeded
```

**解决：**
- 等待一段时间后重试
- 检查账号的速率限制配置
- 考虑升级账号套餐

### 错误 5：区域限制

**症状：**
```
HTTP 403: Forbidden
```

**解决：**
- 使用 VPN 或代理
- 检查 Deepseek 的服务区域限制

## 完整验证流程

### 脚本化验证

创建 `verify-deepseek.sh`：

```bash
#!/bin/bash

set -e

echo "=== Deepseek 模型验证 ==="
echo ""

# 1. 检查环境变量
echo "1. 检查环境变量..."
if [ -z "$DEEPSEEK_API_KEY" ]; then
    echo "✗ DEEPSEEK_API_KEY 未设置"
    echo "请运行: export DEEPSEEK_API_KEY='sk-xxxxx'"
    exit 1
fi
echo "✓ DEEPSEEK_API_KEY 已设置: ${DEEPSEEK_API_KEY:0:10}..."
echo ""

# 2. 测试 API 连接
echo "2. 测试 API 连接..."
response=$(curl -s -w "\n%{http_code}" https://api.deepseek.com/v1/models \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json")

status_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | sed '$d')

if [ "$status_code" = "200" ]; then
    echo "✓ API 连接成功 (HTTP $status_code)"
    echo "可用模型:"
    echo "$body" | jq -r '.data[].id' 2>/dev/null || echo "  deepseek-chat"
else
    echo "✗ API 连接失败 (HTTP $status_code)"
    echo "$body"
    exit 1
fi
echo ""

# 3. 测试简单对话
echo "3. 测试简单对话..."
chat_response=$(curl -s https://api.deepseek.com/v1/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-chat",
    "messages": [{"role": "user", "content": "Hi"}]
  }')

if echo "$chat_response" | jq -e '.choices[0].message.content' > /dev/null 2>&1; then
    echo "✓ 对话测试成功"
    echo "响应: $(echo "$chat_response" | jq -r '.choices[0].message.content')"
else
    echo "✗ 对话测试失败"
    echo "$chat_response"
    exit 1
fi
echo ""

# 4. 使用 RealConsole 测试
echo "4. 使用 RealConsole 测试..."
if [ -f "./target/debug/realconsole" ]; then
    echo "运行诊断命令..."
    ./target/debug/realconsole --config test-deepseek.yaml --once "/llm diag primary"
    echo ""
    echo "运行提问测试..."
    ./target/debug/realconsole --config test-deepseek.yaml --once "/ask 1+1=?"
else
    echo "⚠ 未找到 realconsole 二进制文件"
    echo "请先运行: cargo build"
fi
echo ""

echo "=== 验证完成 ==="
echo "✓ Deepseek 模型可用"
```

运行验证：
```bash
chmod +x verify-deepseek.sh
./verify-deepseek.sh
```

## 性能测试

### 延迟测试

```bash
# 测试响应时间
time curl -s https://api.deepseek.com/v1/chat/completions \
  -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "deepseek-chat",
    "messages": [{"role": "user", "content": "Hi"}]
  }' > /dev/null
```

### 并发测试

```bash
# 10 个并发请求
for i in {1..10}; do
  (
    curl -s https://api.deepseek.com/v1/chat/completions \
      -H "Authorization: Bearer $DEEPSEEK_API_KEY" \
      -H "Content-Type: application/json" \
      -d '{
        "model": "deepseek-chat",
        "messages": [{"role": "user", "content": "Hi"}]
      }' > /dev/null
    echo "请求 $i 完成"
  ) &
done
wait
echo "所有请求完成"
```

## 监控和日志

### 启用详细日志

```bash
# 设置日志级别
export RUST_LOG=debug

# 运行 RealConsole
./target/debug/realconsole --config test-deepseek.yaml
```

### 查看统计信息

```bash
# 查看 LLM 统计（未来功能）
> /llm stats
Total calls: 10
Total success: 9
Total errors: 1
Average latency: 250ms
```

## 最佳实践

### 1. 安全存储 API Key

**不要：**
```yaml
# ❌ 直接写在配置文件中
api_key: sk-xxxxx
```

**应该：**
```yaml
# ✅ 使用环境变量
api_key: ${DEEPSEEK_API_KEY}
```

```bash
# .env 文件
DEEPSEEK_API_KEY=sk-xxxxx

# 添加到 .gitignore
echo ".env" >> .gitignore
```

### 2. 错误处理

配置重试策略：
```rust
RetryPolicy {
    max_attempts: 3,
    initial_backoff_ms: 800,
    max_backoff_ms: 8000,
    backoff_multiplier: 1.8,
}
```

### 3. 成本控制

```bash
# 估算 token 使用
echo "Prompt: 10 tokens"
echo "Response: ~50 tokens"
echo "Total: ~60 tokens"
echo "Cost: ~$0.0001 (按 Deepseek 定价)"
```

### 4. 备用方案

配置 fallback LLM：
```yaml
llm:
  primary:
    provider: deepseek
    # ...
  fallback:
    provider: ollama
    model: qwen3:4b
```

## 故障排除清单

- [ ] API key 已设置：`echo $DEEPSEEK_API_KEY`
- [ ] API key 有效：`curl https://api.deepseek.com/v1/models ...`
- [ ] 网络连通：`ping api.deepseek.com`
- [ ] 配置文件正确：`cat test-deepseek.yaml`
- [ ] RealConsole 已构建：`ls target/debug/realconsole`
- [ ] 诊断命令成功：`/llm diag primary`
- [ ] 简单提问成功：`/ask Hi`

## 总结

三种验证方法按优先级排序：

1. **RealConsole 诊断命令** ⭐⭐⭐
   - 最简单
   - 集成度高
   - 推荐日常使用

2. **curl 直接测试** ⭐⭐
   - 快速验证
   - 独立于应用
   - 适合调试

3. **Rust 测试程序** ⭐
   - 完整控制
   - 适合开发
   - 需要编译

开始验证：
```bash
# 快速验证（推荐）
export DEEPSEEK_API_KEY="sk-xxxxx"
./target/debug/realconsole --config test-deepseek.yaml --once "/llm diag primary"
```

祝验证成功！🚀
