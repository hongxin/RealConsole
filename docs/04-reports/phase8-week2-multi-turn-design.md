# Phase 8 Week 2: 多轮对话功能设计方案

**日期**: 2025-10-16
**任务**: Day 8-12
**优先级**: 🔴 高优先级
**状态**: 设计中

## 🎯 目标

实现智能多轮对话功能，让 RealConsole 能够：
- 理解用户意图并主动询问缺失参数
- 保持对话上下文，支持连续交互
- 自动验证参数并提供友好提示
- 优雅处理超时和用户退出

## 📖 用户场景

### 场景 1: 日志分析

```bash
用户: 帮我分析 nginx 日志
AI:   我需要知道日志文件路径，请提供完整路径
      提示: 通常在 /var/log/nginx/ 目录下

用户: /var/log/nginx/access.log
AI:   [分析中...]
      发现 23 个错误请求，最高频的是:
      - 404 Not Found: 12 次
      - 500 Internal Server Error: 8 次
      - 403 Forbidden: 3 次

      是否需要查看详细信息？

用户: 只看过去 1 小时的
AI:   [重新分析...]
      过去 1 小时有 5 个错误:
      - 404: 3 次 (16:20, 16:35, 16:48)
      - 500: 2 次 (16:25, 16:42)
```

### 场景 2: 文件操作

```bash
用户: 把所有 log 文件打包
AI:   需要以下信息:
      1. 源目录路径
      2. 目标压缩包名称

      请提供源目录路径:

用户: /var/log/
AI:   好的，源目录: /var/log/
      请提供目标压缩包名称 (回车使用默认: logs_20251016.tar.gz):

用户: [Enter]
AI:   使用默认名称: logs_20251016.tar.gz

      预览命令:
      tar -czf logs_20251016.tar.gz /var/log/*.log

      确认执行？[Y/n]

用户: y
AI:   [执行中...]
      ✓ 已创建压缩包: logs_20251016.tar.gz (1.2 MB)
```

### 场景 3: Git 操作

```bash
用户: 创建新的功能分支
AI:   请提供分支名称:

用户: feature/multi-turn
AI:   好的，将基于当前分支创建: feature/multi-turn

      预览操作:
      1. git checkout -b feature/multi-turn
      2. git push -u origin feature/multi-turn

      确认？[Y/n]

用户: y
AI:   ✓ 已创建并切换到分支: feature/multi-turn
      ✓ 已推送到远程仓库
```

## 🏗️ 架构设计

### 一分为三：状态 - 逻辑 - 展示

```
┌─────────────────────────────────────────┐
│         Conversation Module             │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────────┐   ┌───────────────┐  │
│  │  State Layer │   │  Logic Layer  │  │
│  │  (状态管理)  │←→│  (参数收集)   │  │
│  └──────────────┘   └───────────────┘  │
│         ↓                   ↓           │
│  ┌──────────────────────────────────┐  │
│  │      Presentation Layer          │  │
│  │      (用户交互显示)               │  │
│  └──────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### 核心组件

#### 1. ConversationContext（对话上下文）

```rust
/// 对话上下文
pub struct ConversationContext {
    /// 对话 ID
    pub id: String,

    /// 对话状态
    pub state: ConversationState,

    /// 意图类型
    pub intent: Intent,

    /// 已收集的参数
    pub parameters: HashMap<String, ParameterValue>,

    /// 待收集的参数
    pub pending_parameters: Vec<ParameterSpec>,

    /// 对话历史
    pub history: Vec<Turn>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 最后活动时间
    pub last_active: DateTime<Utc>,

    /// 超时设置（秒）
    pub timeout: u64,
}

/// 对话轮次
pub struct Turn {
    pub role: Role,           // User or Assistant
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// 参数值（支持多种类型）
pub enum ParameterValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Path(PathBuf),
    List(Vec<String>),
}

/// 参数规格
pub struct ParameterSpec {
    pub name: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub description: String,
    pub default: Option<ParameterValue>,
    pub validator: Option<Box<dyn Fn(&ParameterValue) -> Result<(), String>>>,
}
```

#### 2. ConversationState（状态机）

```rust
/// 对话状态
#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    /// 初始化：理解用户意图
    Initializing,

    /// 收集参数：询问缺失的参数
    CollectingParameters {
        current_param: String,
        retry_count: u8,
    },

    /// 验证中：验证所有参数
    Validating,

    /// 确认中：等待用户确认
    Confirming,

    /// 执行中：执行任务
    Executing,

    /// 完成：任务完成
    Completed {
        success: bool,
        message: String,
    },

    /// 已取消：用户取消
    Cancelled {
        reason: String,
    },

    /// 超时：对话超时
    Timeout,
}

/// 状态转换
impl ConversationState {
    /// 转换到下一个状态
    pub fn transition(&mut self, event: StateEvent) -> Result<(), String> {
        match (self.clone(), event) {
            // 初始化 -> 收集参数
            (Self::Initializing, StateEvent::IntentRecognized) => {
                *self = Self::CollectingParameters {
                    current_param: "".to_string(),
                    retry_count: 0
                };
                Ok(())
            }

            // 收集参数 -> 验证
            (Self::CollectingParameters { .. }, StateEvent::ParameterProvided) => {
                *self = Self::Validating;
                Ok(())
            }

            // 验证 -> 确认
            (Self::Validating, StateEvent::ValidationPassed) => {
                *self = Self::Confirming;
                Ok(())
            }

            // 确认 -> 执行
            (Self::Confirming, StateEvent::UserConfirmed) => {
                *self = Self::Executing;
                Ok(())
            }

            // 执行 -> 完成
            (Self::Executing, StateEvent::ExecutionCompleted { success, message }) => {
                *self = Self::Completed { success, message };
                Ok(())
            }

            // 任何状态 -> 取消
            (_, StateEvent::UserCancelled(reason)) => {
                *self = Self::Cancelled { reason };
                Ok(())
            }

            // 任何状态 -> 超时
            (_, StateEvent::Timeout) => {
                *self = Self::Timeout;
                Ok(())
            }

            // 无效转换
            _ => Err(format!("Invalid state transition: {:?} -> {:?}", self, event)),
        }
    }
}

/// 状态事件
#[derive(Debug, Clone)]
pub enum StateEvent {
    IntentRecognized,
    ParameterProvided,
    ValidationPassed,
    ValidationFailed(String),
    UserConfirmed,
    UserRejected,
    ExecutionCompleted { success: bool, message: String },
    UserCancelled(String),
    Timeout,
}
```

#### 3. ConversationManager（对话管理器）

```rust
/// 对话管理器
pub struct ConversationManager {
    /// 活跃的对话
    active_conversations: HashMap<String, ConversationContext>,

    /// LLM 客户端（用于智能理解）
    llm: Arc<dyn LlmClient>,

    /// 默认超时（秒）
    default_timeout: u64,
}

impl ConversationManager {
    /// 创建新对话
    pub fn start_conversation(
        &mut self,
        user_input: &str
    ) -> Result<ConversationContext, ConversationError> {
        // 1. 使用 LLM 理解用户意图
        let intent = self.extract_intent(user_input)?;

        // 2. 提取已有参数
        let initial_params = self.extract_parameters(user_input, &intent)?;

        // 3. 确定待收集的参数
        let pending_params = self.get_pending_parameters(&intent, &initial_params);

        // 4. 创建对话上下文
        let context = ConversationContext {
            id: Uuid::new_v4().to_string(),
            state: ConversationState::Initializing,
            intent,
            parameters: initial_params,
            pending_parameters: pending_params,
            history: vec![],
            created_at: Utc::now(),
            last_active: Utc::now(),
            timeout: self.default_timeout,
        };

        Ok(context)
    }

    /// 处理用户输入
    pub async fn handle_user_input(
        &mut self,
        conversation_id: &str,
        user_input: &str,
    ) -> Result<Response, ConversationError> {
        let context = self.get_mut_context(conversation_id)?;

        // 检查超时
        if self.is_timeout(context) {
            context.state.transition(StateEvent::Timeout)?;
            return Ok(Response::Timeout);
        }

        // 更新活跃时间
        context.last_active = Utc::now();

        // 记录用户输入
        context.history.push(Turn {
            role: Role::User,
            content: user_input.to_string(),
            timestamp: Utc::now(),
        });

        // 根据状态处理输入
        match &context.state {
            ConversationState::CollectingParameters { current_param, .. } => {
                self.collect_parameter(context, current_param, user_input).await
            }
            ConversationState::Confirming => {
                self.handle_confirmation(context, user_input).await
            }
            _ => Err(ConversationError::InvalidState),
        }
    }

    /// 收集参数
    async fn collect_parameter(
        &mut self,
        context: &mut ConversationContext,
        param_name: &str,
        value: &str,
    ) -> Result<Response, ConversationError> {
        // 1. 解析参数值
        let param_spec = context.pending_parameters
            .iter()
            .find(|p| p.name == param_name)
            .ok_or(ConversationError::ParameterNotFound)?;

        let param_value = self.parse_parameter_value(value, &param_spec.param_type)?;

        // 2. 验证参数
        if let Some(validator) = &param_spec.validator {
            validator(&param_value)?;
        }

        // 3. 保存参数
        context.parameters.insert(param_name.to_string(), param_value);

        // 4. 检查是否还有待收集的参数
        context.pending_parameters.retain(|p| p.name != param_name);

        if context.pending_parameters.is_empty() {
            // 所有参数已收集，转到验证状态
            context.state.transition(StateEvent::ParameterProvided)?;
            Ok(Response::AllParametersCollected)
        } else {
            // 继续收集下一个参数
            let next_param = &context.pending_parameters[0];
            Ok(Response::AskForParameter {
                name: next_param.name.clone(),
                description: next_param.description.clone(),
                default: next_param.default.clone(),
            })
        }
    }

    /// 检查超时
    fn is_timeout(&self, context: &ConversationContext) -> bool {
        let elapsed = (Utc::now() - context.last_active).num_seconds();
        elapsed as u64 > context.timeout
    }
}

/// 响应类型
#[derive(Debug)]
pub enum Response {
    /// 询问参数
    AskForParameter {
        name: String,
        description: String,
        default: Option<ParameterValue>,
    },

    /// 所有参数已收集
    AllParametersCollected,

    /// 确认操作
    ConfirmAction {
        summary: String,
        command: String,
    },

    /// 执行结果
    ExecutionResult {
        success: bool,
        output: String,
    },

    /// 超时
    Timeout,

    /// 已取消
    Cancelled,
}
```

## 🔧 实现计划

### Day 8-9: 基础结构

**文件**:
- `src/conversation.rs` - 核心模块
- `src/conversation/context.rs` - 对话上下文
- `src/conversation/state.rs` - 状态机
- `src/conversation/manager.rs` - 对话管理器

**任务**:
1. ✅ 定义数据结构（ConversationContext, ConversationState）
2. ✅ 实现状态机转换逻辑
3. ✅ 实现基本的参数收集流程
4. ✅ 单元测试（状态转换、参数验证）

### Day 10-11: Agent 集成

**文件**:
- `src/agent.rs` - 添加多轮对话支持
- `src/repl.rs` - REPL 交互增强

**任务**:
1. ✅ Agent 添加 ConversationManager 字段
2. ✅ 实现 `handle_multi_turn()` 方法
3. ✅ REPL 检测和管理对话状态
4. ✅ 实现超时和取消机制

### Day 12: 场景实现和测试

**文件**:
- `src/conversation/scenarios/log_analysis.rs`
- `src/conversation/scenarios/file_operations.rs`
- `tests/test_conversation.rs`

**任务**:
1. ✅ 实现日志分析场景
2. ✅ 实现文件操作场景
3. ✅ 集成测试（完整对话流程）
4. ✅ 用户文档

## 📊 验收标准

### 功能标准
- ✅ 多轮对话状态管理可用
- ✅ 参数收集和验证正确
- ✅ 3+ 场景测试通过
- ✅ 超时机制正确触发
- ✅ 取消和退出优雅处理

### 性能标准
- ✅ 对话响应 < 100ms
- ✅ 内存占用 < 5MB（每个对话）
- ✅ 支持至少 10 个并发对话

### 用户体验标准
- ✅ 参数提示清晰友好
- ✅ 默认值建议合理
- ✅ 错误提示有建设性
- ✅ 取消操作即时响应

## 🎨 UI/UX 设计

### 提示符设计

```bash
# 普通模式
hongxin real-console %

# 对话模式（收集参数）
hongxin real-console [日志分析] %

# 对话模式（确认）
hongxin real-console [确认] %
```

### 参数提示格式

```bash
💬 需要以下信息:
   1. 日志文件路径
   2. 分析时间范围（可选）

📝 请提供日志文件路径:
   提示: 通常在 /var/log/ 目录下
   示例: /var/log/nginx/access.log

» /var/log/nginx/access.log
```

### 确认界面

```bash
📋 操作预览:
   任务: 分析 nginx 日志
   文件: /var/log/nginx/access.log
   时间: 过去 1 小时

🔍 将执行的命令:
   tail -n 1000 /var/log/nginx/access.log | grep "$(date -d '1 hour ago' '+%d/%b/%Y:%H')"

❓ 确认执行？[Y/n/preview]
```

## 🧪 测试策略

### 单元测试
```rust
#[test]
fn test_state_transition() {
    let mut state = ConversationState::Initializing;
    assert!(state.transition(StateEvent::IntentRecognized).is_ok());
    assert_eq!(state, ConversationState::CollectingParameters {
        current_param: "".to_string(),
        retry_count: 0
    });
}

#[test]
fn test_parameter_collection() {
    let mut manager = ConversationManager::new(llm, 300);
    let context = manager.start_conversation("分析日志").unwrap();

    // 模拟参数收集
    let response = manager.handle_user_input(
        &context.id,
        "/var/log/nginx/access.log"
    ).await.unwrap();

    assert!(matches!(response, Response::AllParametersCollected));
}

#[test]
fn test_timeout() {
    let mut context = ConversationContext::new(Intent::LogAnalysis);
    context.timeout = 1; // 1 秒超时

    std::thread::sleep(Duration::from_secs(2));

    assert!(manager.is_timeout(&context));
}
```

### 集成测试
```rust
#[tokio::test]
async fn test_complete_conversation() {
    // 完整对话流程测试
    let agent = create_test_agent();

    // 1. 启动对话
    let response1 = agent.handle("分析 nginx 日志").await;
    assert!(response1.contains("请提供日志文件路径"));

    // 2. 提供参数
    let response2 = agent.handle("/var/log/nginx/access.log").await;
    assert!(response2.contains("分析中") || response2.contains("发现"));

    // 3. 细化请求
    let response3 = agent.handle("只看过去 1 小时的").await;
    assert!(response3.contains("过去 1 小时"));
}
```

## 📚 相关文档

- **用户指南**: `docs/02-practice/user/multi-turn-guide.md`
- **API 文档**: `docs/02-practice/developer/conversation-api.md`
- **状态机图**: `docs/01-understanding/design/conversation-fsm.md`

## 🚀 后续优化

### Phase 9 增强
- 对话历史分析（学习用户习惯）
- 智能参数推荐（基于历史）
- 多对话并行管理
- 对话持久化和恢复
- 语音输入支持

---

**下一步**: 开始实现 `src/conversation.rs` 核心模块
