# Phase 4.2 P2 功能设计方案

**日期**: 2025-10-27
**版本**: v1.8.0 (计划)
**状态**: 📋 设计中

---

## 目录

1. [P2 功能概览](#一p2-功能概览)
2. [优先级分析](#二优先级分析)
3. [功能1：学习用户反馈](#三功能1学习用户反馈)
4. [功能2：智能参数补全](#四功能2智能参数补全)
5. [功能3：上下文链式建议](#五功能3上下文链式建议)
6. [实施计划](#六实施计划)

---

## 一、P2 功能概览

Phase 4.2 P2 包含三个增强功能：

| 功能 | 目标 | 价值 | 复杂度 | 优先级 |
|------|------|------|--------|--------|
| **学习用户反馈** | 记录用户选择，优化建议评分 | ⭐⭐⭐⭐⭐ | 中 | 🥇 P2.1 |
| **智能参数补全** | 自动填充占位符（如 `<url>`） | ⭐⭐⭐⭐ | 中 | 🥈 P2.2 |
| **上下文链式建议** | 执行后生成下一步建议 | ⭐⭐⭐ | 高 | 🥉 P2.3 |

---

## 二、优先级分析

### 2.1 RICE 评分

| 功能 | Reach | Impact | Confidence | Effort | Score |
|------|-------|--------|-----------|--------|-------|
| 学习用户反馈 | 100% | 9 | 80% | 2周 | **360** |
| 智能参数补全 | 70% | 8 | 70% | 1.5周 | **261** |
| 上下文链式建议 | 60% | 7 | 60% | 3周 | **140** |

### 2.2 推荐顺序

1. 🥇 **学习用户反馈**（Score: 360）
   - 最高投入产出比
   - 为后续功能提供数据基础
   - 持续提升系统能力

2. 🥈 **智能参数补全**（Score: 261）
   - 直接提升用户体验
   - 实现相对简单
   - 可独立使用

3. 🥉 **上下文链式建议**（Score: 140）
   - 更复杂的功能
   - 依赖前两个功能的数据
   - 可以作为 P3 或后续版本

---

## 三、功能1：学习用户反馈

### 3.1 设计目标

**核心价值**：让系统通过用户行为自我进化

```
用户选择建议
    ↓
记录反馈数据
    ↓
分析使用模式
    ↓
调整建议评分
    ↓
系统越用越智能
```

### 3.2 数据模型

#### SuggestionFeedback（建议反馈）

```rust
/// 建议反馈记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionFeedback {
    /// 反馈 ID
    pub id: String,

    /// 建议内容
    pub suggestion: String,

    /// 建议来源
    pub source: SuggestionSource,

    /// 原始评分
    pub original_score: f64,

    /// 用户是否选择（接受）
    pub accepted: bool,

    /// 选择的索引（如果接受）
    pub selected_index: Option<usize>,

    /// 上下文信息
    pub context: FeedbackContext,

    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 反馈上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackContext {
    /// 当前目录
    pub current_dir: String,

    /// 项目类型
    pub project_type: Option<String>,

    /// 失败的命令
    pub failed_command: Option<String>,

    /// 错误输出
    pub error_output: Option<String>,

    /// 最近命令
    pub recent_commands: Vec<String>,
}
```

#### FeedbackStats（反馈统计）

```rust
/// 建议使用统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionStats {
    /// 建议命令（模式）
    pub command_pattern: String,

    /// 总展示次数
    pub shown_count: usize,

    /// 被选择次数
    pub accepted_count: usize,

    /// 接受率
    pub acceptance_rate: f64,

    /// 平均选择位置（1-based）
    pub avg_position: f64,

    /// 最后更新时间
    pub last_updated: DateTime<Utc>,
}
```

### 3.3 核心组件

#### FeedbackCollector（反馈收集器）

```rust
pub struct FeedbackCollector {
    /// 反馈存储
    storage: Arc<RwLock<FeedbackStorage>>,

    /// 统计缓存
    stats_cache: Arc<RwLock<HashMap<String, SuggestionStats>>>,
}

impl FeedbackCollector {
    /// 记录建议展示
    pub async fn record_suggestion_shown(
        &self,
        suggestions: &[Suggestion],
        context: &SuggestionContext,
    ) -> Result<String> { ... }

    /// 记录用户选择
    pub async fn record_selection(
        &self,
        feedback_id: &str,
        selected_index: usize,
    ) -> Result<()> { ... }

    /// 记录用户跳过（未选择）
    pub async fn record_skip(&self, feedback_id: &str) -> Result<()> { ... }

    /// 获取建议的统计数据
    pub async fn get_stats(&self, command_pattern: &str) -> Option<SuggestionStats> { ... }
}
```

#### FeedbackLearner（反馈学习器）

```rust
pub struct FeedbackLearner {
    /// 反馈收集器
    collector: Arc<FeedbackCollector>,

    /// 学习配置
    config: LearningConfig,
}

impl FeedbackLearner {
    /// 根据反馈调整建议评分
    pub async fn adjust_score(
        &self,
        suggestion: &Suggestion,
        context: &SuggestionContext,
    ) -> f64 {
        let base_score = suggestion.score;

        // 获取统计数据
        let stats = self.collector.get_stats(&suggestion.command).await;

        if let Some(stats) = stats {
            // 根据接受率调整
            let acceptance_boost = stats.acceptance_rate * 0.2;

            // 根据位置调整（越靠前越好）
            let position_penalty = (stats.avg_position - 1.0) * 0.05;

            // 计算调整后的分数
            let adjusted = base_score + acceptance_boost - position_penalty;

            adjusted.clamp(0.0, 1.0)
        } else {
            base_score
        }
    }

    /// 学习常用模式
    pub async fn learn_patterns(&self) -> Vec<CommandPattern> { ... }
}
```

### 3.4 集成方式

**步骤1：记录建议展示**

```rust
// src/agent.rs - 建议显示后
if !suggestions.is_empty() && self.config.features.auto_suggest.unwrap_or(true) {
    println!("\n{}", "💡 建议尝试：".yellow().bold());
    for (i, suggestion) in suggestions.iter().take(3).enumerate() {
        println!("  {}. {} {}", i + 1, suggestion.category.icon(), suggestion.command.cyan());
    }

    // ✨ P2.1: 记录建议展示
    if let Some(ref collector) = self.feedback_collector {
        let feedback_id = collector.record_suggestion_shown(&suggestions, &ctx).await?;
        // 保存 feedback_id 用于后续记录选择
        *self.current_feedback_id.write().await = Some(feedback_id);
    }
}
```

**步骤2：记录用户选择**

```rust
// src/agent.rs - try_execute_cached_suggestion
if let Some(suggestions) = cache.get() {
    if index < suggestions.len() {
        // ✨ P2.1: 记录用户选择
        if let Some(ref collector) = self.feedback_collector {
            if let Some(ref feedback_id) = *self.current_feedback_id.read().await {
                collector.record_selection(feedback_id, index).await?;
            }
        }

        Ok(suggestions[index].command.clone())
    }
}
```

**步骤3：调整建议评分**

```rust
// src/suggestion/ranker.rs
pub fn rank(&self, mut suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
    // 原有排序逻辑...

    // ✨ P2.1: 根据用户反馈调整分数
    if let Some(ref learner) = self.feedback_learner {
        for suggestion in &mut suggestions {
            suggestion.score = learner.adjust_score(suggestion, &self.context).await;
        }

        // 重新排序
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    }

    suggestions
}
```

### 3.5 存储设计

**文件位置**：`~/.realconsole/feedback/`

```
~/.realconsole/feedback/
├── feedbacks.json          # 反馈记录
├── stats.json             # 统计数据
└── patterns.json          # 学习到的模式
```

**数据格式**（示例）：

```json
// feedbacks.json
[
  {
    "id": "fb_20251027_123456",
    "suggestion": "cargo build",
    "source": "Context",
    "original_score": 0.85,
    "accepted": true,
    "selected_index": 0,
    "context": {
      "current_dir": "/path/to/project",
      "project_type": "RustProject",
      "failed_command": "cago build"
    },
    "timestamp": "2025-10-27T12:34:56Z"
  }
]

// stats.json
{
  "cargo build": {
    "command_pattern": "cargo build",
    "shown_count": 10,
    "accepted_count": 8,
    "acceptance_rate": 0.8,
    "avg_position": 1.2,
    "last_updated": "2025-10-27T12:34:56Z"
  }
}
```

### 3.6 "一分为三"哲学体现

**三态反馈**：
```
接受（Accepted）   →  积极信号，提升评分
跳过（Skipped）    →  中性信号，保持评分
拒绝（Rejected）   →  消极信号，降低评分（未来功能）
```

**三层学习**：
```
即时学习（Instant）    →  单次反馈立即调整
短期学习（Short-term）  →  最近 N 次反馈的模式
长期学习（Long-term）   →  历史数据的趋势分析
```

---

## 四、功能2：智能参数补全

### 4.1 设计目标

**问题**：建议中的占位符需要手动填写

```bash
# 当前
💡 建议尝试：
  1. git clone <url>      # 用户需要手动替换 <url>
  2. ssh <user>@<host>    # 用户需要手动替换 <user> 和 <host>
```

**目标**：自动填充占位符

```bash
# 改进后
💡 建议尝试：
  1. git clone https://github.com/user/repo.git  # 从剪贴板或上下文推断
  2. ssh user@192.168.1.100                      # 从历史命令推断
```

### 4.2 核心组件

#### ParameterExtractor（参数提取器）

```rust
/// 参数占位符
#[derive(Debug, Clone)]
pub enum Placeholder {
    Url,              // <url>
    Host,             // <host>
    User,             // <user>
    Port,             // <port>
    File,             // <file>
    Dir,              // <dir>
    Command,          // <cmd>
    Custom(String),   // <custom>
}

pub struct ParameterExtractor {
    /// 上下文
    context: Arc<SuggestionContext>,

    /// 历史命令
    history: Arc<RwLock<HistoryManager>>,
}

impl ParameterExtractor {
    /// 识别命令中的占位符
    pub fn find_placeholders(&self, command: &str) -> Vec<(Placeholder, usize)> {
        // 正则匹配 <xxx> 格式
        let re = Regex::new(r"<(\w+)>").unwrap();
        re.captures_iter(command)
            .map(|cap| {
                let name = cap.get(1).unwrap().as_str();
                let pos = cap.get(0).unwrap().start();
                (Placeholder::from_str(name), pos)
            })
            .collect()
    }

    /// 从上下文推断参数值
    pub async fn infer_parameter(&self, placeholder: &Placeholder) -> Option<String> {
        match placeholder {
            Placeholder::Url => self.infer_url().await,
            Placeholder::Host => self.infer_host().await,
            Placeholder::User => self.infer_user().await,
            Placeholder::File => self.infer_file().await,
            _ => None,
        }
    }

    /// 推断 URL（从剪贴板、浏览器、历史）
    async fn infer_url(&self) -> Option<String> {
        // 1. 检查剪贴板
        if let Ok(clipboard) = ClipboardContext::new() {
            if let Ok(content) = clipboard.get_contents() {
                if content.starts_with("http") {
                    return Some(content);
                }
            }
        }

        // 2. 从最近的命令中提取 URL
        let history = self.history.read().await;
        for entry in history.recent(10, SortStrategy::Time) {
            if let Some(url) = extract_url(&entry.command) {
                return Some(url);
            }
        }

        None
    }
}
```

#### ParameterFiller（参数填充器）

```rust
pub struct ParameterFiller {
    extractor: ParameterExtractor,
}

impl ParameterFiller {
    /// 填充命令中的占位符
    pub async fn fill_placeholders(&self, command: &str) -> String {
        let mut result = command.to_string();
        let placeholders = self.extractor.find_placeholders(command);

        for (placeholder, _) in placeholders {
            if let Some(value) = self.extractor.infer_parameter(&placeholder).await {
                let pattern = format!("<{}>", placeholder.name());
                result = result.replace(&pattern, &value);
            }
        }

        result
    }

    /// 填充建议列表
    pub async fn fill_suggestions(&self, suggestions: Vec<Suggestion>) -> Vec<Suggestion> {
        let mut filled = Vec::new();

        for mut suggestion in suggestions {
            if suggestion.command.contains('<') {
                // 创建原始版本（用户可以选择）
                let original = suggestion.clone();

                // 填充参数
                suggestion.command = self.fill_placeholders(&suggestion.command).await;
                suggestion.description = format!("{} (自动填充)", suggestion.description);

                // 如果填充成功（没有剩余占位符），使用填充版本
                if !suggestion.command.contains('<') {
                    filled.push(suggestion);
                } else {
                    // 填充失败，保留原始版本
                    filled.push(original);
                }
            } else {
                filled.push(suggestion);
            }
        }

        filled
    }
}
```

### 4.3 集成方式

```rust
// src/suggestion/engine.rs
pub async fn suggest(&self, context: &SuggestionContext) -> Vec<Suggestion> {
    // 原有建议生成逻辑
    let mut suggestions = Vec::new();
    suggestions.extend(self.context_suggester.suggest(context).await);
    suggestions.extend(self.history_suggester.suggest(context).await);
    suggestions.extend(self.llm_suggester.suggest(context).await);

    // 排序
    suggestions = self.ranker.rank(suggestions);

    // ✨ P2.2: 智能参数补全
    if self.config.enable_parameter_filling {
        suggestions = self.parameter_filler.fill_suggestions(suggestions).await;
    }

    suggestions
}
```

### 4.4 推断策略

| 占位符 | 推断来源 | 优先级 |
|--------|---------|--------|
| `<url>` | 1. 剪贴板<br>2. 历史命令<br>3. 浏览器历史 | 高 |
| `<host>` | 1. 历史 SSH 命令<br>2. 当前网络<br>3. ~/.ssh/config | 高 |
| `<user>` | 1. $USER 环境变量<br>2. 历史命令 | 高 |
| `<file>` | 1. 当前目录最近修改<br>2. 失败命令中的文件 | 中 |
| `<dir>` | 1. 当前目录<br>2. 最近访问的目录 | 中 |
| `<port>` | 1. 常用端口（3000, 8080）<br>2. 历史命令 | 低 |

---

## 五、功能3：上下文链式建议

### 5.1 设计目标

**问题**：建议是独立的，不考虑执行结果

```bash
> cargo build
Error: Cargo.toml not found

💡 建议尝试：
  1. cargo init    # 建议初始化项目

> 1               # 用户执行
Created binary project

# 当前：没有后续建议
# 理想：自动建议下一步
💡 接下来可以：
  1. cargo build   # 现在可以构建了
  2. cargo run     # 或者直接运行
```

### 5.2 核心组件

#### ChainedSuggester（链式建议器）

```rust
pub struct ChainedSuggester {
    /// 命令执行历史（带结果）
    execution_history: VecDeque<ExecutionRecord>,

    /// 工作流模式库
    workflow_patterns: Vec<WorkflowPattern>,
}

/// 执行记录
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    command: String,
    success: bool,
    output: String,
    timestamp: DateTime<Utc>,
}

/// 工作流模式
#[derive(Debug, Clone)]
pub struct WorkflowPattern {
    name: String,
    steps: Vec<WorkflowStep>,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    command_pattern: String,
    expected_result: ResultPattern,
    next_suggestions: Vec<String>,
}

impl ChainedSuggester {
    /// 根据执行结果生成下一步建议
    pub async fn suggest_next(
        &self,
        last_execution: &ExecutionRecord,
        context: &SuggestionContext,
    ) -> Vec<Suggestion> {
        // 匹配工作流模式
        for pattern in &self.workflow_patterns {
            if let Some(next_step) = pattern.match_and_suggest(last_execution) {
                return next_step;
            }
        }

        // 使用通用规则
        self.generic_next_suggestions(last_execution, context).await
    }
}
```

### 5.3 预定义工作流

```rust
// 示例：Git 工作流
WorkflowPattern {
    name: "Git Initialize Workflow",
    steps: vec![
        WorkflowStep {
            command_pattern: "git init",
            expected_result: ResultPattern::Success,
            next_suggestions: vec![
                "git add .",
                "git commit -m 'Initial commit'",
            ],
        },
        WorkflowStep {
            command_pattern: "git commit",
            expected_result: ResultPattern::Success,
            next_suggestions: vec![
                "git remote add origin <url>",
                "git push -u origin main",
            ],
        },
    ],
}

// 示例：Rust 项目工作流
WorkflowPattern {
    name: "Rust Project Workflow",
    steps: vec![
        WorkflowStep {
            command_pattern: "cargo init",
            expected_result: ResultPattern::Success,
            next_suggestions: vec![
                "cargo build",
                "cargo run",
            ],
        },
        WorkflowStep {
            command_pattern: "cargo build",
            expected_result: ResultPattern::Error("Cargo.toml not found"),
            next_suggestions: vec![
                "cargo init",
            ],
        },
    ],
}
```

---

## 六、实施计划

### 6.1 P2.1 - 学习用户反馈（2周）

**Week 1**：
- [ ] Day 1-2: 设计数据模型和存储
- [ ] Day 3-4: 实现 FeedbackCollector
- [ ] Day 5: 实现 FeedbackStorage

**Week 2**：
- [ ] Day 1-3: 实现 FeedbackLearner
- [ ] Day 4: 集成到 Agent 和 SuggestionEngine
- [ ] Day 5: 测试和优化

**交付物**：
- `src/suggestion/feedback/mod.rs`
- `src/suggestion/feedback/collector.rs`
- `src/suggestion/feedback/learner.rs`
- `src/suggestion/feedback/storage.rs`
- 单元测试 + 集成测试

### 6.2 P2.2 - 智能参数补全（1.5周）

**Week 1**：
- [ ] Day 1-2: 实现 ParameterExtractor
- [ ] Day 3-4: 实现 ParameterFiller
- [ ] Day 5: 实现推断策略（URL、Host、User）

**Week 2 (前半周)**：
- [ ] Day 1-2: 集成到 SuggestionEngine
- [ ] Day 3: 测试和优化

**交付物**：
- `src/suggestion/parameter_filler.rs`
- 单元测试 + 集成测试

### 6.3 P2.3 - 上下文链式建议（3周，可选）

**Week 1**：
- [ ] 设计工作流模式
- [ ] 实现 ExecutionRecord 追踪

**Week 2**：
- [ ] 实现 ChainedSuggester
- [ ] 构建预定义工作流

**Week 3**：
- [ ] 集成测试
- [ ] 工作流优化

---

## 七、成功指标

### 7.1 P2.1 - 学习用户反馈

**定量指标**：
- 反馈记录覆盖率 > 90%
- 评分调整准确率 > 70%
- 系统响应延迟 < 10ms

**定性指标**：
- 建议质量随使用时间提升
- 用户常用命令优先级提高

### 7.2 P2.2 - 智能参数补全

**定量指标**：
- 参数推断成功率 > 60%
- URL 推断成功率 > 80%
- Host/User 推断成功率 > 50%

**定性指标**：
- 减少手动编辑次数
- 提升建议的直接可用性

### 7.3 P2.3 - 上下文链式建议

**定量指标**：
- 工作流匹配率 > 40%
- 下一步建议准确率 > 60%

**定性指标**：
- 用户感知流程更连贯
- 减少中间步骤思考时间

---

## 八、风险与缓解

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| 反馈数据隐私 | 高 | 低 | 本地存储，加密敏感数据 |
| 参数推断错误 | 中 | 中 | 提供原始版本选项 |
| 工作流模式过时 | 中 | 中 | 定期更新模式库 |
| 性能影响 | 低 | 低 | 异步处理，缓存优化 |

---

## 九、下一步行动

### 立即开始（推荐）

**选项 A：P2.1 - 学习用户反馈**
- 投入产出比最高
- 为后续功能提供基础
- 2周可交付

**选项 B：P2.2 - 智能参数补全**
- 用户价值明显
- 实现相对简单
- 1.5周可交付

**选项 C：全部实施（顺序）**
- P2.1 (2周) → P2.2 (1.5周) → P2.3 (3周)
- 总计 6.5周

### 你的选择？

请选择：
1. 开始 P2.1（学习用户反馈）
2. 开始 P2.2（智能参数补全）
3. 查看更多细节后再决定
4. 调整优先级或设计

---

**文档版本**: v1.0
**最后更新**: 2025-10-27
**作者**: RealConsole P2 设计团队
