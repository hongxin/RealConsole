# Workflow Intent 系统集成方案

**创建日期**: 2025-10-18
**状态**: 设计阶段
**优先级**: 向后兼容性第一

---

## 设计原则

### 1. **完全向后兼容** 🛡️
- 默认情况下不启用 Workflow Intent
- 现有功能路径完全不受影响
- 用户需要**明确配置**才能使用新功能

### 2. **渐进式增强** 📈
- 作为现有 Intent DSL 的增强层
- 匹配失败时平滑回退到现有流程
- 不引入 breaking changes

### 3. **配置可控** ⚙️
- 通过配置文件控制是否启用
- 支持运行时动态切换
- 提供配置向导辅助设置

---

## 现有决策链分析

### 当前 handle_text() 流程（src/agent.rs:750-777）

```rust
fn handle_text(&self, text: &str) -> String {
    // 1️⃣ 对话态：如果有活跃对话，继续对话流程
    if has_active_conversation() {
        return self.handle_conversation_input(text);
    }

    // 2️⃣ 检测是否需要启动新对话
    if let Some(response) = self.try_start_conversation(text) {
        return response;
    }

    // 3️⃣ 工具调用态：如果启用 tool_calling_enabled
    if self.config.features.tool_calling_enabled.unwrap_or(false) {
        return self.handle_text_with_tools(text);
    }

    // 4️⃣ Intent DSL 态：尝试匹配 Intent
    if let Some(plan) = self.try_match_intent(text) {
        return self.execute_intent(&plan);
    }

    // 5️⃣ 流式态：最后回退到流式 LLM
    self.handle_text_streaming(text)
}
```

**特点**:
- **优先级明确**: 对话 > 工具调用 > Intent > 流式
- **逐级回退**: 每一级未匹配则进入下一级
- **已经很成熟**: 经过充分测试，稳定可靠

---

## 集成方案设计

### 方案：在 3️⃣ 和 4️⃣ 之间插入 Workflow Intent

#### 新的决策链

```rust
fn handle_text(&self, text: &str) -> String {
    // 1️⃣ 对话态（不变）
    if has_active_conversation() {
        return self.handle_conversation_input(text);
    }

    // 2️⃣ 检测新对话（不变）
    if let Some(response) = self.try_start_conversation(text) {
        return response;
    }

    // 3️⃣ 工具调用态（不变）
    let use_tools = self.config.features.tool_calling_enabled.unwrap_or(false);
    if use_tools {
        return self.handle_text_with_tools(text);
    }

    // ✨ 3.5️⃣ Workflow Intent 态（新增，默认禁用）
    if self.config.features.workflow_enabled.unwrap_or(false) {
        if let Some(response) = self.try_match_workflow(text) {
            return response;
        }
        // 未匹配到 Workflow，继续回退到传统 Intent
    }

    // 4️⃣ Intent DSL 态（不变，作为回退）
    if let Some(plan) = self.try_match_intent(text) {
        return self.execute_intent(&plan);
    }

    // 5️⃣ 流式态（不变，最后回退）
    self.handle_text_streaming(text)
}
```

#### 插入位置的理由

**为什么在 3️⃣ 和 4️⃣ 之间？**

1. **在工具调用之后**：
   - 工具调用模式是**完全 LLM 驱动**的，适合复杂、未知的任务
   - Workflow Intent 是**模板化**的，适合已知套路
   - 如果用户启用了工具调用，说明想要最大的灵活性，应优先使用

2. **在传统 Intent 之前**：
   - Workflow Intent 是传统 Intent 的**增强版本**
   - 能匹配到 Workflow 的，说明是已知的高频场景，应该优先优化
   - 传统 Intent 作为**回退选项**，保证兼容性

3. **在流式之前**：
   - 流式是最后的兜底方案
   - 任何结构化的路径都应该优先于流式

---

## 配置系统设计

### 配置文件结构

```yaml
# realconsole.yaml

features:
  # 现有配置（不变）
  shell_enabled: true
  tool_calling_enabled: true
  max_tool_iterations: 5
  max_tools_per_round: 3

  # ✨ 新增配置
  workflow_enabled: false              # 是否启用 Workflow Intent（默认 false）
  workflow_cache_enabled: true         # 是否启用缓存（默认 true）
  workflow_priority: "high"            # 优先级：high（在 tool_calling 之后）、medium（在 Intent 之后）

# ✨ 新增 workflow 配置节（可选）
workflow:
  builtin_enabled: true                # 是否启用内置模板（默认 true）
  custom_template_dir: "~/.realconsole/workflows"  # 自定义模板目录
  cache_ttl: 300                       # 默认缓存 TTL（秒）
  max_iterations: 5                    # 工作流最大迭代次数
```

### Config 结构体修改

```rust
// src/config.rs

#[derive(Debug, Clone, Deserialize)]
pub struct Features {
    pub shell_enabled: bool,
    pub tool_calling_enabled: Option<bool>,
    pub max_tool_iterations: usize,
    pub max_tools_per_round: usize,

    // ✨ 新增字段（所有可选，默认 false/None）
    #[serde(default)]
    pub workflow_enabled: Option<bool>,

    #[serde(default)]
    pub workflow_cache_enabled: Option<bool>,

    #[serde(default)]
    pub workflow_priority: Option<String>,
}

// ✨ 新增 Workflow 配置节
#[derive(Debug, Clone, Deserialize)]
pub struct WorkflowConfig {
    #[serde(default = "default_true")]
    pub builtin_enabled: bool,

    #[serde(default)]
    pub custom_template_dir: Option<String>,

    #[serde(default = "default_cache_ttl")]
    pub cache_ttl: u64,

    #[serde(default = "default_max_iterations")]
    pub max_iterations: usize,
}

fn default_true() -> bool { true }
fn default_cache_ttl() -> u64 { 300 }
fn default_max_iterations() -> usize { 5 }

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    // 现有字段（不变）
    pub display: DisplayConfig,
    pub features: Features,
    pub memory: Option<MemoryConfig>,
    pub intent: IntentConfig,
    pub prefix: String,

    // ✨ 新增字段（可选）
    #[serde(default)]
    pub workflow: Option<WorkflowConfig>,
}
```

**关键点**:
- ✅ 所有新字段都是 `Option<T>`，默认值为 `None`
- ✅ 不影响现有配置文件的解析
- ✅ 用户不需要修改配置文件就能继续使用

---

## Agent 结构修改

### Agent 新增字段

```rust
// src/agent.rs

pub struct Agent {
    // 现有字段（不变）
    pub config: Config,
    pub registry: CommandRegistry,
    pub llm_manager: Arc<RwLock<LlmManager>>,
    pub memory: Arc<RwLock<Memory>>,
    pub exec_logger: Arc<RwLock<ExecutionLogger>>,
    pub tool_registry: Arc<RwLock<ToolRegistry>>,
    pub tool_executor: Arc<ToolExecutor>,
    pub intent_matcher: IntentMatcher,
    pub template_engine: TemplateEngine,
    // ... 其他现有字段 ...

    // ✨ 新增字段（可选，默认 None）
    pub workflow_intents: Option<Arc<RwLock<Vec<WorkflowIntent>>>>,
    pub workflow_executor: Option<Arc<WorkflowExecutor>>,
}
```

### Agent::new() 修改

```rust
impl Agent {
    pub fn new(config: Config, registry: CommandRegistry) -> Self {
        // ... 现有初始化逻辑（不变） ...

        // ✨ 新增：初始化 Workflow 系统（仅在启用时）
        let (workflow_intents, workflow_executor) = if config.features.workflow_enabled.unwrap_or(false) {
            // 初始化工作流意图列表
            let mut workflows = Vec::new();

            // 加载内置模板（如果启用）
            if config.workflow.as_ref()
                .and_then(|w| Some(w.builtin_enabled))
                .unwrap_or(true) {
                workflows.extend(register_builtin_workflows());
            }

            // TODO: 加载自定义模板（从配置目录）

            // 创建工作流执行器
            let executor = WorkflowExecutor::new(
                Arc::clone(&tool_registry),
                Some(Arc::clone(&llm_manager)),
            );

            (
                Some(Arc::new(RwLock::new(workflows))),
                Some(Arc::new(executor)),
            )
        } else {
            // 未启用，保持 None
            (None, None)
        };

        Self {
            // ... 现有字段初始化（不变） ...
            workflow_intents,
            workflow_executor,
        }
    }
}
```

**关键点**:
- ✅ 仅在 `workflow_enabled=true` 时初始化
- ✅ 默认情况下不占用内存
- ✅ 不影响现有用户的启动速度

---

## 核心方法实现

### try_match_workflow()

```rust
impl Agent {
    /// 尝试匹配 Workflow Intent
    ///
    /// 如果启用了 Workflow 系统，尝试匹配用户输入到工作流模板。
    /// 匹配成功则执行工作流，失败则返回 None（回退到传统 Intent）。
    ///
    /// # 返回
    /// - `Some(String)`: 匹配成功，返回执行结果
    /// - `None`: 没有匹配的工作流，应回退到传统 Intent DSL
    fn try_match_workflow(&self, text: &str) -> Option<String> {
        // 1. 检查 Workflow 系统是否初始化
        let workflow_intents = self.workflow_intents.as_ref()?;
        let workflow_executor = self.workflow_executor.as_ref()?;

        // 2. 尝试匹配工作流意图
        let workflows = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                workflow_intents.read().await
            })
        });

        // 3. 遍历所有工作流，找到最佳匹配
        let mut best_match: Option<(&WorkflowIntent, IntentMatch)> = None;
        let mut best_confidence = 0.0;

        for workflow in workflows.iter() {
            // 使用现有的 IntentMatcher 匹配基础意图
            if let Some(intent_match) = self.intent_matcher.match_intent(
                text,
                &workflow.base_intent,
            ) {
                if intent_match.confidence > best_confidence {
                    best_confidence = intent_match.confidence;
                    best_match = Some((workflow, intent_match));
                }
            }
        }

        // 4. 检查是否有足够置信度的匹配
        let (workflow, intent_match) = best_match?;
        if !intent_match.meets_threshold() {
            return None; // 置信度不足，回退
        }

        // 5. 显示匹配信息
        Display::workflow_match(
            self.config.display.mode,
            &workflow.base_intent.name,
            intent_match.confidence,
        );

        // 6. 执行工作流
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                workflow_executor.execute(workflow, &intent_match).await
            })
        });

        // 7. 处理结果
        match result {
            Ok(workflow_result) => {
                // 显示性能统计
                Display::workflow_stats(
                    self.config.display.mode,
                    workflow_result.duration_ms,
                    workflow_result.llm_calls,
                    workflow_result.tool_calls,
                );

                Some(workflow_result.output)
            }
            Err(e) => {
                // 工作流执行失败，显示错误但不中断流程
                eprintln!("{} {}", "⚠ 工作流执行失败:".yellow(), e);
                None // 回退到传统 Intent
            }
        }
    }
}
```

### 辅助方法：match_intent()

```rust
impl IntentMatcher {
    /// 尝试匹配单个意图（现有方法的增强）
    ///
    /// 与 best_match() 不同，这个方法针对指定的单个意图进行匹配
    pub fn match_intent(&self, text: &str, intent: &Intent) -> Option<IntentMatch> {
        // 复用现有的匹配逻辑
        let keywords_score = self.match_keywords(text, &intent.keywords);
        let patterns_score = self.match_patterns(text, &intent.patterns);

        // 计算综合置信度
        let confidence = (keywords_score + patterns_score) / 2.0;

        if confidence < intent.confidence_threshold {
            return None;
        }

        // 提取实体
        let extractor = EntityExtractor::new();
        let extracted_entities = extractor.extract_basic(text, &intent.entities);

        Some(IntentMatch {
            intent: intent.clone(),
            confidence,
            matched_keywords: vec![], // 简化
            extracted_entities,
        })
    }
}
```

---

## Display 增强

```rust
// src/display.rs

impl Display {
    /// 显示 Workflow 匹配信息
    pub fn workflow_match(mode: DisplayMode, workflow_name: &str, confidence: f64) {
        if mode == DisplayMode::Verbose {
            println!(
                "{} {} (置信度: {:.2})",
                "🔄 工作流:".cyan().bold(),
                workflow_name.yellow(),
                confidence
            );
        }
    }

    /// 显示 Workflow 执行统计
    pub fn workflow_stats(
        mode: DisplayMode,
        duration_ms: u64,
        llm_calls: usize,
        tool_calls: usize,
    ) {
        if mode == DisplayMode::Verbose {
            println!(
                "{} 耗时: {}ms | LLM 调用: {} 次 | 工具调用: {} 次",
                "📊".dimmed(),
                duration_ms.to_string().cyan(),
                llm_calls.to_string().yellow(),
                tool_calls.to_string().green(),
            );
        }
    }
}
```

---

## 配置向导增强

```rust
// src/wizard.rs

pub fn workflow_setup_wizard() -> WorkflowConfig {
    println!("\n{}", "=== 工作流系统配置 ===".cyan().bold());
    println!("\n{}", "工作流系统将常用任务套路化，提升性能并降低成本。".dimmed());

    // 1. 是否启用内置模板
    println!("\n{}",  "是否启用内置工作流模板? [Y/n]: ".yellow());
    let builtin_enabled = read_yes_no(true);

    // 2. 自定义模板目录
    println!("\n{}", "自定义工作流模板目录 (留空使用默认): ".yellow());
    let custom_dir = read_optional_path();

    // 3. 缓存 TTL
    println!("\n{}", "默认缓存时间 (秒) [300]: ".yellow());
    let cache_ttl = read_number(300);

    WorkflowConfig {
        builtin_enabled,
        custom_template_dir: custom_dir,
        cache_ttl,
        max_iterations: 5,
    }
}
```

---

## 兼容性测试清单

### 测试场景

#### 1. 默认配置（未启用 Workflow）
- [ ] 现有所有功能正常工作
- [ ] 不加载 Workflow 模块
- [ ] 启动速度无影响
- [ ] 内存占用无明显增加

#### 2. 启用 Workflow
- [ ] Workflow 匹配优先于传统 Intent
- [ ] 未匹配时回退到传统 Intent
- [ ] 传统 Intent 仍然可用
- [ ] 流式输出仍然可用

#### 3. Workflow + 工具调用
- [ ] 工具调用优先级高于 Workflow
- [ ] 两种模式可以共存
- [ ] 不会相互干扰

#### 4. 性能测试
- [ ] Workflow 匹配速度（<10ms）
- [ ] 缓存命中速度（<1ms）
- [ ] 未启用时无性能损失

#### 5. 错误处理
- [ ] Workflow 失败时优雅降级
- [ ] 错误信息清晰友好
- [ ] 不会导致程序崩溃

---

## 迁移路径

### 阶段 1: 可选功能（当前）
- 默认禁用
- 需要手动配置启用
- 文档说明如何启用

### 阶段 2: 试用期（1-2 个月）
- 收集用户反馈
- 优化模板和性能
- 修复发现的 bug

### 阶段 3: 逐步推广（3-6 个月）
- 在新用户中默认启用
- 为现有用户提供迁移指南
- 保持向后兼容

### 阶段 4: 完全整合（6-12 个月）
- 默认启用，但可关闭
- 成为标准功能
- 继续保持向后兼容

---

## 风险评估

### 低风险 ✅
- **配置系统**: 完全向后兼容，不会破坏现有配置
- **默认禁用**: 不影响现有用户
- **平滑回退**: 失败时自动回退到现有流程

### 中风险 ⚠️
- **代码复杂度**: 增加了一个新的决策分支
  - **缓解**: 保持代码简洁，充分测试
- **性能开销**: 增加了匹配逻辑
  - **缓解**: 仅在启用时执行，优化匹配算法

### 需要关注 📌
- **用户体验**: 新用户可能不知道如何启用
  - **缓解**: 配置向导提示，文档清晰
- **模板质量**: 内置模板需要持续优化
  - **缓解**: 收集反馈，迭代改进

---

## 总结

这个集成方案遵循**渐进式增强**原则：

1. ✅ **完全向后兼容**: 不影响任何现有功能
2. ✅ **配置可控**: 用户可以选择是否启用
3. ✅ **平滑回退**: 失败时自动降级
4. ✅ **性能优先**: 默认不启用，无额外开销
5. ✅ **易于测试**: 新旧功能独立，便于单独测试

**下一步**:
1. 实现 `try_match_workflow()` 方法
2. 修改 Config 结构添加新字段
3. 集成到 `handle_text()` 决策链
4. 编写兼容性测试
5. 更新用户文档

---

**创建人**: RealConsole Team + Claude Code
**最后更新**: 2025-10-18
