# 对话上下文模式设计方案

**创建时间**: 2025-10-20
**状态**: 设计中
**阶段**: UX 优化

## 背景与动机

### 初心 vs 现实

**初心**：RealConsole 定位为智能化 Console，设计为单命令执行模式：
- 每条命令独立执行
- 无需维护复杂的对话状态
- 轻量、快速、聚焦

**现实**：用户使用场景演化：
- 许多用户将其作为 LLM 对话工具使用
- 需要跨多轮维护上下文
- 期望"记住"之前的对话内容

### 解决方案

**一分为三的设计哲学**：将上下文视为**可选模式**，而非强制特性

- **关闭模式（Disabled）**：保持初心，单命令执行，无上下文
- **手动模式（Manual）**：用户显式控制上下文的开启/关闭
- **自动模式（Auto）**：智能识别需要上下文的场景

---

## 配置设计

### ConversationConfig 结构

```yaml
conversation:
  # 上下文模式：disabled（关闭）、manual（手动）、auto（自动）
  mode: manual

  # 最大轮次（保留最近 N 轮对话）
  max_turns: 10

  # 最大上下文长度（字符数，超过则自动裁剪）
  max_context_length: 8000

  # 自动清除策略
  auto_clear:
    # 是否启用自动清除
    enabled: true

    # 空闲多久后清除（秒）
    idle_timeout: 600  # 10 分钟

    # 任务完成后是否清除
    on_task_complete: false

  # 上下文包含内容
  include:
    # 是否包含工具调用历史
    tool_calls: true

    # 是否包含 Shell 执行结果
    shell_output: false

    # 是否包含错误信息
    errors: true
```

### 三种模式说明

#### 1. Disabled（关闭模式）

```yaml
conversation:
  mode: disabled
```

**行为**：
- 每条命令独立执行
- 不保留任何上下文
- 最快、最轻量
- 适合：快速查询、单次命令、脚本化使用

#### 2. Manual（手动模式）

```yaml
conversation:
  mode: manual
  max_turns: 10
```

**行为**：
- 默认不启用上下文
- 用户通过命令显式控制：
  - `/context start` - 开始记录上下文
  - `/context stop` - 停止并清除上下文
  - `/context show` - 查看当前上下文
  - `/context clear` - 清除上下文但不停止

**适合**：
- 长时间对话任务
- 多轮问答
- 需要精确控制的场景

#### 3. Auto（自动模式）

```yaml
conversation:
  mode: auto
  max_turns: 5
  auto_clear:
    enabled: true
    idle_timeout: 300
```

**行为**：
- 智能识别需要上下文的场景
- 触发条件：
  - 用户输入包含代词（"它"、"这个"、"那个"）
  - 追问（"为什么"、"继续"、"详细说明"）
  - 任务未完成需要多轮
- 自动清除：空闲超时或任务完成

**适合**：
- 日常混合使用
- 既有单命令也有对话
- 平衡便利性和性能

---

## 实现方案

### 1. 配置层（config.rs）

```rust
/// 对话上下文配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationConfig {
    /// 上下文模式
    #[serde(default)]
    pub mode: ContextMode,

    /// 最大轮次
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,

    /// 最大上下文长度（字符）
    #[serde(default = "default_max_context_length")]
    pub max_context_length: usize,

    /// 自动清除策略
    #[serde(default)]
    pub auto_clear: AutoClearConfig,

    /// 上下文包含内容
    #[serde(default)]
    pub include: ContextIncludeConfig,
}

/// 上下文模式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextMode {
    Disabled,  // 关闭
    Manual,    // 手动
    Auto,      // 自动
}

/// 自动清除配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoClearConfig {
    pub enabled: bool,
    pub idle_timeout: u64,        // 秒
    pub on_task_complete: bool,
}

/// 上下文包含配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextIncludeConfig {
    pub tool_calls: bool,
    pub shell_output: bool,
    pub errors: bool,
}
```

### 2. 上下文管理器（conversation/context_manager.rs）

```rust
/// 对话上下文管理器
pub struct ContextManager {
    config: ConversationConfig,
    turns: VecDeque<Turn>,
    is_active: bool,
    last_activity: DateTime<Utc>,
}

impl ContextManager {
    /// 检查是否应该启用上下文（Auto 模式）
    fn should_enable_context(&self, input: &str) -> bool {
        // 代词检测
        let pronouns = ["它", "这个", "那个", "this", "that", "it"];
        if pronouns.iter().any(|p| input.contains(p)) {
            return true;
        }

        // 追问检测
        let followups = ["为什么", "继续", "详细", "why", "continue", "more"];
        if followups.iter().any(|f| input.contains(f)) {
            return true;
        }

        false
    }

    /// 构建上下文消息（用于 LLM API）
    fn build_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        for turn in &self.turns {
            messages.push(Message::user(&turn.user_input));
            messages.push(Message::assistant(&turn.assistant_response));

            // 可选：包含工具调用
            if self.config.include.tool_calls {
                for tool in &turn.tools_called {
                    // ... 添加工具调用信息
                }
            }
        }

        messages
    }

    /// 清理过期上下文
    fn cleanup_if_needed(&mut self) {
        if !self.config.auto_clear.enabled {
            return;
        }

        let idle_duration = Utc::now() - self.last_activity;
        if idle_duration.num_seconds() > self.config.auto_clear.idle_timeout as i64 {
            self.clear();
        }
    }
}
```

### 3. Agent 集成

在 `Agent::run_llm()` 中：

```rust
// 根据配置模式决定是否使用上下文
let messages = match self.state_manager().context_manager().mode() {
    ContextMode::Disabled => {
        // 单条消息，无上下文
        vec![Message::user(input)]
    }
    ContextMode::Manual => {
        // 仅在显式启用时使用上下文
        if self.state_manager().context_manager().is_active() {
            self.state_manager().context_manager().build_messages()
        } else {
            vec![Message::user(input)]
        }
    }
    ContextMode::Auto => {
        // 智能决策
        if self.state_manager().context_manager().should_use_context(input) {
            self.state_manager().context_manager().build_messages()
        } else {
            vec![Message::user(input)]
        }
    }
};
```

---

## 用户交互

### 系统命令

```bash
# 手动模式控制
/context start       # 开始记录上下文
/context stop        # 停止并清除
/context clear       # 仅清除，不停止
/context show        # 查看当前上下文（轮次、长度）
/context status      # 查看配置和状态

# 示例输出
> /context status
上下文模式: Manual (手动)
当前状态: Active (活跃)
已记录轮次: 3/10
上下文长度: 1247/8000 字符
最后活动: 2 分钟前
```

### REPL 提示

在 **Auto 模式**下，提示用户上下文状态：

```bash
# 上下文激活时
[上下文: 3轮] > 继续解释

# 空闲即将清除
[上下文: 5轮 | 8分钟前] >
⚠️  上下文即将因空闲清除（10分钟超时）
```

---

## 配置示例

### 极简用户（关闭模式）

```yaml
conversation:
  mode: disabled
```

### 对话用户（手动模式）

```yaml
conversation:
  mode: manual
  max_turns: 20
  max_context_length: 16000
  auto_clear:
    enabled: false  # 不自动清除，手动控制
```

### 混合用户（自动模式）

```yaml
conversation:
  mode: auto
  max_turns: 5
  max_context_length: 8000
  auto_clear:
    enabled: true
    idle_timeout: 600      # 10 分钟
    on_task_complete: true # 任务完成后清除
  include:
    tool_calls: true
    shell_output: false
    errors: true
```

---

## 迁移策略

### 向后兼容

1. **默认配置**：`mode: disabled`
   - 保持现有行为，不影响老用户

2. **配置缺失**：使用默认值
   ```rust
   impl Default for ConversationConfig {
       fn default() -> Self {
           Self {
               mode: ContextMode::Disabled,
               max_turns: 10,
               max_context_length: 8000,
               // ...
           }
       }
   }
   ```

3. **渐进式引导**：
   - 检测到用户频繁使用代词/追问时，提示启用上下文模式
   - Wizard 配置时询问用户偏好

---

## 性能考量

### Token 消耗

| 模式 | Token 消耗 | 适用场景 |
|------|-----------|----------|
| Disabled | 最低（仅当前输入） | 单命令、查询 |
| Manual (5轮) | 中等 | 短对话任务 |
| Auto (智能) | 动态平衡 | 混合使用 |

### 内存占用

- **Disabled**: ~0 KB（无缓存）
- **Manual (10轮，8K/轮)**: ~80 KB
- **Auto**: 动态，一般 < 50 KB

---

## 未来扩展

1. **上下文摘要**（Context Summarization）
   - 当超过 max_turns 时，用 LLM 生成摘要
   - 保留关键信息，压缩历史

2. **跨会话持久化**
   - 保存重要对话到 Memory
   - 下次启动时可恢复

3. **上下文分支**
   - 支持多个并行上下文
   - 适合同时处理多个任务

4. **语义压缩**
   - 智能去重相似内容
   - 保留语义关键信息

---

## 实施计划

- [x] Phase 1: 设计方案（当前）
- [ ] Phase 2: 实现配置层（config.rs）
- [ ] Phase 3: 实现 ContextManager
- [ ] Phase 4: 集成到 Agent
- [ ] Phase 5: 添加系统命令
- [ ] Phase 6: 测试和文档
- [ ] Phase 7: 用户反馈和优化

---

**设计原则**：
- ✅ **可选性**：保持极简主义，上下文是选项而非强制
- ✅ **灵活性**：三种模式适应不同用户
- ✅ **性能**：默认关闭，避免不必要开销
- ✅ **智能**：Auto 模式降低用户心智负担
- ✅ **一分为三**：Disabled/Manual/Auto 三态清晰

---

**参考**：
- OpenAI ChatGPT：默认保留全部上下文
- Claude Code：按需上下文（类似我们的 Manual）
- GitHub Copilot Chat：短期上下文 + 智能遗忘

RealConsole 的差异化：**可选 + 可配置 + 智能化**
