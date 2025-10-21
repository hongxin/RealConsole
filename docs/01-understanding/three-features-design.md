# 三项核心功能设计方案

**日期**: 2025-10-21
**哲学指导**: 易变（三态思维）+ 极简主义
**设计者**: RealConsole Contributors

---

## 📋 概述

本文档详细设计三项核心功能的优化和新增方案：

1. **Memory 指令优化** - 提升记忆系统的易用性和智能化
2. **LLM 交互日志** - 记录完整的 LLM 对话用于复盘和分析
3. **语音播报功能** - 可选的语音输出能力（macOS `say` 命令）

所有设计遵循项目的**一分为三**哲学和**极简主义**原则。

---

## 🧠 任务一：Memory 指令优化

### 当前状态分析

**现有功能**（`src/commands/memory.rs`）:
- ✅ 基础 CRUD：recent / search / clear / dump / save
- ✅ 类型过滤：按 user/assistant/system/shell/tool 分类
- ✅ JSONL 持久化
- ✅ 环形缓冲区（固定容量）

**痛点识别**:
1. **缺乏智能性**：没有重要性评分、自动总结
2. **搜索能力有限**：仅支持简单字符串匹配，无语义搜索
3. **展示不够直观**：时间展示只有 HH:MM:SS，无相对时间
4. **缺少统计分析**：无法了解记忆使用模式
5. **无主动推荐**：不会根据上下文主动提示相关记忆

### 设计方案：三态记忆系统

基于**一分为三**哲学，将记忆划分为三个层次：

```
记忆三态：
├─ 短期记忆（Short）：最近的交互，高度相关
├─ 中期记忆（Medium）：重要的知识点，需要保留
└─ 长期记忆（Long）：核心知识库，永久存储
```

#### 1.1 记忆重要性评分（三态）

```rust
/// 记忆重要性等级（一分为三）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Importance {
    /// 短期记忆：普通交互，自动淘汰
    Transient,
    /// 中期记忆：有价值的内容，延长保留
    Important,
    /// 长期记忆：核心知识，永久保存
    Critical,
}
```

**自动评分规则**:
- 包含代码块 → `Important`
- 包含错误修复 → `Important`
- 用户标记为重要 → `Critical`
- 工具调用结果 → `Important`
- 普通对话 → `Transient`

#### 1.2 新增命令

```bash
# 智能搜索（支持模糊匹配 + 时间范围）
/memory smart <关键词> [--days 7]

# 统计分析
/memory stats

# 重要性管理
/memory promote <id>     # 提升为重要记忆
/memory demote <id>      # 降级记忆
/memory critical         # 仅显示核心记忆

# 智能推荐
/memory related          # 基于当前上下文推荐相关记忆

# 导出增强
/memory export --format [json|md|html]
```

#### 1.3 展示优化

```rust
// 相对时间展示
fn relative_time(timestamp: DateTime<Utc>) -> String {
    let duration = Utc::now() - timestamp;

    if duration.num_seconds() < 60 {
        "刚刚".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}分钟前", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}小时前", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}天前", duration.num_days())
    } else {
        timestamp.format("%m-%d %H:%M").to_string()
    }
}
```

#### 1.4 统计面板

```
记忆统计 (最近 7 天)
━━━━━━━━━━━━━━━━━━━━
总条目:    156 条
  - 短期:  120 条 (77%)
  - 中期:   30 条 (19%)
  - 长期:    6 条 (4%)

类型分布:
  User       │████████████████ 45%
  Assistant  │████████████ 35%
  Shell      │█████ 15%
  Tool       │██ 5%

活跃时段:
  09:00-12:00  │████████ 高峰
  14:00-18:00  │██████ 活跃
  其他时段     │███ 正常
```

### 实现优先级

**Phase 1 (今日完成)**:
- [x] 相对时间展示
- [x] 统计分析命令
- [x] 重要性标记（手动）

**Phase 2 (本周)**:
- [ ] 智能评分
- [ ] 模糊搜索
- [ ] Markdown 导出

**Phase 3 (未来)**:
- [ ] 语义搜索（需要嵌入模型）
- [ ] 智能推荐
- [ ] 自动总结

---

## 📝 任务二：LLM 交互日志系统

### 需求分析

**目标**:
- 记录完整的 LLM 请求和响应
- 便于 debug 和问题排查
- 支持后续数据分析
- 隐私保护和数据脱敏

**记录内容**:
```json
{
  "timestamp": "2025-10-21T10:30:00Z",
  "session_id": "uuid",
  "model": "deepseek-chat",
  "request": {
    "messages": [...],
    "temperature": 0.7,
    "max_tokens": 2000
  },
  "response": {
    "content": "...",
    "usage": {
      "prompt_tokens": 150,
      "completion_tokens": 200,
      "total_tokens": 350
    },
    "finish_reason": "stop"
  },
  "latency_ms": 1200,
  "status": "success"
}
```

### 设计方案：三态日志系统

```
LLM 日志三态：
├─ 请求态（Request）：发送给 LLM 的内容
├─ 响应态（Response）：LLM 返回的内容
└─ 元态（Meta）：性能、错误、统计信息
```

#### 2.1 日志结构

```rust
/// LLM 交互日志
#[derive(Debug, Serialize, Deserialize)]
pub struct LlmInteractionLog {
    /// 会话 ID
    pub session_id: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 模型名称
    pub model: String,

    /// 请求内容
    pub request: LlmRequest,

    /// 响应内容
    pub response: Option<LlmResponse>,

    /// 元数据
    pub meta: LlmMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmRequest {
    /// 消息列表（可选择是否包含内容）
    pub messages: Vec<Message>,

    /// 摘要（用于隐私保护）
    pub summary: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    /// 响应内容
    pub content: String,

    /// Token 使用量
    pub usage: Option<TokenUsage>,

    /// 结束原因
    pub finish_reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmMetadata {
    /// 延迟（毫秒）
    pub latency_ms: u64,

    /// 状态（success/error/timeout）
    pub status: String,

    /// 错误信息（如果有）
    pub error: Option<String>,

    /// 是否为流式
    pub is_streaming: bool,
}
```

#### 2.2 配置选项

```yaml
# realconsole.yaml
llm:
  logging:
    enabled: true

    # 日志级别
    level: full  # full | meta | off

    # 存储路径
    log_dir: "~/.realconsole/llm_logs"

    # 隐私保护
    privacy:
      # 是否记录完整消息内容
      include_content: true

      # 敏感词过滤（正则）
      sensitive_patterns:
        - "api[_-]?key"
        - "password"
        - "token"

    # 自动清理
    retention:
      days: 30
      max_size_mb: 100
```

#### 2.3 日志管理命令

```bash
# 查看最近的 LLM 交互
/llm-log recent [n]

# 搜索日志
/llm-log search <关键词>

# 统计分析
/llm-log stats [--days 7]

# 导出日志
/llm-log export [--from date] [--to date]

# 清理日志
/llm-log clean [--days 30]

# 回放对话（用于 debug）
/llm-log replay <session_id>
```

#### 2.4 统计面板

```
LLM 交互统计 (最近 7 天)
━━━━━━━━━━━━━━━━━━━━━━━
总请求:   245 次
  - 成功:  238 次 (97%)
  - 失败:    7 次 (3%)

模型分布:
  deepseek-chat  │████████████████ 85%
  fallback       │████ 15%

性能指标:
  平均延迟:  1.2s
  P95 延迟:  2.5s
  Token 总量: 150K

Token 使用:
  Prompt:      85K (57%)
  Completion:  65K (43%)
```

#### 2.5 实现位置

```rust
// src/llm/logger.rs (新文件)
pub struct LlmLogger {
    config: LlmLoggingConfig,
    log_file: Arc<RwLock<File>>,
    session_id: String,
}

impl LlmLogger {
    pub async fn log_request(&self, model: &str, messages: &[Message]);
    pub async fn log_response(&self, content: &str, usage: TokenUsage, latency: Duration);
    pub async fn log_error(&self, error: &str);
}
```

**集成点**:
- `src/llm/deepseek.rs` - 在 `chat_stream` 中添加日志
- `src/services/llm_service.rs` - 在服务层统一记录
- `src/agent.rs` - 在 `handle_llm_chat` 中集成

### 实现优先级

**Phase 1 (今日完成)**:
- [x] 基础日志结构
- [x] 文件写入
- [x] 配置选项

**Phase 2 (本周)**:
- [ ] 日志查询命令
- [ ] 统计分析
- [ ] 隐私过滤

**Phase 3 (未来)**:
- [ ] 可视化分析
- [ ] 智能告警
- [ ] 性能优化建议

---

## 🔊 任务三：语音播报功能

### 需求分析

**使用场景**:
- 编码时听反馈（无需看屏幕）
- 长文本朗读
- 无障碍支持

**设计原则**:
- ✅ 可选特性（默认关闭）
- ✅ 轻量级（使用系统命令）
- ✅ 无外部依赖

### 设计方案：三态播报系统

```
语音播报三态：
├─ 静默态（Silent）：完全关闭
├─ 选择态（Selective）：仅重要内容
└─ 全播态（Full）：全部内容
```

#### 3.1 配置结构

```yaml
# realconsole.yaml
voice:
  # 启用状态（三态）
  mode: selective  # off | selective | full

  # 播报规则（selective 模式）
  rules:
    # 播报 LLM 响应
    llm_response: true

    # 播报系统消息
    system_message: true

    # 播报错误
    error: true

    # 播报 Shell 输出
    shell_output: false

    # 最小内容长度（避免播报太短的内容）
    min_length: 10

    # 最大内容长度（避免播报太长的内容）
    max_length: 500

  # macOS say 命令选项
  say:
    # 语音名称（中文：Ting-Ting，英文：Samantha）
    voice: "Ting-Ting"

    # 语速（words per minute）
    rate: 200

    # 音量（0-100）
    volume: 50

  # 内容预处理
  preprocessing:
    # 移除代码块
    remove_code: true

    # 移除 URL
    remove_urls: true

    # 最大句子长度
    max_sentence_length: 100
```

#### 3.2 实现架构

```rust
// src/voice/mod.rs (新模块)
pub mod broadcaster;
pub mod config;
pub mod preprocessor;

/// 语音播报器
pub struct VoiceBroadcaster {
    config: VoiceConfig,
    enabled: Arc<AtomicBool>,
}

impl VoiceBroadcaster {
    /// 创建新的语音播报器
    pub fn new(config: VoiceConfig) -> Self;

    /// 播报文本
    pub fn speak(&self, text: &str, content_type: ContentType);

    /// 异步播报（不阻塞）
    pub fn speak_async(&self, text: String, content_type: ContentType);

    /// 停止当前播报
    pub fn stop(&self);

    /// 动态切换模式
    pub fn set_mode(&self, mode: VoiceMode);
}

/// 内容类型（用于决策是否播报）
pub enum ContentType {
    LlmResponse,
    SystemMessage,
    Error,
    ShellOutput,
}

/// 播报模式（三态）
pub enum VoiceMode {
    Off,         // 关闭
    Selective,   // 选择性播报
    Full,        // 全部播报
}
```

#### 3.3 文本预处理

```rust
/// 文本预处理器
pub struct TextPreprocessor {
    config: PreprocessingConfig,
}

impl TextPreprocessor {
    /// 预处理文本（用于语音播报）
    pub fn preprocess(&self, text: &str) -> String {
        let mut result = text.to_string();

        // 1. 移除代码块
        if self.config.remove_code {
            result = self.remove_code_blocks(&result);
        }

        // 2. 移除 URL
        if self.config.remove_urls {
            result = self.remove_urls(&result);
        }

        // 3. 截断长句
        result = self.truncate_sentences(&result, self.config.max_sentence_length);

        // 4. 移除特殊符号
        result = self.clean_special_chars(&result);

        result
    }

    fn remove_code_blocks(&self, text: &str) -> String {
        // 移除 ```code``` 和 `code`
        let re = regex::Regex::new(r"```[\s\S]*?```|`[^`]+`").unwrap();
        re.replace_all(text, "[代码]").to_string()
    }

    fn remove_urls(&self, text: &str) -> String {
        let re = regex::Regex::new(r"https?://[^\s]+").unwrap();
        re.replace_all(text, "[链接]").to_string()
    }

    fn truncate_sentences(&self, text: &str, max_len: usize) -> String {
        // 如果整体太长，只取前 max_len 字符 + "..."
        if text.chars().count() > max_len {
            let truncated: String = text.chars().take(max_len).collect();
            format!("{}...", truncated)
        } else {
            text.to_string()
        }
    }
}
```

#### 3.4 macOS `say` 命令封装

```rust
use std::process::Command;

impl VoiceBroadcaster {
    /// 使用 macOS say 命令播报
    fn speak_via_say(&self, text: &str) {
        let voice = &self.config.say.voice;
        let rate = self.config.say.rate.to_string();
        let volume = self.config.say.volume.to_string();

        // 构建命令
        let mut cmd = Command::new("say");
        cmd.arg("-v").arg(voice)
           .arg("-r").arg(&rate)
           .arg("--volume").arg(&volume)
           .arg(text);

        // 异步执行（不阻塞）
        tokio::spawn(async move {
            match cmd.output() {
                Ok(output) => {
                    if !output.status.success() {
                        eprintln!("语音播报失败: {:?}", output.stderr);
                    }
                }
                Err(e) => {
                    eprintln!("语音播报错误: {}", e);
                }
            }
        });
    }
}
```

#### 3.5 集成方式

```rust
// 在 Agent 中集成
impl Agent {
    pub fn handle_llm_chat(&self, text: &str) -> String {
        // ... 原有逻辑

        // 语音播报（如果启用）
        if let Some(ref voice) = self.voice_broadcaster {
            voice.speak_async(
                response.clone(),
                ContentType::LlmResponse
            );
        }

        response
    }
}
```

#### 3.6 管理命令

```bash
# 查看语音状态
/voice status

# 切换模式
/voice mode [off|selective|full]

# 测试播报
/voice test "Hello, RealConsole!"

# 设置语音
/voice set-voice <voice-name>

# 列出可用语音
/voice list-voices

# 调整语速
/voice rate <wpm>
```

### 实现优先级

**Phase 1 (今日完成)**:
- [x] 基础结构和配置
- [x] `say` 命令封装
- [x] 简单文本预处理

**Phase 2 (本周)**:
- [ ] 三态模式切换
- [ ] 高级文本预处理
- [ ] 管理命令

**Phase 3 (未来)**:
- [ ] 支持其他 TTS 引擎（espeak, pico2wave）
- [ ] 自定义语音模板
- [ ] 语音情感控制

---

## 🎯 实施计划

### 今日任务分解

#### 上午（9:00 - 12:00）

**Task 1.1: Memory 优化 - 基础功能**
- [ ] 添加相对时间展示
- [ ] 实现 `/memory stats` 命令
- [ ] 添加重要性标记字段

**Task 2.1: LLM 日志 - 核心结构**
- [ ] 创建 `src/llm/logger.rs`
- [ ] 实现日志数据结构
- [ ] 实现文件写入逻辑

#### 下午（14:00 - 18:00）

**Task 2.2: LLM 日志 - 集成**
- [ ] 在 Deepseek 客户端集成日志
- [ ] 在 LlmService 集成日志
- [ ] 添加配置选项

**Task 3.1: 语音播报 - 基础实现**
- [ ] 创建 `src/voice/` 模块
- [ ] 实现 `VoiceBroadcaster`
- [ ] 封装 `say` 命令
- [ ] 简单测试

### 测试策略

```bash
# Memory 测试
cargo test --lib memory -- --nocapture

# LLM Logger 测试
cargo test --lib llm::logger -- --nocapture

# Voice 测试
cargo test --lib voice -- --nocapture

# 集成测试
./target/debug/realconsole --once "测试语音播报"
```

---

## 📚 参考资源

### Memory 优化
- [Ebbinghaus Forgetting Curve](https://en.wikipedia.org/wiki/Forgetting_curve)
- [Spaced Repetition](https://en.wikipedia.org/wiki/Spaced_repetition)

### LLM 日志
- [OpenTelemetry for LLM](https://opentelemetry.io/docs/)
- [LangSmith Tracing](https://docs.smith.langchain.com/)

### 语音播报
- [macOS `say` Manual](https://ss64.com/osx/say.html)
- [espeak-ng Documentation](https://github.com/espeak-ng/espeak-ng)

---

## 🔄 版本规划

- **v1.3.7** (今日): Memory 优化 + LLM 日志基础
- **v1.3.8** (本周): LLM 日志完善 + 语音播报
- **v1.4.0** (下月): 智能记忆 + 高级分析

---

## 📝 变更日志

- 2025-10-21: 初始设计方案
