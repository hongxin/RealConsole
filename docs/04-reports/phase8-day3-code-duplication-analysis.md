# Phase 8 Day 3 - LLM 客户端代码重复分析

**日期**: 2025-10-16
**任务**: 分析 DeepseekClient 和 OllamaClient 的代码重复情况

## 📊 代码统计

| 文件 | 行数 | 测试行数 | 核心代码行数 |
|------|------|---------|-------------|
| deepseek.rs | 608 | 243 | ~365 |
| ollama.rs | 519 | 224 | ~295 |
| mod.rs | 488 | 92 | ~396 |
| **总计** | **1615** | **559** | **1056** |

## 🔍 重复代码分析

### 1. HTTP 客户端配置（100% 重复）

**Deepseek** (deepseek.rs:41-44):
```rust
let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .build()
    .map_err(|e| LlmError::Config(e.to_string()))?;
```

**Ollama** (ollama.rs:34-37):
```rust
let client = Client::builder()
    .timeout(Duration::from_secs(60))
    .build()
    .map_err(|e| LlmError::Config(e.to_string()))?;
```

**重复度**: 100% 相同（4 行）

---

### 2. Endpoint 规范化（100% 重复）

**Deepseek** (deepseek.rs:38-39):
```rust
let endpoint = endpoint.into();
let endpoint = endpoint.trim_end_matches('/').to_string();
```

**Ollama** (ollama.rs:31-32):
```rust
let endpoint = endpoint.into();
let endpoint = endpoint.trim_end_matches('/').to_string();
```

**重复度**: 100% 相同（2 行）

---

### 3. 字段初始化（95% 重复）

**Deepseek** (deepseek.rs:46-53):
```rust
Ok(Self {
    endpoint,
    model: model.into(),
    api_key,  // 额外字段
    client,
    stats: ClientStats::new(),
    retry_policy: RetryPolicy::default(),
})
```

**Ollama** (ollama.rs:39-46):
```rust
Ok(Self {
    endpoint,
    model: model.into(),
    client,
    stats: ClientStats::new(),
    retry_policy: RetryPolicy::default(),
    model_cache: Arc::new(Mutex::new(None)),  // 额外字段
})
```

**重复度**: 95% 相似（5/7 字段相同）

---

### 4. 重试逻辑（90% 重复）

**Deepseek** (deepseek.rs:105-134):
```rust
async fn chat_with_retry(&self, messages: &[Message]) -> Result<String, LlmError> {
    let mut last_error = None;

    for attempt in 1..=self.retry_policy.max_attempts {
        match self.chat_once(messages).await {
            Ok(response) => {
                if attempt > 1 {
                    self.stats.record_retry();
                }
                return Ok(response);
            }
            Err(e) => {
                last_error = Some(e.clone());

                if attempt < self.retry_policy.max_attempts
                    && self.retry_policy.is_retryable(&e)
                {
                    let backoff = self.retry_policy.backoff_duration(attempt);
                    tokio::time::sleep(backoff).await;
                    continue;
                } else {
                    break;
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| LlmError::Other("Unknown error".into())))
}
```

**Ollama** (ollama.rs:216-242) - Native API fallback 部分:
```rust
// 尝试 Native API（fallback）
let mut last_error = None;
for attempt in 1..=self.retry_policy.max_attempts {
    match self.chat_native(messages).await {
        Ok(response) => {
            if attempt > 1 {
                self.stats.record_retry();
            }
            return Ok(Self::strip_think_tags(&response));
        }
        Err(e) => {
            last_error = Some(e.clone());

            if attempt < self.retry_policy.max_attempts
                && self.retry_policy.is_retryable(&e)
            {
                let backoff = self.retry_policy.backoff_duration(attempt);
                tokio::time::sleep(backoff).await;
                continue;
            } else {
                break;
            }
        }
    }
}

Err(last_error.unwrap_or_else(|| LlmError::Other("Unknown error".into())))
```

**重复度**: 90% 相似（30 行，结构完全相同，只有调用的函数名不同）

---

### 5. HTTP 错误处理（100% 重复）

**Deepseek** (deepseek.rs:80-87):
```rust
let status = resp.status();
if !status.is_success() {
    let error_text = resp.text().await.unwrap_or_default();
    return Err(LlmError::Http {
        status: status.as_u16(),
        message: error_text,
    });
}
```

**Ollama** (ollama.rs:145-150, 175-180):
```rust
if !resp.status().is_success() {
    return Err(LlmError::Http {
        status: resp.status().as_u16(),
        message: resp.text().await.unwrap_or_default(),
    });
}
```

**重复度**: 100% 相同逻辑（出现 3+ 次）

---

### 6. JSON 响应解析（80% 重复）

**Deepseek** (deepseek.rs:89):
```rust
let data: Value = resp.json().await.map_err(|e| LlmError::Parse(e.to_string()))?;
```

**Ollama** (ollama.rs:152, 182):
```rust
let data: Value = resp.json().await.map_err(|e| LlmError::Parse(e.to_string()))?;
```

**重复度**: 100% 相同（出现 3+ 次）

---

### 7. LlmClient trait chat() 实现（95% 重复）

**Deepseek** (deepseek.rs:216-228):
```rust
async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
    self.stats.record_call();

    match self.chat_with_retry(&messages).await {
        Ok(response) => {
            self.stats.record_success();
            Ok(response)
        }
        Err(e) => {
            self.stats.record_error();
            Err(e)
        }
    }
}
```

**Ollama** (ollama.rs:247-259):
```rust
async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
    self.stats.record_call();

    match self.chat_with_retry(&messages).await {
        Ok(response) => {
            self.stats.record_success();
            Ok(response)
        }
        Err(e) => {
            self.stats.record_error();
            Err(e)
        }
    }
}
```

**重复度**: 100% 完全相同（13 行）

---

### 8. model() 和 stats() 实现（100% 重复）

**Deepseek** (deepseek.rs:334-340):
```rust
fn model(&self) -> &str {
    &self.model
}

fn stats(&self) -> ClientStats {
    self.stats.clone()
}
```

**Ollama** (ollama.rs:262-268):
```rust
fn model(&self) -> &str {
    &self.model
}

fn stats(&self) -> ClientStats {
    self.stats.clone()
}
```

**重复度**: 100% 完全相同（7 行）

---

### 9. HTTP POST 请求模式（90% 重复）

**Deepseek** (deepseek.rs:71-78):
```rust
let resp = self
    .client
    .post(&url)
    .header("Authorization", format!("Bearer {}", self.api_key))  // 差异
    .header("Content-Type", "application/json")
    .json(&payload)
    .send()
    .await?;
```

**Ollama** (ollama.rs:143, 173):
```rust
let resp = self.client.post(&url).json(&payload).send().await?;
```

**重复度**: 80% 相似（除了认证 header）

---

## 📈 重复度汇总

| 类别 | 行数（估算） | 重复度 | 优先级 |
|------|-------------|--------|-------|
| HTTP 客户端配置 | ~15 | 100% | P0 |
| 重试逻辑 | ~60 | 90% | P0 |
| HTTP 错误处理 | ~21 | 100% | P0 |
| 统计记录 | ~20 | 95% | P0 |
| trait 实现 | ~40 | 95% | P0 |
| JSON 解析 | ~10 | 100% | P1 |
| 请求构建 | ~30 | 85% | P1 |
| **总重复代码** | **~196 行** | **~92%** | - |

**估算重复率**:
- 核心代码: ~660 行（deepseek 365 + ollama 295）
- 重复代码: ~196 行
- **重复率: 196/660 ≈ 30% 直接重复，考虑逻辑相似度约 60-70%**

---

## 🎯 重构目标

### 目标 1: 提取公共 HTTP 客户端层

创建 `HttpClientBase` 结构，包含：

```rust
pub struct HttpClientBase {
    pub client: Client,
    pub endpoint: String,
    pub stats: ClientStats,
    pub retry_policy: RetryPolicy,
}

impl HttpClientBase {
    // 1. 通用构造
    pub fn new(endpoint: String, timeout_secs: u64) -> Result<Self, LlmError>;

    // 2. 通用 HTTP POST
    pub async fn post_json(
        &self,
        url: &str,
        payload: Value,
        headers: Option<HeaderMap>,
    ) -> Result<Response, LlmError>;

    // 3. 通用重试逻辑
    pub async fn with_retry<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, LlmError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, LlmError>>;

    // 4. 通用错误处理
    pub async fn handle_response(resp: Response) -> Result<Value, LlmError>;

    // 5. 统计包装器
    pub async fn record_operation<F, Fut, T>(
        &self,
        operation: F,
    ) -> Result<T, LlmError>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, LlmError>>;
}
```

### 目标 2: 简化客户端实现

**重构后的 DeepseekClient**:
```rust
pub struct DeepseekClient {
    base: HttpClientBase,  // 复用公共层
    model: String,
    api_key: String,
}

impl DeepseekClient {
    pub fn new(...) -> Result<Self, LlmError> {
        Ok(Self {
            base: HttpClientBase::new(endpoint, 60)?,
            model: model.into(),
            api_key: api_key.into(),
        })
    }

    async fn chat_once(&self, messages: &[Message]) -> Result<String, LlmError> {
        let url = format!("{}/chat/completions", self.base.endpoint);
        let payload = json!({
            "model": self.model,
            "messages": messages,
        });

        // 使用公共层
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", format!("Bearer {}", self.api_key).parse().unwrap());

        let data = self.base.post_json(&url, payload, Some(headers)).await?;

        // 提取响应（简化的业务逻辑）
        if let Some(content) = data["choices"][0]["message"]["content"].as_str() {
            Ok(content.to_string())
        } else {
            Ok(data.to_string())
        }
    }
}

#[async_trait]
impl LlmClient for DeepseekClient {
    async fn chat(&self, messages: Vec<Message>) -> Result<String, LlmError> {
        // 使用公共层的统计和重试
        self.base.record_operation(|| async {
            self.base.with_retry(|| self.chat_once(&messages)).await
        }).await
    }

    fn stats(&self) -> ClientStats {
        self.base.stats.clone()  // 直接使用 base 的 stats
    }
}
```

---

## 📊 预期改进

| 指标 | 重构前 | 重构后 | 改进 |
|------|--------|--------|------|
| DeepseekClient 行数 | ~365 | ~180 | -51% |
| OllamaClient 行数 | ~295 | ~160 | -46% |
| 代码重复率 | ~70% | <30% | -57% |
| HttpClientBase 行数 | 0 | ~200 | 新增 |
| 总代码行数 | ~660 | ~540 | -18% |
| 测试复用度 | 低 | 高 | +80% |

---

## 🚀 实施计划

### 阶段 1: 创建 HttpClientBase（2小时）

1. 创建 `src/llm/http_base.rs` 文件
2. 实现通用 HTTP 客户端配置
3. 实现通用重试逻辑
4. 实现通用错误处理
5. 实现统计包装器
6. 添加单元测试

### 阶段 2: 重构 DeepseekClient（1.5小时）

1. 修改 DeepseekClient 使用 HttpClientBase
2. 简化 chat_once 实现
3. 移除重复的重试逻辑
4. 移除重复的统计代码
5. 验证所有测试通过

### 阶段 3: 重构 OllamaClient（1.5小时）

1. 修改 OllamaClient 使用 HttpClientBase
2. 保留特殊逻辑（双接口降级、think 标签）
3. 移除重复代码
4. 验证所有测试通过

### 阶段 4: 验证和优化（1小时）

1. 运行完整测试套件
2. 性能基准测试（确保无退化）
3. 代码重复率检查（目标 < 30%）
4. 文档更新

---

## 🔍 技术难点

### 难点 1: 异步闭包的生命周期

**问题**: `with_retry` 需要接受异步闭包，Rust 的异步闭包生命周期复杂

**方案**: 使用 trait bound + 泛型参数
```rust
pub async fn with_retry<F, Fut, T>(&self, operation: F) -> Result<T, LlmError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, LlmError>>,
{
    // ...
}
```

### 难点 2: 不同的认证方式

**问题**: Deepseek 需要 Bearer token，Ollama 无认证

**方案**: Optional headers 参数
```rust
pub async fn post_json(
    &self,
    url: &str,
    payload: Value,
    headers: Option<HeaderMap>,  // 可选的额外 headers
) -> Result<Response, LlmError>
```

### 难点 3: 统计记录的组合

**问题**: 如何在公共层和业务层正确记录统计

**方案**: 使用装饰器模式
```rust
pub async fn record_operation<F, Fut, T>(&self, operation: F) -> Result<T, LlmError>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, LlmError>>,
{
    self.stats.record_call();
    match operation().await {
        Ok(result) => {
            self.stats.record_success();
            Ok(result)
        }
        Err(e) => {
            self.stats.record_error();
            Err(e)
        }
    }
}
```

---

## ✅ 验收标准

1. **代码重复率 < 30%**
   - 使用 `cargo-geiger` 或手动计算
   - DeepseekClient 和 OllamaClient 的核心代码应减少 40%+

2. **所有测试通过**
   - `cargo test --lib --tests`
   - 包括现有的 non-mockito 测试

3. **性能无退化**
   - 基准测试显示 ≤5% 性能差异
   - 内存占用无明显增加

4. **代码质量**
   - `cargo clippy` 无警告
   - `cargo fmt` 格式正确
   - 文档注释完整

---

**报告人**: Claude Code Agent
**审阅**: 待用户确认
**下一步**: 开始实现 HttpClientBase
