# 两仪状态演化系统设计

**日期**: 2025-10-28
**版本**: v1.9.0-alpha
**哲学基础**: 易经·太极生两仪·先天八卦（竖看）

---

## 🌌 哲学背景

### 先天与后天：竖看与横看

> "先天八卦，竖看者也；后天八卦，横看者也"

| 维度 | 先天八卦（两仪） | 后天八卦（Bagua） |
|------|-----------------|------------------|
| **观察角度** | 竖看（时间维度） | 横看（空间维度） |
| **性质** | 体（本体、本质） | 用（功用、应用） |
| **关注点** | 演化序列、生成次序 | 空间分布、实用应用 |
| **应用** | 状态转换、时间流 | 数据存储、功能模块 |
| **RealConsole** | Liangyyi System | Bagua Memory Palace |

### 当前状态：后天八卦（横看/用）

```
Bagua Memory Palace ✅
├── 乾☰ Intent     (意图) - 空间维度
├── 坤☷ Conversation (对话) - 空间维度
├── 震☳ Action     (行动) - 空间维度
├── 巽☴ Trend      (趋势) - 空间维度
├── 坎☵ Pattern    (模式) - 空间维度
├── 离☲ Knowledge  (知识) - 空间维度
├── 艮☶ Checkpoint (检查点) - 空间维度
└── 兑☱ Feedback   (反馈) - 空间维度

特点：
- 横向分布的八个维度
- 空间化的数据存储
- 实用功能的载体
- "用"的层面
```

### 目标：先天八卦（竖看/体）

```
Liangyyi Evolution System 🚧
太极 (Taiji) - 统一状态
  ↓ 一生二（时间分化）
两仪 (Liangyyi)
  ├── 太阴 ☽ (Taiyin) - 静、收、聚
  └── 太阳 ☉ (Taiyang) - 动、放、散
  ↓ 二生四（状态细分）
四象 (Sixiang)
  ├── 老阴 ▅▅ ▅▅ ▅▅ (Lao Yin) - 极静
  ├── 少阳 ▅▅▅▅▅ ▅▅ ▅▅ (Shao Yang) - 动中有静
  ├── 少阴 ▅▅ ▅▅ ▅▅▅▅▅ (Shao Yin) - 静中有动
  └── 老阳 ▅▅▅▅▅ ▅▅▅▅▅ ▅▅▅▅▅ (Lao Yang) - 极动
  ↓ 四生八（连接 Bagua）
八卦 (Bagua) → Bagua Memory Palace

特点：
- 纵向的时间演化序列
- 状态转换的规律
- 本质层面的描述
- "体"的层面
```

---

## 🎯 设计目标

### 1. 体用合一

**体（Liangyyi）+ 用（Bagua）= 完整系统**

```
         体（Liangyyi）              用（Bagua）
              ↓                          ↓
        时间演化规律               空间数据存储
              ↓                          ↓
        状态转换逻辑               功能模块应用
              ↓                          ↓
        本质层面描述               实用层面操作
              ↓                          ↓
         "竖看"                      "横看"
              ↓                          ↓
            ┌─────────────────────────────┐
            │   RealConsole 完整系统       │
            │  体用合一，阴阳调和          │
            └─────────────────────────────┘
```

### 2. 时间维度建模

**追踪系统状态的演化**：

```
t0: 老阴 (静)
    ↓ 用户开始探索
t1: 少阳 (动中有静)
    ↓ 频繁操作
t2: 老阳 (动)
    ↓ 逐渐稳定
t3: 少阴 (静中有动)
    ↓ 回归静态
t4: 老阴 (静)
```

### 3. 状态感知的决策

**根据状态调整行为**：

```rust
match current_state {
    Sixiang::LaoYin => {
        // 极静：提供学习资源、文档、概念
        provide_learning_materials();
    }
    Sixiang::ShaoYang => {
        // 探索：鼓励尝试、提供示例
        encourage_experimentation();
    }
    Sixiang::LaoYang => {
        // 极动：优化效率、提供快捷方式
        optimize_for_speed();
    }
    Sixiang::ShaoYin => {
        // 蓄势：确认意图、提供反馈
        confirm_and_validate();
    }
}
```

---

## 🏗️ 系统架构

### 模块结构

```
src/liangyyi/
├── mod.rs           - 模块入口
├── taiji.rs         - 太极（统一状态）
├── liangyyi.rs      - 两仪（阴阳分化）
├── sixiang.rs       - 四象（四种状态）
├── tracker.rs       - 状态追踪器
├── predictor.rs     - 转换预测器
└── config.rs        - 配置

集成点：
├── agent.rs         - 主 Agent
├── suggestion/      - 建议引擎（状态感知）
└── bagua/           - 记忆宫殿（数据存储）
```

### 核心数据结构

#### 1. Taiji（太极）

```rust
/// 太极：系统的统一状态
///
/// 阴阳能量的连续表示，0.0-1.0
#[derive(Debug, Clone)]
pub struct Taiji {
    /// 阴能量（静、收、聚、藏）
    pub yin_energy: f64,  // 0.0-1.0

    /// 阳能量（动、放、散、发）
    pub yang_energy: f64, // 0.0-1.0

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 上下文类型
    pub context: TaijiContext,
}

#[derive(Debug, Clone, Copy)]
pub enum TaijiContext {
    /// 用户交互
    UserInteraction,
    /// 系统运行
    SystemRunning,
    /// 学习过程
    LearningProcess,
    /// 决策阶段
    DecisionMaking,
}

impl Taiji {
    /// 创建初始太极（阴阳平衡）
    pub fn new() -> Self {
        Self {
            yin_energy: 0.5,
            yang_energy: 0.5,
            timestamp: Utc::now(),
            context: TaijiContext::SystemRunning,
        }
    }

    /// 更新能量（基于事件）
    pub fn update_from_event(&mut self, event: &Event) {
        match event {
            Event::UserRead => {
                self.yin_energy += 0.05;
                self.yang_energy -= 0.03;
            }
            Event::UserWrite => {
                self.yin_energy -= 0.03;
                self.yang_energy += 0.05;
            }
            Event::UserExecute => {
                self.yin_energy -= 0.05;
                self.yang_energy += 0.08;
            }
            Event::UserThink => {
                self.yin_energy += 0.08;
                self.yang_energy -= 0.05;
            }
        }

        // 归一化到 [0, 1]
        self.normalize();
    }

    /// 归一化能量
    fn normalize(&mut self) {
        self.yin_energy = self.yin_energy.clamp(0.0, 1.0);
        self.yang_energy = self.yang_energy.clamp(0.0, 1.0);
    }

    /// 平衡度（0.0-1.0，1.0 表示完全平衡）
    pub fn balance(&self) -> f64 {
        1.0 - (self.yin_energy - self.yang_energy).abs()
    }
}
```

#### 2. Liangyyi（两仪）

```rust
/// 两仪：阴阳二元状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Liangyyi {
    /// 太阴 ☽ - 极静、深层、内敛
    Taiyin,

    /// 太阳 ☉ - 极动、表层、外放
    Taiyang,
}

impl Liangyyi {
    /// 从太极分化
    pub fn from_taiji(taiji: &Taiji) -> Self {
        if taiji.yin_energy > taiji.yang_energy {
            Liangyyi::Taiyin
        } else {
            Liangyyi::Taiyang
        }
    }

    /// 转换到对立面
    pub fn opposite(&self) -> Self {
        match self {
            Liangyyi::Taiyin => Liangyyi::Taiyang,
            Liangyyi::Taiyang => Liangyyi::Taiyin,
        }
    }
}
```

#### 3. Sixiang（四象）

```rust
/// 四象：老阴、少阳、少阴、老阳
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sixiang {
    /// 老阴 ▅▅ ▅▅ ▅▅ (极静)
    ///
    /// 特征：深度思考、数据沉淀、知识固化
    /// 例子：长时间未操作、深度学习、规划设计
    LaoYin,

    /// 少阳 ▅▅▅▅▅ ▅▅ ▅▅ (动中有静)
    ///
    /// 特征：探索尝试、初次使用、实验性操作
    /// 例子：首次运行命令、探索新功能、查看文档
    ShaoYang,

    /// 少阴 ▅▅ ▅▅ ▅▅▅▅▅ (静中有动)
    ///
    /// 特征：准备阶段、蓄势待发、确认意图
    /// 例子：思考命令、检查状态、分析问题
    ShaoYin,

    /// 老阳 ▅▅▅▅▅ ▅▅▅▅▅ ▅▅▅▅▅ (极动)
    ///
    /// 特征：高频操作、连续执行、快速迭代
    /// 例子：批量处理、紧急修复、自动化脚本
    LaoYang,
}

impl Sixiang {
    /// 从两仪和活动level推导
    pub fn from_liangyyi_and_activity(
        liangyyi: Liangyyi,
        activity_level: f64, // 0.0-1.0
    ) -> Self {
        match liangyyi {
            Liangyyi::Taiyin => {
                if activity_level < 0.3 {
                    Sixiang::LaoYin // 极静
                } else {
                    Sixiang::ShaoYin // 静中有动
                }
            }
            Liangyyi::Taiyang => {
                if activity_level > 0.7 {
                    Sixiang::LaoYang // 极动
                } else {
                    Sixiang::ShaoYang // 动中有静
                }
            }
        }
    }

    /// 自然转换（按周期）
    pub fn transition(&self) -> Self {
        match self {
            Sixiang::LaoYin => Sixiang::ShaoYang,  // 静极生动
            Sixiang::ShaoYang => Sixiang::LaoYang, // 动渐增
            Sixiang::LaoYang => Sixiang::ShaoYin,  // 动极生静
            Sixiang::ShaoYin => Sixiang::LaoYin,   // 静渐增
        }
    }

    /// 描述文本
    pub fn description(&self) -> &'static str {
        match self {
            Sixiang::LaoYin => "极静·深思·沉淀",
            Sixiang::ShaoYang => "探索·尝试·初发",
            Sixiang::ShaoYin => "蓄势·准备·确认",
            Sixiang::LaoYang => "极动·快速·连续",
        }
    }

    /// 符号表示
    pub fn symbol(&self) -> &'static str {
        match self {
            Sixiang::LaoYin => "▅▅ ▅▅ ▅▅",
            Sixiang::ShaoYang => "▅▅▅▅▅ ▅▅ ▅▅",
            Sixiang::ShaoYin => "▅▅ ▅▅ ▅▅▅▅▅",
            Sixiang::LaoYang => "▅▅▅▅▅ ▅▅▅▅▅ ▅▅▅▅▅",
        }
    }
}
```

#### 4. StateTracker（状态追踪器）

```rust
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 状态追踪器
pub struct StateTracker {
    /// 当前太极状态
    current_taiji: Arc<RwLock<Taiji>>,

    /// 当前四象状态
    current_sixiang: Arc<RwLock<Sixiang>>,

    /// 状态历史（最近 N 个）
    state_history: Arc<RwLock<VecDeque<StateSnapshot>>>,

    /// 配置
    config: StateTrackerConfig,
}

#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub taiji: Taiji,
    pub liangyyi: Liangyyi,
    pub sixiang: Sixiang,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct StateTrackerConfig {
    /// 历史记录大小
    pub history_size: usize,

    /// 快照间隔（秒）
    pub snapshot_interval: u64,

    /// 能量衰减率（每秒）
    pub energy_decay_rate: f64,
}

impl Default for StateTrackerConfig {
    fn default() -> Self {
        Self {
            history_size: 100,
            snapshot_interval: 60,
            energy_decay_rate: 0.01,
        }
    }
}

impl StateTracker {
    /// 创建新的追踪器
    pub fn new(config: StateTrackerConfig) -> Self {
        Self {
            current_taiji: Arc::new(RwLock::new(Taiji::new())),
            current_sixiang: Arc::new(RwLock::new(Sixiang::LaoYin)),
            state_history: Arc::new(RwLock::new(VecDeque::with_capacity(config.history_size))),
            config,
        }
    }

    /// 更新状态（基于事件）
    pub async fn update_from_event(&self, event: Event) {
        let mut taiji = self.current_taiji.write().await;
        taiji.update_from_event(&event);

        // 推导两仪
        let liangyyi = Liangyyi::from_taiji(&taiji);

        // 计算活动水平（基于最近历史）
        let activity_level = self.calculate_activity_level().await;

        // 推导四象
        let sixiang = Sixiang::from_liangyyi_and_activity(liangyyi, activity_level);

        // 更新当前四象
        let mut current_sixiang = self.current_sixiang.write().await;
        *current_sixiang = sixiang;

        // 记录快照
        self.record_snapshot(taiji.clone(), liangyyi, sixiang).await;
    }

    /// 获取当前状态
    pub async fn current_state(&self) -> StateSnapshot {
        let taiji = self.current_taiji.read().await.clone();
        let sixiang = *self.current_sixiang.read().await;
        let liangyyi = Liangyyi::from_taiji(&taiji);

        StateSnapshot {
            taiji,
            liangyyi,
            sixiang,
            timestamp: Utc::now(),
        }
    }

    /// 计算活动水平（基于最近历史）
    async fn calculate_activity_level(&self) -> f64 {
        let history = self.state_history.read().await;
        if history.is_empty() {
            return 0.5;
        }

        // 计算最近 10 个快照的阳能量平均值
        let recent_yang: f64 = history
            .iter()
            .rev()
            .take(10)
            .map(|s| s.taiji.yang_energy)
            .sum();

        (recent_yang / 10.0).clamp(0.0, 1.0)
    }

    /// 记录快照
    async fn record_snapshot(&self, taiji: Taiji, liangyyi: Liangyyi, sixiang: Sixiang) {
        let mut history = self.state_history.write().await;

        let snapshot = StateSnapshot {
            taiji,
            liangyyi,
            sixiang,
            timestamp: Utc::now(),
        };

        history.push_back(snapshot);

        // 限制大小
        if history.len() > self.config.history_size {
            history.pop_front();
        }
    }

    /// 应用能量衰减
    pub async fn apply_decay(&self) {
        let mut taiji = self.current_taiji.write().await;
        let decay = self.config.energy_decay_rate;

        // 向平衡态衰减
        if taiji.yin_energy > 0.5 {
            taiji.yin_energy -= decay;
        } else {
            taiji.yin_energy += decay;
        }

        if taiji.yang_energy > 0.5 {
            taiji.yang_energy -= decay;
        } else {
            taiji.yang_energy += decay;
        }

        taiji.normalize();
    }
}
```

#### 5. Event（事件定义）

```rust
/// 系统事件
#[derive(Debug, Clone)]
pub enum Event {
    /// 用户读取（查看文档、帮助等）
    UserRead,

    /// 用户写入（编辑文件、创建内容等）
    UserWrite,

    /// 用户执行（运行命令等）
    UserExecute,

    /// 用户思考（长时间无操作，但在线）
    UserThink,

    /// 系统空闲
    SystemIdle,
}
```

---

## 🔄 数据流设计

### 完整流程

```
1. 用户操作
   ↓
2. Agent 解析为 Event
   ↓
3. StateTracker.update_from_event(event)
   ↓
4. 更新 Taiji（阴阳能量）
   ↓
5. 推导 Liangyyi（太阴/太阳）
   ↓
6. 计算 activity_level
   ↓
7. 推导 Sixiang（老阴/少阳/少阴/老阳）
   ↓
8. 记录 StateSnapshot
   ↓
9. 写入 Bagua Memory Palace
   ├── StateSnapshot → 艮☶ Checkpoint
   └── StateTransition → 巽☴ Trend
   ↓
10. 影响 SuggestionEngine（状态感知建议）
   ↓
11. 返回给用户
```

### 与 Bagua 的交互

```
Liangyyi System          Bagua Memory Palace
      ↓                         ↓
StateSnapshot    →    艮☶ Checkpoint (状态快照)
      ↓                         ↓
StateTrend       →    巽☴ Trend (趋势分析)
      ↓                         ↑
StateKnowledge   →    离☲ Knowledge (状态知识)
      ↓                         ↑
TransitionPattern ←   坎☵ Pattern (转换模式)
```

---

## 📊 应用场景

### 1. 状态感知的建议

```rust
async fn get_state_aware_suggestions(&self, input: &str) -> Vec<Suggestion> {
    let state = self.state_tracker.current_state().await;

    match state.sixiang {
        Sixiang::LaoYin => {
            // 极静：推荐学习资源、文档、概念
            vec![
                Suggestion::new("阅读 README", 0.9),
                Suggestion::new("查看文档 --help", 0.85),
                Suggestion::new("理解概念", 0.8),
            ]
        }
        Sixiang::ShaoYang => {
            // 探索：推荐实验性命令、示例
            vec![
                Suggestion::new("试试 cargo check", 0.9),
                Suggestion::new("运行示例", 0.85),
                Suggestion::new("探索新功能", 0.8),
            ]
        }
        Sixiang::LaoYang => {
            // 极动：推荐快捷命令、自动化
            vec![
                Suggestion::new("使用别名 cb", 0.9),
                Suggestion::new("批量操作", 0.85),
                Suggestion::new("自动化脚本", 0.8),
            ]
        }
        Sixiang::ShaoYin => {
            // 蓄势：推荐检查、确认
            vec![
                Suggestion::new("检查 git status", 0.9),
                Suggestion::new("验证环境", 0.85),
                Suggestion::new("确认参数", 0.8),
            ]
        }
    }
}
```

### 2. 学习阶段识别

```rust
pub fn identify_learning_stage(&self, state: &StateSnapshot) -> LearningStage {
    match state.sixiang {
        Sixiang::LaoYin => LearningStage::Beginner {
            phase: "深度学习",
            recommendation: "提供基础教程和概念讲解",
        },
        Sixiang::ShaoYang => LearningStage::Practicing {
            phase: "动手尝试",
            recommendation: "提供示例和实践指导",
        },
        Sixiang::LaoYang => LearningStage::Proficient {
            phase: "熟练运用",
            recommendation: "提供高级技巧和优化方法",
        },
        Sixiang::ShaoYin => LearningStage::Reflecting {
            phase: "反思总结",
            recommendation: "提供最佳实践和模式总结",
        },
    }
}
```

---

## 🚀 实施计划

### Phase 1: 核心结构（1 天）

**任务**：
1. 创建 `src/liangyyi/` 模块
2. 实现 Taiji、Liangyyi、Sixiang 结构
3. 实现 Event 定义
4. 单元测试

**交付**：
- taiji.rs (80 行)
- liangyyi.rs (40 行)
- sixiang.rs (120 行)
- mod.rs (20 行)
- 测试覆盖 >90%

### Phase 2: 状态追踪（0.5 天）

**任务**：
1. 实现 StateTracker
2. 集成到 Agent
3. 事件更新逻辑
4. Bagua 集成

**交付**：
- tracker.rs (200 行)
- Agent 集成 (30 行)

### Phase 3: 应用集成（0.5 天）

**任务**：
1. SuggestionEngine 状态感知
2. 学习阶段识别
3. 完整测试
4. 文档完善

**交付**：
- engine.rs 修改 (50 行)
- 完成报告

---

## 📝 配置示例

```yaml
liangyyi:
  enabled: true

  # 状态追踪配置
  state_tracker:
    history_size: 100
    snapshot_interval: 60  # 秒
    energy_decay_rate: 0.01  # 每秒

  # 事件→能量映射
  event_energy_map:
    user_read: { yin: 0.05, yang: -0.03 }
    user_write: { yin: -0.03, yang: 0.05 }
    user_execute: { yin: -0.05, yang: 0.08 }
    user_think: { yin: 0.08, yang: -0.05 }

  # 四象阈值
  sixiang_thresholds:
    lao_yin_activity: 0.3   # < 0.3 → 老阴
    lao_yang_activity: 0.7  # > 0.7 → 老阳
```

---

**制定者**: RealConsole Team
**日期**: 2025-10-28
**版本**: v1.9.0-alpha
**状态**: 🚧 设计完成，待实施

---

> "太极生两仪，两仪生四象，四象生八卦"
> "竖看时间之演化，横看空间之分布"
> "体用合一，阴阳调和，道法自然"
>
> Liangyyi State Evolution System，体之设计！☯️🌌✨
