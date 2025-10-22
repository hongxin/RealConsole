# RealConsole 连续场重构设计 - 从离散到连续

**创建日期**: 2025-10-22
**版本**: 1.0 (设计阶段)
**状态**: 📝 纸面设计
**哲学基础**: [think.md](../../00-core/think.md) - "一分为三"的深层理解

---

## 🎯 重构目标

基于 [think.md](../../00-core/think.md) 的哲学洞察，将 RealConsole 的核心架构从**离散状态机**演化为**连续势能场**。

**核心原则**：
1. **定义势能场，而非离散状态**
2. **用渐变替代跳变**
3. **多维向量决策，而非 if/else 树**
4. **保持演化空间的开放性**

---

## 🔍 当前架构分析：识别"离散化"痕迹

### 1. ContextMode 的硬三态

**当前实现** (`src/config.rs:318-327`):
```rust
pub enum ContextMode {
    Disabled,
    Manual,
    Auto,
}
```

**问题诊断**：
- ❌ 三个离散状态，无法表达"50% Auto"或"渐变激活"
- ❌ Disabled/Manual/Auto 是硬切换，缺少过渡区
- ❌ 用户无法微调"自动化程度"

**哲学对照** ([think.md 第112-132行](../../00-core/think.md#112-132)):
> 不是三个离散的开关，而是**上下文感知这个连续维度上的不同配置**。

**真实连续谱**：
```
上下文感知强度
  0%  ────────────────────────────────── 100%
  │                    │                   │
Disabled          Manual              Auto
(显式否定)      (半自动感知)      (全自动感知)
```

---

### 2. should_use_context 的硬决策树

**当前实现** (`src/conversation/context_manager.rs:149-174`):
```rust
pub fn should_use_context(&mut self, input: &str) -> bool {
    match self.config.mode {
        ContextMode::Disabled => false,
        ContextMode::Manual => self.is_active,
        ContextMode::Auto => {
            // 复杂逻辑，但最终返回 bool
        }
    }
}
```

**问题诊断**：
- ❌ 返回 `bool`：要么用、要么不用，无中间态
- ❌ `match` 分支决策，离散跳变
- ❌ 无法表达"轻度使用上下文"（如只用最近1轮）

**理想设计**：
```rust
// 返回上下文强度 (0.0 - 1.0)
pub fn calculate_context_strength(&mut self, input: &str) -> f64 {
    // 基于多个因素计算连续强度
    // 然后按权重应用上下文
}
```

---

### 3. should_enable_context 的关键词硬触发

**当前实现** (`src/conversation/context_manager.rs:105-141`):
```rust
pub fn should_enable_context(&self, input: &str) -> bool {
    let pronouns = ["它", "这个", "那个", ...];
    if pronouns.iter().any(|p| input_lower.contains(p)) {
        return true;  // ⚠️ 100% 触发
    }
    // ... 更多关键词检测
    false
}
```

**问题诊断**：
- ❌ 布尔返回：发现关键词就 100% 启用上下文
- ❌ 关键词列表 hard-coded，无权重
- ❌ 没有"置信度"概念：检测到"它"可能只需要 30% 上下文强度

**理想设计**：
```rust
pub fn calculate_context_need(&self, input: &str) -> f64 {
    let mut score = 0.0;

    // 代词：轻度需求
    if has_pronouns(input) {
        score += 0.3;
    }

    // 追问：中度需求
    if has_followup(input) {
        score += 0.5;
    }

    // 上下文依赖词：高度需求
    if has_context_ref(input) {
        score += 0.8;
    }

    score.min(1.0)  // 返回 0.0-1.0 的连续值
}
```

---

### 4. 布尔值的二元对立

**当前实现**：
```rust
pub enabled: bool,              // 开/关
pub is_active: bool,            // 激活/未激活
pub auto_clear.enabled: bool,   // 自动清除 是/否
```

**问题诊断**：
- ❌ 无法表达"部分激活"、"逐渐激活"
- ❌ 无法实现"渐变开启"（如从 0% 逐步增加到 100%）

**理想设计**：
```rust
pub activation_level: f64,      // 0.0 - 1.0 激活程度
pub auto_clear_probability: f64, // 0.0 - 1.0 清除概率
```

---

### 5. 硬阈值的突变行为

**当前实现**：
```rust
pub validation_threshold: f64,  // 0.7
pub idle_timeout: u64,          // 600 秒
pub max_turns: usize,           // 10
```

**问题诊断**：
- ❌ `threshold: 0.7` → 0.69 失败，0.71 成功（断崖式）
- ❌ `idle_timeout: 600` → 599秒继续，601秒清除（跳变）
- ❌ `max_turns: 10` → 第9轮OK，第10轮截断（硬边界）

**理想设计**：
```rust
// 使用软阈值和衰减函数
fn calculate_acceptance_probability(score: f64, threshold: f64, softness: f64) -> f64 {
    // sigmoid 函数：在阈值附近平滑过渡
    1.0 / (1.0 + (-(score - threshold) / softness).exp())
}

// 渐变清除：不是"601秒立即清除"，而是"概率逐渐增加"
fn calculate_clear_probability(idle_seconds: i64, timeout: i64) -> f64 {
    if idle_seconds < timeout / 2 {
        0.0  // 前半段：不清除
    } else {
        // 后半段：概率从 0 → 1 渐变
        let progress = (idle_seconds - timeout / 2) as f64 / (timeout / 2) as f64;
        progress.min(1.0)
    }
}
```

---

## 🎨 重构设计方案

### 方案一：连续化上下文感知系统

#### 1.1 新的配置结构（势能场参数）

**不要这样（离散）**:
```rust
pub enum ContextMode {
    Disabled,
    Manual,
    Auto
}
```

**而是这样（连续）**:
```rust
/// 上下文感知场配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAwarenessField {
    /// 基础敏感度 (0.0 - 1.0)
    /// 0.0 = 完全关闭（等价于旧 Disabled）
    /// 0.5 = 中等感知（等价于旧 Manual）
    /// 1.0 = 全自动感知（等价于旧 Auto）
    #[serde(default = "default_sensitivity")]
    pub sensitivity: f64,

    /// 自动触发阈值 (0.0 - 1.0)
    /// 当"需要上下文"的置信度 >= 此值时自动启用
    #[serde(default = "default_auto_threshold")]
    pub auto_threshold: f64,

    /// 上下文衰减速率 (每秒衰减百分比)
    /// 控制上下文强度如何随时间衰减
    #[serde(default = "default_decay_rate")]
    pub decay_rate: f64,

    /// 最大上下文强度 (0.0 - 1.0)
    /// 限制上下文的最大影响力
    #[serde(default = "default_max_strength")]
    pub max_strength: f64,
}

fn default_sensitivity() -> f64 { 0.0 }      // 默认关闭（向后兼容）
fn default_auto_threshold() -> f64 { 0.6 }  // 60% 置信度触发
fn default_decay_rate() -> f64 { 0.001 }    // 每秒衰减 0.1%
fn default_max_strength() -> f64 { 1.0 }    // 允许全强度
```

#### 1.2 向后兼容层

为保持用户配置兼容性，提供迁移逻辑：

```rust
impl From<ContextMode> for ContextAwarenessField {
    fn from(mode: ContextMode) -> Self {
        match mode {
            ContextMode::Disabled => ContextAwarenessField {
                sensitivity: 0.0,
                auto_threshold: 1.0,  // 永不自动触发
                decay_rate: 0.01,     // 快速衰减
                max_strength: 0.0,    // 零强度
            },
            ContextMode::Manual => ContextAwarenessField {
                sensitivity: 0.5,
                auto_threshold: 1.0,  // 不自动触发（需手动启动）
                decay_rate: 0.001,
                max_strength: 0.8,    // 中等强度
            },
            ContextMode::Auto => ContextAwarenessField {
                sensitivity: 1.0,
                auto_threshold: 0.6,  // 较低阈值，容易触发
                decay_rate: 0.0005,   // 缓慢衰减
                max_strength: 1.0,    // 全强度
            },
        }
    }
}
```

#### 1.3 核心方法重构

**旧实现（离散）**:
```rust
pub fn should_use_context(&mut self, input: &str) -> bool {
    match self.config.mode {
        ContextMode::Disabled => false,
        ContextMode::Manual => self.is_active,
        ContextMode::Auto => { /* ... */ }
    }
}
```

**新实现（连续）**:
```rust
/// 计算当前上下文场强度
pub fn calculate_context_strength(&mut self, input: &str) -> f64 {
    let field = &self.config.awareness_field;

    // 1. 基础强度（配置的敏感度）
    let mut strength = field.sensitivity;

    // 2. 输入驱动的增强
    let need_score = self.calculate_context_need(input);

    // 自动触发检查
    if need_score >= field.auto_threshold {
        strength = strength.max(need_score);
    }

    // 3. 历史上下文的增强
    if !self.turns.is_empty() {
        // 有历史上下文时，增强当前强度
        let history_boost = (self.turns.len() as f64 / self.config.max_turns as f64) * 0.3;
        strength += history_boost;
    }

    // 4. 时间衰减
    let idle = self.idle_seconds() as f64;
    let decay = (-field.decay_rate * idle).exp();
    strength *= decay;

    // 5. 限制在最大强度内
    strength.min(field.max_strength).max(0.0)
}

/// 计算输入需要上下文的程度（0.0 - 1.0）
fn calculate_context_need(&self, input: &str) -> f64 {
    let input_lower = input.to_lowercase();
    let mut score = 0.0;

    // 代词检测：轻度需求 (+0.3)
    let pronouns = ["它", "这个", "那个", "this", "that", "it"];
    if pronouns.iter().any(|p| input_lower.contains(p)) {
        score += 0.3;
    }

    // 追问检测：中度需求 (+0.5)
    let followups = ["为什么", "继续", "详细", "why", "continue", "more"];
    if followups.iter().any(|f| input_lower.contains(f)) {
        score += 0.5;
    }

    // 上下文依赖词：高度需求 (+0.7)
    let context_refs = ["刚才", "之前", "earlier", "previous"];
    if context_refs.iter().any(|c| input_lower.contains(c)) {
        score += 0.7;
    }

    score.min(1.0)
}

/// 应用上下文（按强度加权）
pub fn build_messages_with_strength(
    &self,
    current_input: &str,
    strength: f64
) -> Vec<Message> {
    let mut messages = Vec::new();

    if strength < 0.1 {
        // 强度太低，不使用上下文
        messages.push(Message::user(current_input));
        return messages;
    }

    // 根据强度决定包含多少历史轮次
    let turns_to_include = (self.turns.len() as f64 * strength).ceil() as usize;
    let start_index = self.turns.len().saturating_sub(turns_to_include);

    // 包含选定的历史轮次
    for turn in self.turns.iter().skip(start_index) {
        messages.push(Message::user(&turn.user_input));
        messages.push(Message::assistant(&turn.assistant_response));
    }

    // 当前输入
    messages.push(Message::user(current_input));

    messages
}
```

---

### 方案二：软阈值决策系统

#### 2.1 替换硬阈值为软边界

**旧实现（硬阈值）**:
```rust
if confidence >= self.config.validation_threshold {
    // 接受
} else {
    // 拒绝
}
```

**新实现（软边界）**:
```rust
/// 计算接受概率（sigmoid 函数）
fn acceptance_probability(score: f64, threshold: f64, softness: f64) -> f64 {
    // softness 控制过渡区宽度
    // softness = 0.1: 陡峭过渡（接近硬阈值）
    // softness = 0.5: 平缓过渡（宽容）
    1.0 / (1.0 + (-(score - threshold) / softness).exp())
}

// 使用示例
let accept_prob = acceptance_probability(confidence, 0.7, 0.15);

if accept_prob > 0.9 {
    // 高置信度：直接接受
} else if accept_prob > 0.5 {
    // 中等置信度：提示用户确认
} else if accept_prob > 0.1 {
    // 低置信度：警告 + 确认
} else {
    // 极低置信度：拒绝
}
```

#### 2.2 渐变清除策略

**旧实现（硬超时）**:
```rust
if idle_seconds > timeout {
    self.clear();  // 突然清除
}
```

**新实现（概率清除）**:
```rust
/// 计算清除概率（随时间平滑增加）
fn calculate_clear_probability(&self) -> f64 {
    let idle = self.idle_seconds();
    let timeout = self.config.auto_clear.idle_timeout as i64;

    if idle < timeout / 2 {
        0.0  // 前半段：不清除
    } else {
        // 后半段：从 0 → 1 线性增长
        let progress = (idle - timeout / 2) as f64 / (timeout / 2) as f64;
        progress.min(1.0)
    }
}

/// 带概率的清除检查
pub fn maybe_cleanup(&mut self) {
    let clear_prob = self.calculate_clear_probability();

    if clear_prob > 0.95 {
        // 几乎确定清除
        self.clear();
    } else if clear_prob > 0.5 {
        // 中等概率：开始衰减上下文（而非直接清除）
        self.decay_context(clear_prob);
    }
    // 否则：保持不变
}

/// 衰减上下文（渐变）
fn decay_context(&mut self, decay_factor: f64) {
    // 按概率移除最旧的轮次
    if !self.turns.is_empty() && rand::random::<f64>() < decay_factor {
        self.turns.pop_front();
    }
}
```

---

## 🧪 实验性：多维向量决策

### 3.1 Intent 匹配的向量空间

**旧实现（单一置信度）**:
```rust
pub struct MatchResult {
    pub confidence: f64,  // 单一维度
    // ...
}
```

**新实现（多维向量）**:
```rust
pub struct IntentVector {
    /// 语义匹配度 (0.0 - 1.0)
    pub semantic_match: f64,

    /// 风险评估 (0.0 - 1.0, 越高越危险)
    pub risk_level: f64,

    /// 用户熟练度相关性 (0.0 - 1.0)
    pub user_skill_fit: f64,

    /// 上下文相关性 (0.0 - 1.0)
    pub context_relevance: f64,
}

impl IntentVector {
    /// 计算综合决策分数（加权）
    pub fn decision_score(&self, weights: &DecisionWeights) -> f64 {
        self.semantic_match * weights.semantic
            + (1.0 - self.risk_level) * weights.safety  // 风险越低越好
            + self.user_skill_fit * weights.skill
            + self.context_relevance * weights.context
    }
}

/// 决策权重配置
#[derive(Debug, Clone)]
pub struct DecisionWeights {
    pub semantic: f64,
    pub safety: f64,
    pub skill: f64,
    pub context: f64,
}

impl Default for DecisionWeights {
    fn default() -> Self {
        Self {
            semantic: 0.4,   // 语义最重要
            safety: 0.3,     // 安全次之
            skill: 0.2,      // 用户适配
            context: 0.1,    // 上下文辅助
        }
    }
}
```

---

## 📋 实施计划

### Phase 1: 基础重构（保守，向后兼容）✅ 优先

1. **引入 ContextAwarenessField**
   - 添加新配置结构
   - 保留旧的 `ContextMode` enum
   - 提供 `From<ContextMode>` 转换
   - 文件：`src/config.rs`

2. **实现连续计算方法**
   - `calculate_context_strength()`
   - `calculate_context_need()`
   - `build_messages_with_strength()`
   - 文件：`src/conversation/context_manager.rs`

3. **向后兼容适配器**
   - 保留旧的 `should_use_context()` 方法
   - 内部调用新的连续方法
   - 确保现有测试通过

### Phase 2: 软阈值系统

4. **实现软边界函数**
   - `acceptance_probability()`
   - `calculate_clear_probability()`
   - 工具函数：`src/utils/soft_threshold.rs`

5. **集成到 Intent 系统**
   - 修改验证逻辑使用软阈值
   - 文件：`src/dsl/intent/validator.rs`

### Phase 3: 多维决策（实验性）

6. **引入 IntentVector**
   - 多维匹配向量
   - 可配置的决策权重
   - 文件：`src/dsl/intent/vector.rs`

7. **集成测试与调优**
   - A/B 测试：离散 vs 连续
   - 收集用户反馈
   - 调整默认参数

---

## 🎯 预期效果

### 用户体验改进

1. **更自然的渐变**
   - 上下文不再"突然启用"或"突然清除"
   - 感知到系统的"思考过程"

2. **更灵活的控制**
   - 用户可以微调"自动化程度"（sensitivity 参数）
   - 不再局限于三个固定档位

3. **更智能的决策**
   - 系统能够"部分使用上下文"（如只用最近2轮）
   - 在不确定时表现出"犹豫"而非"跳变"

### 技术优势

1. **可调试性增强**
   - 每个决策都有连续的强度值，便于追踪
   - 可视化"势能场"状态

2. **可扩展性**
   - 易于添加新的影响因素（如时间、地点、用户情绪）
   - 每个因素都是独立的维度

3. **演化空间**
   - 为未来的机器学习优化留下接口
   - 可以用数据驱动调整权重和参数

---

## 🚧 风险与挑战

### 技术风险

1. **性能开销**
   - 连续计算比简单 if/else 稍慢
   - **缓解**: 计算结果缓存，避免重复计算

2. **参数调优难度**
   - 需要找到合适的默认值
   - **缓解**: 提供多个预设配置（conservative/balanced/aggressive）

3. **测试复杂度**
   - 连续系统的边界情况更多
   - **缓解**: 使用基于属性的测试（property-based testing）

### 用户迁移

1. **配置迁移**
   - 旧配置文件需要平滑转换
   - **方案**: 自动检测旧格式，提示升级

2. **行为变化**
   - 用户需要适应新的"渐变"行为
   - **方案**: 提供"经典模式"选项（完全模拟旧行为）

---

## 📚 相关文档

- **哲学基础**: [think.md](../../00-core/think.md) - "一分为三"的深层理解
- **设计哲学**: [philosophy.md](../../00-core/philosophy.md) - 一分为三哲学
- **上下文最佳实践**: [context-mode-best-practices.md](../../02-practice/user/context-mode-best-practices.md)
- **技术债务追踪**: [technical-debt.md](../analysis/technical-debt.md)

---

## 🔄 迭代记录

### v1.0 (2025-10-22)
- ✅ 完成代码分析，识别离散化痕迹
- ✅ 提出连续场重构方案
- ✅ 设计向后兼容策略
- ⏳ 待实施代码重构

---

**创建**: 2025-10-22
**维护**: RealConsole Contributors
**许可**: MIT

**这是一份活的设计文档，会随着实施过程持续演化。** 🌱
