# Web 版本 LLM 初始化诊断功能

**版本**: v1.27.0
**日期**: 2025-11-06
**问题**: Web 版本 LLM 初始化失败时，用户只看到通用错误信息，无法诊断具体原因

---

## 问题背景

### 用户反馈

用户在配置 Ollama 作为 LLM 提供商时，Web 终端显示：

```
% hello
未配置 LLM，无法进行对话
```

但实际上用户已经在 `realconsole.yaml` 中配置了 Ollama，问题不是"未配置"，而是**初始化失败**。

### 根本原因

Web 版本与 CLI 版本的关键差异：

| 环境 | 错误信息可见性 | 用户体验 |
|------|---------------|---------|
| **CLI 版本** | ✅ stderr 直接显示在终端 | 用户能看到详细错误（网络问题、配置错误等） |
| **Web 版本** | ❌ stderr 只在服务器日志中 | 用户只看到通用"未配置"提示 |

**核心问题**：`Session::configure_llm()` 中的错误处理：

```rust
// src/web/session.rs:107-112 (修复前)
Err(e) => {
    let error_msg = format!("{}: {}",
        i18n::t("web.session.primary_llm_init_failed"), e);
    eprintln!("{}", error_msg);  // ❌ 只输出到 stderr
    error_messages.push(error_msg);  // ✅ 但没有传递给用户
}
```

用户在 `websocket.rs` 中只能收到：

```rust
// src/web/websocket.rs:267-271 (修复前)
if llm_manager.primary().is_none() {
    let msg = ServerMessage::Error {
        content: i18n::t("web.llm.not_configured")  // ❌ 通用消息
    };
    sender.send(Message::Text(serde_json::to_string(&msg)?)).await?;
    return Ok(());
}
```

---

## 解决方案

### 设计目标

1. **捕获初始化错误**：在 Session 创建时记录所有 LLM 初始化失败信息
2. **传递错误上下文**：将错误信息通过 Session 传递到 WebSocket 处理函数
3. **用户友好展示**：在 Web 界面显示详细的诊断信息，帮助用户定位问题

### 架构改进

```
┌─────────────────────────────────────────────────────────────┐
│  Session::new()                                             │
│  ├─ configure_llm() → 返回 Option<String>                   │
│  └─ llm_init_error: Option<String> ← 存储初始化错误         │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│  handle_websocket()                                         │
│  └─ session: Arc<Session> ← 传递到消息处理                  │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│  execute_llm_chat(input, agent, session, sender)            │
│  └─ 检查 session.llm_init_error                             │
│      ├─ Some(error) → 显示详细诊断信息                      │
│      └─ None → 显示配置缺失提示                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 实现细节

### 1. Session 结构扩展

**文件**: `src/web/session.rs:44-54`

```rust
pub struct Session {
    /// 会话 ID
    pub id: SessionId,
    /// Agent 实例（独立）
    pub agent: Arc<RwLock<Agent>>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// LLM 初始化错误信息（用于诊断）✨ 新增
    pub llm_init_error: Option<String>,
}
```

### 2. 错误收集机制

**文件**: `src/web/session.rs:78-135`

```rust
/// 配置 Agent 的 LLM
///
/// 返回初始化错误信息（如果有）✨ 修改返回类型
async fn configure_llm(agent: &mut Agent, config: &Config) -> Option<String> {
    let mut manager = agent.llm_manager.write().await;
    let mut error_messages = Vec::new();  // ✨ 收集所有错误

    // 初始化 primary LLM
    if let Some(ref primary_cfg) = config.llm.primary {
        match Self::create_llm_client(primary_cfg) {
            Ok(client) => {
                manager.set_primary(client.clone());
                // ... 成功逻辑 ...
            }
            Err(e) => {
                let error_msg = format!("{}: {}",
                    i18n::t("web.session.primary_llm_init_failed"), e);
                eprintln!("{}", error_msg);
                error_messages.push(error_msg);  // ✨ 收集错误
            }
        }
    }

    // 初始化 fallback LLM
    if let Some(ref fallback_cfg) = config.llm.fallback {
        match Self::create_llm_client(fallback_cfg) {
            Ok(client) => {
                manager.set_fallback(client);
            }
            Err(e) => {
                let error_msg = format!("{}: {}",
                    i18n::t("web.session.fallback_llm_init_failed"), e);
                eprintln!("{}", error_msg);
                error_messages.push(error_msg);  // ✨ 收集错误
            }
        }
    }

    // ✨ 返回合并的错误信息
    if error_messages.is_empty() {
        None
    } else {
        Some(error_messages.join("\n"))
    }
}
```

### 3. Session 创建时捕获错误

**文件**: `src/web/session.rs:68`

```rust
pub async fn new(config: Config, registry: CommandRegistry) -> Self {
    let id = Uuid::new_v4().to_string();

    let mut web_config = config.clone();
    web_config.features.tool_calling_enabled = Some(true);

    let mut agent = Agent::new(web_config.clone(), registry);

    // ✨ 配置 LLM（参考 main.rs），记录初始化错误
    let llm_init_error = Self::configure_llm(&mut agent, &web_config).await;

    Self {
        id,
        agent: Arc::new(RwLock::new(agent)),
        created_at: chrono::Utc::now(),
        llm_init_error,  // ✨ 存储错误信息
    }
}
```

### 4. WebSocket 函数签名更新

**文件**: `src/web/websocket.rs`

#### a. execute_llm_chat 添加 session 参数

**行**: 259-261

```rust
async fn execute_llm_chat(
    input: &str,
    agent: &crate::agent::Agent,
    session: &Arc<Session>,  // ✨ 新增参数
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
```

#### b. execute_intent 添加 session 参数

**行**: 355-361

```rust
async fn execute_intent(
    intent_match: &IntentMatch,
    original_text: &str,
    agent: &crate::agent::Agent,
    session: &Arc<Session>,  // ✨ 新增参数
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
```

### 5. 详细错误信息展示

**文件**: `src/web/websocket.rs:267-298`

```rust
async fn execute_llm_chat(
    input: &str,
    agent: &crate::agent::Agent,
    session: &Arc<Session>,
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let llm_manager = agent.llm_manager.read().await;

    if llm_manager.primary().is_none() {
        drop(llm_manager);

        // ✨ 提供详细的诊断信息
        let error_content = if let Some(ref init_error) = session.llm_init_error {
            // 情况 1: 有初始化错误 → 显示详细诊断
            format!(
                "{}\n\n{}\n{}",
                i18n::t("web.llm.not_configured"),
                i18n::t("web.llm.init_error_details"),
                init_error  // ← 显示实际错误（网络、配置等）
            )
        } else {
            // 情况 2: 配置缺失 → 显示配置提示
            format!(
                "{}\n\n{}",
                i18n::t("web.llm.not_configured"),
                i18n::t("web.llm.config_missing_hint")
            )
        };

        let msg = ServerMessage::Error { content: error_content };
        sender.send(Message::Text(serde_json::to_string(&msg)?)).await?;
        return Ok(());
    }

    // ... 正常 LLM 执行逻辑 ...
}
```

### 6. 调用点更新

**自然语言处理** (行 158-166):

```rust
CommandType::NaturalLanguage(text) => {
    if let Some(intent_match) = try_match_intent(&text, &agent) {
        execute_intent(&intent_match, &text, &agent, session, sender).await  // ✨ 传递 session
    } else {
        execute_llm_chat(&text, &agent, session, sender).await  // ✨ 传递 session
    }
}
```

**Intent 回退** (行 399-403):

```rust
Err(e) => {
    // 如果生成执行计划失败，回退到 LLM 对话
    eprintln!("⚠️ Intent 执行计划生成失败: {}", e);
    execute_llm_chat(original_text, agent, session, sender).await?;  // ✨ 传递 session
}
```

---

## 国际化支持

### 新增翻译 Key

需要在 `locales/zh-CN-cli.yaml` 和 `locales/en-US-cli.yaml` 中添加：

```yaml
web:
  llm:
    not_configured: "未配置 LLM，无法进行对话"
    init_error_details: "LLM 初始化失败详情："
    config_missing_hint: "请检查 realconsole.yaml 配置文件，确保配置了 llm.primary 或 llm.fallback"
  session:
    primary_llm_init_failed: "Primary LLM 初始化失败"
    fallback_llm_init_failed: "Fallback LLM 初始化失败"
    ollama_client_creation_failed: "Ollama 客户端创建失败"
    deepseek_client_creation_failed: "Deepseek 客户端创建失败"
    deepseek_requires_api_key: "Deepseek 需要配置 API Key"
    unknown_llm_provider: "未知的 LLM 提供商"
```

---

## 用户体验改进

### 改进前

```
% hello
❌ 未配置 LLM，无法进行对话
```

用户完全不知道问题所在，可能的原因：
- Ollama 服务未启动？
- 配置文件语法错误？
- 网络连接问题？
- 端点地址错误？

### 改进后 - 场景 1：Ollama 连接失败

```
% hello
❌ 未配置 LLM，无法进行对话

LLM 初始化失败详情：
Primary LLM 初始化失败: Ollama 客户端创建失败: error sending request for url
(http://localhost:11434/api/tags): error trying to connect: tcp connect error:
Connection refused (os error 61)
```

**用户可以清楚知道**：
- 问题是 **Ollama 服务未启动**（Connection refused）
- 端点地址：`http://localhost:11434`
- 解决方案：启动 Ollama 服务

### 改进后 - 场景 2：Deepseek API Key 缺失

```
% hello
❌ 未配置 LLM，无法进行对话

LLM 初始化失败详情：
Primary LLM 初始化失败: Deepseek 需要配置 API Key
```

**用户可以清楚知道**：
- 问题是 **缺少 API Key**
- 解决方案：在配置文件中添加 `api_key` 字段

### 改进后 - 场景 3：配置文件缺失

```
% hello
❌ 未配置 LLM，无法进行对话

请检查 realconsole.yaml 配置文件，确保配置了 llm.primary 或 llm.fallback
```

**用户可以清楚知道**：
- 问题是 **配置文件中没有 LLM 配置**
- 解决方案：添加 `llm.primary` 或 `llm.fallback` 配置

---

## 测试场景

### 1. Ollama 未启动

**配置**：
```yaml
llm:
  primary:
    provider: "ollama"
    model: "qwen2.5:latest"
    endpoint: "http://localhost:11434"
```

**预期行为**：
```
❌ ... Connection refused (os error 61)
```

### 2. Ollama 端点错误

**配置**：
```yaml
llm:
  primary:
    provider: "ollama"
    endpoint: "http://localhost:9999"  # 错误端口
```

**预期行为**：
```
❌ ... tcp connect error: Connection refused
```

### 3. Deepseek API Key 缺失

**配置**：
```yaml
llm:
  primary:
    provider: "deepseek"
    model: "deepseek-chat"
    # api_key 未配置
```

**预期行为**：
```
❌ ... Deepseek 需要配置 API Key
```

### 4. 未知提供商

**配置**：
```yaml
llm:
  primary:
    provider: "unknown-provider"
```

**预期行为**：
```
❌ ... 未知的 LLM 提供商: unknown-provider
```

### 5. Primary + Fallback 都失败

**配置**：
```yaml
llm:
  primary:
    provider: "ollama"
    endpoint: "http://localhost:11434"  # 未启动
  fallback:
    provider: "deepseek"
    # api_key 缺失
```

**预期行为**：
```
❌ LLM 初始化失败详情：
Primary LLM 初始化失败: ... Connection refused
Fallback LLM 初始化失败: ... Deepseek 需要配置 API Key
```

---

## 技术要点

### 1. 错误信息传递链

```
configure_llm()  →  Session.llm_init_error  →  execute_llm_chat()  →  WebSocket  →  Web UI
   (捕获)              (存储)                      (读取)             (发送)       (显示)
```

### 2. Arc<Session> 共享

```rust
// Session 在 WebSocket 生命周期内共享
let session = Arc::new(Session::new(config, registry).await);

// 传递给所有需要诊断信息的函数
execute_llm_chat(&text, &agent, &session, sender).await
execute_intent(&intent_match, &text, &agent, &session, sender).await
```

### 3. 函数签名演变

**Before**:
```rust
async fn execute_llm_chat(
    input: &str,
    agent: &Agent,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<()>
```

**After**:
```rust
async fn execute_llm_chat(
    input: &str,
    agent: &Agent,
    session: &Arc<Session>,  // ← 新增
    sender: &mut SplitSink<WebSocket, Message>,
) -> Result<()>
```

### 4. 错误信息格式化

使用 `\n\n` 分隔不同部分，在 Web UI 中渲染为多行：

```rust
format!(
    "{}\n\n{}\n{}",  // ← 双换行分隔主标题和详情
    i18n::t("web.llm.not_configured"),
    i18n::t("web.llm.init_error_details"),
    init_error
)
```

---

## 潜在问题与改进

### 1. 安全性考虑

**当前实现**：将完整错误信息（包括端点地址、网络错误等）发送到 Web 客户端

**潜在风险**：
- 可能泄露内部网络拓扑信息
- 可能暴露配置文件路径
- 错误堆栈可能包含敏感数据

**改进方向**：
```rust
// 可选：过滤敏感信息
fn sanitize_error(error: &str) -> String {
    error
        .replace("/Users/xxx/", "~/.realconsole/")  // 隐藏用户路径
        .replace("192.168.", "xxx.xxx.")  // 隐藏内网 IP
        // ... 更多过滤规则
}
```

### 2. 错误信息国际化

**当前实现**：部分错误来自底层库（如 `reqwest`），是英文

**改进方向**：
- 解析常见错误模式，映射到国际化 key
- 提供中英文对照的错误描述

### 3. 诊断工具命令

**未来改进**：添加 `/diagnose` 系统命令，主动测试 LLM 连接：

```
% /diagnose llm
🔍 正在诊断 LLM 配置...

Primary (ollama):
  ✅ 端点可访问: http://localhost:11434
  ✅ 模型可用: qwen2.5:latest
  ✅ API 响应正常

Fallback (deepseek):
  ❌ API Key 缺失

总结: Primary LLM 正常，可以使用
```

### 4. 重试机制

**当前实现**：初始化失败后不会重试

**改进方向**：
- 添加 `/llm reload` 命令，重新初始化 LLM
- 自动重试（对网络临时故障）

---

## 修改文件清单

| 文件 | 修改内容 | 行数变化 |
|------|---------|---------|
| `src/web/session.rs` | 1. 添加 `llm_init_error` 字段<br>2. 修改 `configure_llm()` 返回类型<br>3. 收集并返回错误信息 | +8 行 |
| `src/web/websocket.rs` | 1. 更新 `execute_llm_chat()` 签名<br>2. 更新 `execute_intent()` 签名<br>3. 添加详细错误展示逻辑<br>4. 更新所有调用点 | +15 行 |

**总计**：新增代码 ~23 行，修改逻辑 ~30 行

---

## 版本信息

**实现版本**: v1.27.0（计划）
**依赖版本**:
- Rust: 1.90+
- tokio: 1.28+
- serde: 1.0+

**测试状态**:
- ✅ 编译通过
- ⏳ 待用户在不同网络环境下测试 Ollama 连接

---

## 总结

### 核心改进

1. **可见性**：将隐藏在服务器 stderr 中的错误信息展示给 Web 用户
2. **诊断性**：提供详细的错误原因，帮助用户快速定位问题
3. **用户体验**：从"未配置"这样的模糊提示，变为具体的诊断指引

### 设计原则

- **最小侵入性**：只在 Session 和 WebSocket 层添加少量代码
- **向后兼容**：不影响 CLI 版本的错误处理
- **渐进增强**：没有错误信息时仍然有友好的提示

### 用户价值

**Before**: "为什么 LLM 不工作？" → 不知道从何下手
**After**: "哦，原来是 Ollama 服务没启动！" → 明确的行动方向

---

**改进完成** ✅
**诊断能力**: ⭐⭐⭐⭐⭐
**用户体验**: 🚀 显著提升
