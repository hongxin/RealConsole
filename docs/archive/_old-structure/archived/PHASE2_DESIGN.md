# Phase 2: LLM 集成 - 设计文档

## 设计哲学

**参考 Python 成功经验 + 发挥 Rust 优势**

### Python 版本精华
1. ✅ **统一接口** - Protocol/ABC 清晰
2. ✅ **重试机制** - 指数退避 + 抖动
3. ✅ **统计收集** - 可观测性强
4. ✅ **错误处理** - 分类清晰
5. ✅ **降级策略** - Ollama 双接口

### Rust 独特优势
1. 🚀 **trait 系统** - 比 Protocol 更强大
2. 🛡️ **类型安全** - 编译时保证
3. ⚡ **async/await** - 高性能异步
4. 🔒 **线程安全** - Arc + Atomic
5. 💎 **零成本抽象** - 无运行时开销

---

## 核心架构

### 1. LlmClient Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 核心聊天接口
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError>;

    /// 获取模型名称
    fn model(&self) -> &str;

    /// 获取统计信息
    fn stats(&self) -> ClientStats;

    /// 诊断连接
    async fn diagnose(&self) -> String;
}
```

**优势**:
- `async_trait` - 支持异步方法
- `Send + Sync` - 跨线程安全
- `Result<T, E>` - 显式错误处理

### 2. 错误处理（thiserror）

```rust
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("HTTP {status}: {message}")]
    Http { status: u16, message: String },

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Timeout after {0:?}")]
    Timeout(Duration),

    #[error("Parse error: {0}")]
    Parse(String),
}
```

**优势**:
- 类型安全的错误
- 自动实现 Display + Error
- 模式匹配友好

### 3. 消息结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}
```

**优势**:
- 类型安全
- serde 自动序列化
- 枚举约束

### 4. 统计系统（线程安全）

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ClientStats {
    total_calls: Arc<AtomicU64>,
    total_retries: Arc<AtomicU64>,
    total_errors: Arc<AtomicU64>,
}

impl ClientStats {
    pub fn record_call(&self) {
        self.total_calls.fetch_add(1, Ordering::Relaxed);
    }

    pub fn total_calls(&self) -> u64 {
        self.total_calls.load(Ordering::Relaxed)
    }
}
```

**优势**:
- 无锁并发（Atomic）
- 零成本（Arc只在clone时增加引用计数）
- 线程安全

### 5. 重试策略

```rust
pub struct RetryPolicy {
    max_attempts: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    backoff_multiplier: f64,
    retryable_status: HashSet<u16>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        let mut retryable = HashSet::new();
        retryable.extend([429, 500, 502, 503, 504]);

        Self {
            max_attempts: 3,
            initial_backoff_ms: 800,
            max_backoff_ms: 8000,
            backoff_multiplier: 1.8,
            retryable_status: retryable,
        }
    }
}
```

**优势**:
- 可配置
- 类型安全
- 清晰的语义

---

## 客户端实现

### Ollama Client

**特色功能**:
1. 双接口降级 (native → OpenAI compatible)
2. 模型列表缓存
3. <think> 标签过滤
4. 离线模型回退

```rust
pub struct OllamaClient {
    endpoint: String,
    model: String,
    client: Client,
    stats: ClientStats,
    retry_policy: RetryPolicy,
    model_cache: Arc<Mutex<Option<Vec<String>>>>,
}
```

### Deepseek Client

**特色功能**:
1. Bearer Token 认证
2. 标准 OpenAI API
3. 速率限制处理

```rust
pub struct DeepseekClient {
    endpoint: String,
    model: String,
    api_key: String,
    client: Client,
    stats: ClientStats,
    retry_policy: RetryPolicy,
}
```

---

## 重试机制实现

```rust
async fn retry_with_backoff<F, Fut, T>(
    policy: &RetryPolicy,
    mut operation: F,
) -> Result<T, LlmError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, LlmError>>,
{
    let mut backoff_ms = policy.initial_backoff_ms;

    for attempt in 1..=policy.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt == policy.max_attempts {
                    return Err(e);
                }

                // 检查是否可重试
                if !is_retryable(&e, policy) {
                    return Err(e);
                }

                // 指数退避 + 抖动
                let jitter = rand::random::<u64>() % 100;
                tokio::time::sleep(
                    Duration::from_millis(backoff_ms + jitter)
                ).await;

                backoff_ms = (backoff_ms as f64 * policy.backoff_multiplier) as u64;
                backoff_ms = backoff_ms.min(policy.max_backoff_ms);
            }
        }
    }

    unreachable!()
}
```

---

## 集成到 Agent

### 1. 扩展 Config

```rust
pub struct LlmConfig {
    pub primary: Option<LlmProviderConfig>,
    pub fallback: Option<LlmProviderConfig>,
}

pub struct LlmProviderConfig {
    pub provider: String,  // "ollama", "deepseek", "openai"
    pub model: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
}
```

### 2. 工厂模式

```rust
pub fn create_llm_client(
    config: &LlmProviderConfig
) -> Result<Box<dyn LlmClient>, LlmError> {
    match config.provider.as_str() {
        "ollama" => Ok(Box::new(OllamaClient::new(config)?)),
        "deepseek" => Ok(Box::new(DeepseekClient::new(config)?)),
        "openai" => Ok(Box::new(OpenAIClient::new(config)?)),
        _ => Err(LlmError::Parse("Unknown provider".into())),
    }
}
```

### 3. 新增命令

```rust
// /ask - LLM 对话
fn cmd_ask(arg: &str) -> String {
    // 异步调用 LLM
}

// /llm - LLM 管理
fn cmd_llm(arg: &str) -> String {
    // status, switch, stats
}
```

---

## 测试策略

### 1. 单元测试

```rust
#[tokio::test]
async fn test_ollama_chat() {
    let client = OllamaClient::new_with_defaults();
    let messages = vec![
        Message::user("Hello"),
    ];
    let result = client.chat(messages).await;
    assert!(result.is_ok());
}
```

### 2. 模拟测试（mockito）

```rust
#[tokio::test]
async fn test_retry_on_500() {
    let mock = mockito::mock("POST", "/v1/chat/completions")
        .with_status(500)
        .with_body("Internal Error")
        .create();

    // 测试重试逻辑
}
```

### 3. 集成测试

```rust
#[tokio::test]
#[ignore]  // 需要真实服务
async fn test_ollama_integration() {
    // 真实 Ollama 测试
}
```

---

## 依赖更新

```toml
[dependencies]
# 已有
tokio = { version = "1.40", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# 新增
async-trait = "0.1"
thiserror = "1.0"
rand = "0.8"

[dev-dependencies]
mockito = "1.0"
tokio-test = "0.4"
```

---

## 实施计划

### Week 1: 基础架构
- [x] 设计文档
- [ ] LlmClient trait
- [ ] 错误类型
- [ ] 消息结构
- [ ] 统计系统

### Week 2: Ollama
- [ ] OllamaClient 实现
- [ ] 双接口降级
- [ ] 模型管理
- [ ] 单元测试

### Week 3: Deepseek
- [ ] DeepseekClient 实现
- [ ] 认证处理
- [ ] 重试机制
- [ ] 单元测试

### Week 4: 集成
- [ ] Agent 集成
- [ ] /ask 命令
- [ ] /llm 命令
- [ ] 文档更新

---

## 性能目标

| 指标 | 目标 | Python 对比 |
|------|------|------------|
| 首次请求延迟 | < 100ms | ~= |
| 并发请求 | 100+ QPS | 10x |
| 内存占用 | < 10 MB | 8x |
| CPU 占用 | < 5% | 5x |

---

## 风险与缓解

### 风险 1: 异步复杂度
**缓解**:
- 使用 tokio::test
- 简化异步边界
- 充分测试

### 风险 2: 错误处理
**缓解**:
- 使用 thiserror
- 清晰的错误类型
- 完整的错误传播

### 风险 3: 兼容性
**缓解**:
- 参考 Python 实现
- 保持 API 一致
- 充分集成测试

---

## 成功标准

1. ✅ 所有测试通过
2. ✅ Clippy 无警告
3. ✅ 性能达标
4. ✅ 文档完整
5. ✅ 与 Python 版本功能对等

---

**文档版本**: v0.2.0
**作者**: RealConsole Team
**日期**: 2025-10-13
