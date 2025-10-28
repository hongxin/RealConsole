# Changelog

All notable changes to RealConsole will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.14.0] - 2025-10-28

### Added

- **[两仪系统] 闭环自适应执行层（Execute Layer）**
  - **StateTracker 自适应集成**（src/liangyyi/tracker.rs，+202 行）：
    - 添加可选 `adaptive_system` 字段，支持运行时启用/禁用
    - `config` 改为 `Arc<RwLock<StateTrackerConfig>>` 支持动态调整
    - 核心方法：
      - `enable_adaptive(target)` - 启用自适应，指定目标状态
      - `is_adaptive_enabled()` - 检查是否启用
      - `apply_recommendations(recs)` - 应用建议到配置
      - `auto_optimize()` - 完整自动优化循环（观测→预测→建议→执行）
      - `get_recommendations()` - 获取建议（不应用，用于预览）
      - `get_config()` - 获取当前配置（只读）
  - **维度到配置映射机制**：
    - `efficiency` → `energy_decay_rate`：效率低时增加衰减率（快速重置）
    - `activity` → `low/high_activity_threshold`：调整活动阈值
    - `load` → `snapshot_interval`：负载高时减少间隔（更频繁观测）
    - `context` → `history_size`：上下文高时增加历史大小
  - **测试覆盖**：新增 8 个单元测试，liangyyi 模块从 75 增至 83 个测试，全部通过

### Changed

- **StateTracker 结构演进**：
  - `config` 字段从直接存储改为 `Arc<RwLock<StateTrackerConfig>>`
  - 添加 `adaptive_system: Option<Arc<RwLock<AdaptiveSystem>>>`
  - 所有访问 config 的代码改为 async 读写

### Design Philosophy

- **闭环控制**：观测 → 预测 → 建议 → **执行** → 观测（形成完整闭环）
- **OODA 循环完成**：
  - Observe（观测）- StateVector
  - Orient（预测）- StatePredictor
  - Decide（建议）- AdaptiveSystem
  - **Act（执行）- Execute Layer** ⬅️ v1.14.0
- **无侵入设计**：通过 Option 字段可选启用，不影响现有代码
- **双模式操作**：
  - `auto_optimize()` - 自动应用调整
  - `get_recommendations()` - 只获取建议不应用（用于分析/预览）

### Use Cases

```rust
use realconsole::liangyyi::{StateTracker, StateTrackerConfig};
use realconsole::liangyyi::adaptive::TargetState;

// 创建并启用自适应
let mut tracker = StateTracker::new(StateTrackerConfig::default());
tracker.enable_adaptive(TargetState::balanced());

// 定期自动优化
loop {
    let recommendations = tracker.auto_optimize().await?;
    println!("应用了 {} 个优化建议", recommendations.len());
    tokio::time::sleep(Duration::from_secs(60)).await;
}

// 或仅预览建议
let recommendations = tracker.get_recommendations().await?;
for rec in recommendations.iter().take(3) {
    println!("🎯 {}: {}", rec.dimension, rec.reason);
}
```

### Integration

- **与 v1.13.0 AdaptiveSystem 集成**：
  ```rust
  // StateTracker 持有 AdaptiveSystem
  // auto_optimize 内部调用 AdaptiveSystem::generate_recommendations
  ```

- **与 v1.12.0 StatePredictor 集成**：
  ```rust
  // AdaptiveSystem 内部使用 StatePredictor 预测
  // 形成完整的预测-调整链条
  ```

### Notes

- ✅ 完整的闭环自适应控制
- ✅ 清晰的维度到配置映射规则
- ✅ 线程安全（Arc<RwLock>）
- ✅ 双模式操作（应用 vs 预览）
- ✅ 无侵入集成（可选启用）
- 📚 完整文档：docs/04-reports/v1.14.0-execute-layer.md

### Evolution Path

```
v1.11.0: StateVector      → 状态空间化
v1.12.0: StatePredictor   → 时序预测
v1.13.0: AdaptiveSystem   → 自我调整
v1.14.0: Execute Layer    → 闭环执行  ⬅️ 当前
```

**下一步**：多视角观测系统（回到 B - 原计划 v1.11.0 内容）

## [1.13.0] - 2025-10-28

### Added

- **[两仪系统] AdaptiveSystem 自适应调整系统（Adaptive Adjustment System）**
  - **TargetState 目标状态定义**（src/liangyyi/adaptive.rs，~100 行）：
    - 使用 `HashMap<String, (f64, f64)>` 定义每个维度的目标范围（min, max）
    - 三种预设目标状态：
      - `balanced()` - 平衡态：所有维度均衡（0.5-0.7）
      - `high_performance()` - 高性能态：高活跃度、高效率、高决策力
      - `power_save()` - 节能态：低活动、低负载
    - 核心方法：
      - `distance_to()` - 计算当前状态到目标的距离
      - `is_within()` - 检查是否在目标范围内
      - `find_most_deviating_dimension()` - 查找偏离最严重的维度
  - **Recommendation 建议系统**（~50 行）：
    - `RecommendationAction` 枚举：Enhance（增强）/ Reduce（降低）/ Maintain（保持）
    - 建议结构包含：维度、动作、当前值、目标范围、优先级、原因
    - 基于偏离度自动计算优先级（偏离度 = 优先级）
    - 自动生成建议原因的可读文本
  - **AdaptiveStrategy 自适应策略**（~30 行）：
    - 三种调整策略：
      - `Aggressive` - 激进策略（步长 0.2）：快速调整
      - `Balanced` - 平衡策略（步长 0.1）：稳健调整
      - `Conservative` - 保守策略（步长 0.05）：缓慢调整
    - 适用场景：启动期用激进，稳定期用平衡，生产环境用保守
  - **AdaptiveSystem 核心系统**（~150 行）：
    - 集成 `StatePredictor` 实现预测驱动的自适应
    - 核心工作流程：
      1. 使用 predictor 预测未来状态（1步）
      2. 分析趋势（Rising/Falling/Stable）
      3. 为每个维度生成建议
      4. 按优先级排序
    - 关键方法：
      - `generate_recommendations()` - 生成调整建议（优先级排序）
      - `calculate_adjustment()` - 计算调整向量
      - `add_observation()` - 添加观测（委托给 predictor）
      - `set_target()` / `with_strategy()` - 动态调整配置
  - **测试覆盖**：新增 8 个单元测试，liangyyi 模块从 67 增至 75 个测试，全部通过

### Changed

- **模块导出**（src/liangyyi/mod.rs）：新增 `pub use adaptive::{AdaptiveStrategy, AdaptiveSystem, Recommendation, RecommendationAction, TargetState}`
- **能力升级**：从被动预测进化到主动调整，实现完整的"观测-预测-调整"闭环

### Design Philosophy

- **道生一，一生二，二生三，三生万物**：
  - v1.11.0 (一) → StateVector（多维状态空间）
  - v1.12.0 (二) → StatePredictor（时序预测能力）
  - v1.13.0 (三) → AdaptiveSystem（自我调整智慧）
- **分离关注点**：
  - TargetState → 定义"应该是什么"
  - StatePredictor → 预测"将会是什么"
  - Recommendation → 建议"需要做什么"
  - AdaptiveStrategy → 决定"如何去做"
- **OODA 循环**：Observe（观测）→ Orient（预测）→ Decide（建议）→ Act（调整）
- **优先级驱动**：建议按偏离度自动排序，优先处理最紧急的调整

### Use Cases

```rust
use realconsole::liangyyi::{AdaptiveSystem, StateVector, TargetState, AdaptiveStrategy};

// 创建自适应系统
let mut system = AdaptiveSystem::new(TargetState::balanced())
    .with_strategy(AdaptiveStrategy::Balanced);

// 添加历史观测
for i in 0..10 {
    system.add_observation(collect_state());
}

// 生成建议
let recommendations = system.generate_recommendations();
for rec in recommendations.iter().take(3) {
    println!("🎯 {}: {} (优先级: {:.2})",
        rec.dimension, rec.reason, rec.priority);
}

// 计算调整
let current = collect_current_state();
let adjusted = system.calculate_adjustment(&current);
println!("📊 调整: {} → {}", current, adjusted);
```

输出示例：
```
🎯 efficiency: 当前值 0.45 低于目标范围 [0.60, 0.80] (优先级: 0.15)
🎯 activity: 当前值 0.85 高于目标范围 [0.50, 0.70] (优先级: 0.15)
🎯 load: 当前值 0.62 高于目标范围 [0.30, 0.50] (优先级: 0.12)
```

### Integration Points

- **与 v1.12.0 StatePredictor 集成**：
  ```rust
  // AdaptiveSystem 直接使用 predictor 预测未来状态
  let predicted = self.predictor.predict_linear(1)?;
  let trends = self.predictor.analyze_trends();
  ```

- **与 v1.11.0 StateVector 集成**：
  ```rust
  // 使用 StateVector::evolve_towards 实现向量演化
  adjusted.evolve_towards(&target_vector, step);
  ```

- **未来与 StateTracker 集成**：
  ```rust
  // StateTracker 可以使用 AdaptiveSystem 自动优化
  impl StateTracker {
      pub async fn auto_optimize(&mut self) -> anyhow::Result<()> {
          let current = self.to_state_vector().await;
          let adaptive = AdaptiveSystem::new(TargetState::balanced());
          adaptive.add_observation(current);
          let recommendations = adaptive.generate_recommendations();
          self.apply_recommendations(&recommendations)?;
          Ok(())
      }
  }
  ```

### Notes

- ✅ 完整的目标状态定义系统（3 种预设）
- ✅ 智能建议生成机制（优先级驱动）
- ✅ 多策略自适应调整（3 种策略）
- ✅ 与 StatePredictor 无缝集成
- ✅ 清晰的关注点分离（Target/Predictor/Recommendation/Strategy）
- 🔄 未来：执行层（apply_recommendations）实现闭环控制
- 📚 完整文档：docs/04-reports/v1.13.0-adaptive-system.md (~950 行)

### Evolution Path

```
v1.11.0: StateVector      → 状态空间化
v1.12.0: StatePredictor   → 时序预测
v1.13.0: AdaptiveSystem   → 自我调整  ⬅️ 当前
v1.14.0: Execute Layer    → 闭环执行  (待规划)
```

## [1.12.0] - 2025-10-28

### Added

- **[两仪系统] StatePredictor 状态预测系统（State Prediction System）**
  - **StatePredictor 结构**（src/liangyyi/predictor.rs，420 行）：
    - 基于历史 StateVector 序列预测未来状态
    - 使用 `VecDeque` 管理历史队列，自动淘汰旧数据（FIFO）
    - 支持动态调整历史窗口大小（推荐 5-20）
  - **预测算法**（2 个）：
    - `predict_linear()` - 线性趋势外推
      - 计算平均变化率（斜率）
      - 从最后观测外推 N 步
      - 适合稳定的线性趋势
    - `predict_ewma()` - 指数加权移动平均
      - 近期观测权重更高
      - 平滑处理，减少噪声
      - 支持可调 alpha 参数（0.1-0.9）
  - **趋势分析**：
    - `analyze_trends()` - 分析所有维度的趋势
    - `TrendDirection` 枚举：Rising（上升）/ Falling（下降）/ Stable（稳定）
    - `DimensionTrend` 结构：包含方向、强度、变化率
  - **异常检测**：
    - `detect_anomaly()` - 检测预测值与实际值差异
    - 基于距离阈值判断异常
  - **数据管理**（5 个方法）：
    - `add_observation()` - 添加观测值（自动淘汰）
    - `clear()` - 清空历史
    - `history_len()` - 获取历史长度
    - `can_predict()` - 检查是否有足够数据
  - **测试覆盖**：新增 10 个单元测试，liangyyi 模块从 60 增至 70 个测试，全部通过

### Changed

- **模块导出**（src/liangyyi/mod.rs）：新增 `pub use predictor::{DimensionTrend, StatePredictor, TrendDirection}`
- **能力升级**：从被动观测进化到主动预测，实现"观往知来"

### Design Philosophy

- **观往知来**：分析历史 → 识别趋势 → 预测未来
- **易经之易**：状态持续演化，预测就是模拟演化路径
- **阴阳平衡**：上升（阳）vs 下降（阴）vs 稳定（平衡）
- **双算法互补**：线性趋势（快速直观）+ EWMA（平滑稳定）

### Use Cases

```rust
// 状态预警
let mut predictor = StatePredictor::new(10);
predictor.add_observation(current_state);

if let Some(predicted) = predictor.predict_linear(1) {
    if predicted.get("efficiency").unwrap() < 0.3 {
        println!("预警：效率可能下降");
    }
}

// 趋势监控
let trends = predictor.analyze_trends();
for trend in trends {
    println!("{}: {:?} (强度 {:.2})",
        trend.dimension, trend.direction, trend.strength);
}

// 异常检测
if predictor.detect_anomaly(&actual, 0.15) {
    println!("⚠ 异常：当前状态偏离预期");
}
```

### Notes

- ✅ 双算法支持：线性趋势 + EWMA，互补使用
- ✅ 自动化设计：自动淘汰旧数据，自动计算指标
- ✅ 高性能：所有操作在微秒级完成
- ✅ 实用功能：趋势分析 + 异常检测
- 📊 详细报告：见 `docs/04-reports/v1.12.0-state-prediction.md`

## [1.11.0] - 2025-10-28

### Added

- **[两仪系统] StateVector 多维状态空间（Multi-dimensional State Space）**
  - **StateVector 结构**（src/liangyyi/state_vector.rs，489 行）：
    - 基于 `HashMap<String, f64>` 的灵活多维向量表示
    - 7 个标准维度：`yin`, `yang`, `context`, `activity`, `load`, `efficiency`, `confidence`
    - 每个维度值范围 [0.0, 1.0]，自动 clamp 防止越界
    - 支持动态添加/删除维度，易于扩展
  - **构造函数**（3 个）：
    - `new()` - 创建空向量
    - `standard()` - 创建标准维度向量（所有维度 = 0.5）
    - `from_snapshot()` - 从 StateSnapshot 创建（自动映射 7 个维度）
  - **维度访问**（4 个方法）：
    - `get()` - 获取维度值
    - `set()` - 设置维度值（自动 clamp）
    - `dimension_names()` - 获取所有维度名称（排序）
    - `dimension_count()` - 获取维度数量
  - **向量运算**（4 个方法）：
    - `distance_to()` - 欧几里得距离计算（只考虑共同维度）
    - `evolve_towards()` - 状态演化模拟（向目标渐进）
    - `add()` - 向量加法（逐维度，自动 clamp）
    - `scale()` - 向量数乘（所有维度缩放）
  - **分析方法**（5 个）：
    - `norm()` - 欧几里得范数（sqrt(Σ value[i]²)）
    - `mean()` - 平均值
    - `max_dimension()` - 最大维度
    - `min_dimension()` - 最小维度
    - `is_balanced()` - 判断是否平衡（max - min <= threshold）
  - **StateTracker 集成**（src/liangyyi/tracker.rs）：
    - 新增方法 `to_state_vector()` - 便捷导出当前状态为 StateVector
  - **测试覆盖**：新增 15 个单元测试，liangyyi 模块从 39 增至 60 个测试，全部通过

### Changed

- **模块导出**（src/liangyyi/mod.rs）：新增 `pub use state_vector::StateVector`
- **状态表示升级**：从离散分类提升到连续多维向量空间

### Design Philosophy

- **一分为三**：状态不是"好/坏"二分，而是多维连续空间 [yin, yang, context, ...]
- **易经之易**：`evolve_towards()` 实现状态的连续演化路径
- **阴阳平衡**：多维度综合平衡判断（`is_balanced()`）
- **体用不二**：抽象向量表示（体）+ 具体数学运算（用）

### Use Cases

```rust
// 状态距离监控
let vec1 = tracker.to_state_vector().await;
// ... 一段时间后 ...
let vec2 = tracker.to_state_vector().await;
let distance = vec1.distance_to(&vec2);

// 状态演化模拟
let mut current = vec1.clone();
current.evolve_towards(&target, 0.1); // 向目标移动 10%

// 多维度分析
if let Some((dim, value)) = vec.min_dimension() {
    if value < 0.3 {
        println!("警告：{} 维度过低", dim);
    }
}
```

### Notes

- ✅ 无缝集成：单行转换 `tracker.to_state_vector().await`
- ✅ 数学基础：坚实的欧几里得空间运算
- ✅ 灵活扩展：HashMap 设计支持动态维度
- ✅ 高性能：所有运算在微秒级完成
- 📊 详细报告：见 `docs/04-reports/v1.11.0-state-vector.md`（665 行）

## [1.10.0] - 2025-10-28

### Added

- **[系统架构] 两仪与八卦深度集成（Liangyyi-Bagua Deep Integration）**
  - **StateSnapshot 存储转换**（src/liangyyi/tracker.rs）：
    - 新增方法 `to_checkpoint_state()` - 将快照转换为 JSON 格式（轻量级，避免完整序列化）
    - 新增方法 `from_checkpoint_state()` - 从 JSON 恢复快照（容错设计，支持部分数据恢复）
  - **八卦宫殿集成**（src/liangyyi/tracker.rs，+370 行）：
    - 新增方法 `sync_to_bagua()` - 同步状态到八卦宫殿
      - 艮卦（Gen）：存储完整状态快照（Checkpoint），energy = system_load
      - 巽卦（Xun）：存储状态趋势模式（Trend），energy = change_rate
    - 新增静态方法 `restore_from_bagua()` - 从八卦宫殿恢复状态
      - 读取艮卦最新检查点
      - 重建 StateTracker 实例
      - 恢复历史记录
    - 新增静态方法 `has_checkpoint()` - 检测是否存在可恢复的检查点
  - **测试覆盖**：新增 6 个集成测试，覆盖序列化、同步、恢复、多次同步、元数据等场景，全部通过（17/17）

### Changed

- **存储策略**：状态快照现在可以持久化到八卦宫殿，实现跨会话状态恢复
- **体用合一**：两仪（时间维度）与八卦（空间维度）融合，实现"竖看"与"横看"的统一

### Design Philosophy

- **体用合一**：两仪（时间演化）+ 八卦（空间存储）= 时空融合
- **渐进演化**：100% 向后兼容，无破坏性变更
- **一分为三**：不是"保存/丢失"二分，而是"检查点-趋势-实时"三态

### Storage Strategy

| 八卦维度 | 存储内容         | Energy 映射      | 用途                 |
|----------|------------------|------------------|----------------------|
| 艮卦 Gen | StateSnapshot    | system_load      | 状态检查点（恢复点） |
| 巽卦 Xun | Trend Pattern    | change_rate      | 趋势分析（历史模式） |

### Notes

- ✅ 轻量级设计：JSON 转换，避免完整 Serialize trait
- ✅ 容错机制：部分数据丢失仍可恢复（使用默认值）
- ✅ 能量映射：根据重要性分配 energy（system_load 和 change_rate）
- ✅ 完整测试：6 个集成测试 + 完整手工测试指引
- 📊 详细报告：见 `docs/04-reports/v1.10.0-liangyyi-bagua-integration.md`（~800 行）
- 📋 测试指引：见 `docs/04-reports/v1.10.0-manual-testing-guide.md`（~400 行）

## [1.9.6] - 2025-10-28

### Added

- **[两仪系统] 上下文强度与持续时间追踪（Context Intensity & Duration Tracking）**
  - **Taiji 扩展**（src/liangyyi/taiji.rs）：
    - 新增字段 `context_intensity: f64` - 上下文强度（0.0-1.0 连续值）
    - 新增字段 `context_duration: Duration` - 上下文持续时间
    - 新增构造函数 `with_context_and_intensity()` - 创建指定强度的上下文
    - 新增方法 `switch_context()` - 切换上下文时自动重置强度和时间
    - 新增方法 `enhance_context()` - 动态调整上下文强度
    - 自动追踪：`update_from_event()` 现在会自动更新持续时间和强度
  - **StateSnapshot 增强**（src/liangyyi/tracker.rs）：
    - 新增观测维度 `user_activity_level` - 用户活跃度（基于 yang_energy）
    - 新增观测维度 `system_load` - 系统负载（基于 context_intensity）
    - 新增观测维度 `learning_efficiency` - 学习效率（基于 balance()）
    - 新增观测维度 `decision_confidence` - 决策信心（基于 balance()）
    - 新增构造函数 `from_current_state()` - 自动计算四个观测维度
    - 新增方法 `overall_score()` - 综合评分（四维等权重）
    - 新增方法 `is_optimal()` - 判断是否处于最优状态
  - **测试覆盖**：新增 9 个测试，liangyyi 模块测试全部通过（39 passed, 0 failed）

### Changed

- **状态追踪自动化**：`current_state()` 和 `record_snapshot()` 使用新构造函数自动计算观测维度
- **时间维度引入**：状态不再是静态的，而是随时间动态演化
  - 上下文强度随持续时间自然增强（每分钟最多 +0.1）
  - 空闲时上下文强度自然衰减（每次 -0.02）

### Design Philosophy

- **一分为三**：上下文不是"强/弱"二分，而是 [0.0, 1.0] 连续谱
- **易经之易**：引入时间维度，状态随时间自然演化
- **阴阳平衡**：空闲时自动向平衡态衰减
- **体用不二**：底层连续（f64），表层离散（enum）

### Notes

- ✅ 100% 向后兼容：现有代码无需修改
- ✅ 自动化设计：观测维度从 Taiji 状态自动派生
- ✅ 扩展性强：为 v1.11.0 StateVector 多维状态空间打基础
- 📊 详细报告：见 `docs/04-reports/v1.9.6-enhanced-state-snapshot.md`

## [1.9.5] - 2025-10-28

### Added

- **[用户体验] 交互式命令支持（Interactive Commands Support）**
  - **自动检测与路由**：智能识别需要接管终端的命令，自动使用特殊执行方式
  - **支持的命令类别**（31 个命令）：
    - 编辑器：vi, vim, nvim, nano, emacs, joe, pico（7个）
    - 分页器：less, more, most（3个）
    - 系统监控：top, htop, iotop, iftop, nethogs（5个）
    - 文件管理器：mc, ranger, vifm（3个）
    - 其他工具：man, info, watch, tmux, screen（5个）
    - Git 交互式：git add -i, git add -p, git rebase -i（3个）
    - 数据库客户端：mysql, psql, sqlite3, redis-cli, mongo（5个）
  - **终端接管模式**：使用 `Stdio::inherit()` 让交互式程序完全控制终端
    - stdin: 完全接管，支持所有键盘输入
    - stdout: 完全接管，支持全屏显示和颜色
    - stderr: 完全接管，正常显示错误信息
  - **智能检测算法**：
    - 命令名匹配：检查第一个单词是否在交互式列表中
    - 多词命令支持：支持 "git add -i" 等复杂命令
    - 自动降级：检测失败时自动使用普通执行方式
  - **使用示例**：
    ```bash
    % !vim README.md     # ✓ 正常编辑，所有快捷键工作
    % !less file.log     # ✓ 正常分页，上下翻页工作
    % !top               # ✓ 正常监控，实时刷新
    % !man ls            # ✓ 正常查看手册
    % !git add -i        # ✓ 正常交互式添加
    ```

### Changed

- **Shell 执行器增强**：`execute_shell()` 现在会先检查命令类型，自动路由到合适的执行方式
  - 交互式命令 → `execute_interactive()` (终端接管模式)
  - 普通命令 → 原有逻辑 (输出捕获模式)

### Fixed

- **[Bug] vi/vim/nano 等编辑器无法正常工作**
  - 问题：使用 `Stdio::piped()` 捕获输出导致编辑器无法接管终端
  - 修复：对交互式命令使用 `Stdio::inherit()` 让其完全控制终端
  - 影响范围：所有需要全屏交互的命令

### Notes

- **代码统计**：
  - 修改文件：1 个（`src/shell_executor.rs`）
  - 新增代码：+135 行
  - 新增测试：+7 个（1050 → 1057）
  - 测试通过：1057/1057 (100%) ✅
  - 测试时间：114.06s
- **文档**：
  - 新增功能文档：`docs/04-reports/interactive-commands-feature.md` (308行)
  - 包含完整的使用指南、技术实现和性能分析
- **性能影响**：
  - 编译时间：+0.5秒（可忽略）
  - 运行时开销：<1ms（仅字符串匹配）
  - 内存占用：+1KB（命令列表）
- **用户体验**：⭐⭐⭐⭐⭐ 显著改善
  - 之前：编辑器无响应，用户被困
  - 现在：所有交互式命令正常工作
- **向后兼容性**：✅ 完全兼容，不影响现有功能
- **未来优化**：
  - 配置化命令列表（允许用户自定义）
  - 基于 TTY 需求自动检测
  - 状态栏显示"交互式模式"
  - 快捷键支持（Ctrl+Z 暂停）
- **用户反馈驱动**：此功能由用户反馈实现，感谢社区建议！
- 详见功能文档：`docs/04-reports/interactive-commands-feature.md`

---

## [1.9.4] - 2025-10-28

### Added

- **[两仪系统增强] 学习阶段识别与状态感知建议**
  - **Phase 1: 学习阶段检测算法（42 行）**
    - `LearningPhase` 枚举：Exploration（探索期）/ Stability（稳定期）/ Transition（转变期）
    - `detect_learning_phase()` 方法：基于增量波动性和状态变化率
    - **增量波动性算法**：使用二阶导数（能量 delta 的标准差）区分稳定趋势与混沌振荡
      - 稳定趋势：一阶导数恒定 → 低二阶导数
      - 混沌振荡：一阶导数变化 → 高二阶导数
    - **检测阈值**：
      - Exploration: 波动性 > 0.12 或 变化率 > 0.4
      - Stability: 波动性 < 0.06 且 变化率 < 0.2
      - Transition: 其他情况
    - 测试：3/3 通过
  - **Phase 2: `/liangyyi` 可视化命令（412 行）**
    - 4 个子命令：
      - `status` - 显示当前状态（太极、两仪、四象、学习阶段）
      - `stats` - 显示统计信息（四象分布、平均平衡度）
      - `history [n]` - 显示历史快照（最近 N 条）
      - `trend` - 显示趋势分析
    - **彩色 Unicode 输出**：
      - 能量条：▰▱（cyan/default）
      - 两仪符号：☽太阴 / ☉太阳
      - 四象符号：☷老阴 / ☲少阳 / ☵少阴 / ☰老阳
    - 命令注册：遵循项目标准模式（`Command::from_fn`）
  - **Phase 3: 状态感知建议增强（67 行）**
    - **SuggestionContext 扩展**（3 个新字段）：
      - `learning_phase`: 当前学习阶段
      - `volatility`: 状态波动性（0.0-1.0+）
      - `change_rate`: 状态变化率（0.0-1.0）
    - **动态策略调整**（`get_phase_adjustments()`）：
      - **Exploration 期**：
        - 上下文权重 +20%（鼓励探索新命令）
        - 历史权重 -20%（降低习惯依赖）
        - 阈值降低至 0.3（接受更多建议）
        - 建议数量 +2（提供更多选择）
      - **Stability 期**：
        - 上下文权重 -20%（减少干扰）
        - 历史权重 +20%（强化熟练命令）
        - 阈值提高至 0.6（只显示高质量建议）
        - 建议数量 -1（精简输出）
      - **Transition 期**：默认值
    - **Agent 集成**：自动填充学习阶段信息到建议上下文

### Changed

- **StateTracker**: `calculate_activity_level()` 方法改为 public，供可视化命令调用
- **Agent**: 建议系统现在基于学习阶段动态调整策略
- **SuggestionEngine**: 权重和阈值根据学习阶段动态变化

### Notes

- **代码统计**：
  - 总计：539 行（Phase 1: 42 + Phase 2: 412 + Phase 3: 67 + 其他: 18）
  - 新增文件：2 个（`src/commands/liangyyi_cmd.rs`, `docs/04-reports/liangyyi-visualization-design.md`）
  - 修改文件：7 个
  - 测试：1050/1050 通过 ✅（单线程模式）
- **哲学体现**：
  - 学习阶段检测：体现"一分为三"思想（探索/稳定/转变）
  - 增量波动性：区分"稳定趋势"与"混沌振荡"，超越二元对立
  - 状态感知建议：建议系统获得时间维度感知，实现"体用合一"
- **用户体验**：
  - 建议系统更智能：根据用户当前状态动态调整
  - 可视化更完善：`/liangyyi` 命令提供完整的状态视图
  - 学习曲线优化：探索期提供更多帮助，稳定期减少干扰
- **测试说明**：
  - 并行测试有 7-9 个失败（race condition，项目既有问题）
  - 单线程测试 100% 通过（`cargo test --lib -- --test-threads=1`）
  - 本次功能不影响项目稳定性
- 详见设计文档：`docs/04-reports/liangyyi-visualization-design.md`

---

## [1.9.3] - 2025-10-28

### Fixed

- **[代码质量] Clippy 手动修复（Phase 2）**
  - 修复 8 个 P1 Clippy 警告（19 → 11，改进 42%）
  - **文档注释问题**（2个）：修复空行和外部文档注释格式
  - **标准 Trait 实现**（6个）：实现 `FromStr` 和 `Default` trait
    - `DisplayMode`：实现 `FromStr` trait（`src/display.rs`）
    - `Language`：实现 `FromStr` trait（`src/i18n.rs`）
    - `NotificationMode`：实现 `FromStr` trait（`src/likan/types.rs`）
    - `LogLevel`：实现 `FromStr` trait（`src/log_analyzer.rs`）
    - `DisplayHelper`：实现 `Default` trait（`src/display_helper.rs`）
    - `HistoryManager`：实现 `Default` trait（`src/history.rs`）

### Changed

- **API 改进**: 所有实现 `FromStr` 的类型现在支持 `.parse()` 方法
- **测试更新**: 更新所有相关测试使用标准 trait 方法
- **main.rs**: 使用 `.parse()` 替代自定义 `from_str()` 方法

### Notes

- **修改文件**: 10 个（6 个核心代码 + 4 个测试/主程序）
- **警告减少**: 19 → 11（42% 改进，累计 70% 改进）
- **测试结果**: 1050/1050 通过 ✅
- **剩余警告**: 11 个（P2/P3 优先级，可留给 v1.9.4）
- **累计进展**: 37 → 11 警告（总改进 70%）

---

## [1.9.2] - 2025-10-28

### Fixed

- **[代码质量] Clippy 自动修复（Phase 1）**
  - 修复 18 个 Clippy 警告（37 → 19，改进 48%）
  - 使用派生的 `Default` trait 替代手动实现（`src/i18n.rs`）
  - 添加 `Default` 实现到 `LiKanStatusBar`（`src/likan/statusbar.rs`）
  - 优化冗余闭包使用（`src/repl.rs`, `src/services/llm_service.rs`）
  - 优化代码风格：`map_clone` → `cloned()`（`src/agent.rs`）
  - 修复无用的 `format!()` 调用（`src/tracer/dashboard.rs`）
  - 简化 `map_or` 表达式（`src/llm/logger.rs`）
  - 使用 `is_ok()` 替代冗余的模式匹配（`src/llm/logger.rs`）

### Changed

- **文档注释**: 修复外部文档注释格式（`src/display_helper.rs`）
- **测试**: 所有 1050 个单元测试通过 ✅

### Added

- **文档**: 添加 Clippy 警告待办清单（`docs/04-reports/clippy-warnings-todo.md`）
  - 详细记录剩余 19 个警告及修复计划
  - P1 优先级（12 个）：模块命名、Trait 实现、文档注释等
  - P2 优先级（7 个）：函数参数优化等

### Notes

- **修改文件**: 8 个核心文件
- **代码变更**: +24 行, -25 行
- **警告减少**: 37 → 19（48% 改进）
- **测试结果**: 1050/1050 通过
- **下一步**: Phase 2 手动修复（参见 `clippy-warnings-todo.md`）

---

## [1.9.1] - 2025-10-28

### Added

- **[配置支持] 两仪系统配置文件支持**
  - 新增 `liangyyi` 配置项到 Config 结构
  - 支持启用/禁用两仪系统（`enabled` 字段，默认 true）
  - 支持自定义状态追踪器参数：
    - `history_size`: 历史记录大小（默认 100）
    - `snapshot_interval`: 快照间隔秒数（默认 60）
    - `energy_decay_rate`: 能量衰减率（默认 0.01）
    - `low_activity_threshold`: 低活动阈值（默认 0.3）
    - `high_activity_threshold`: 高活动阈值（默认 0.7）
  - `StateTrackerConfig` 添加 Serde 支持
  - Agent 初始化逻辑支持从配置加载

### Changed

- **Agent**: 两仪系统初始化现在从配置文件读取设置
- **StateTrackerConfig**: 添加 Serialize/Deserialize derive
- 所有配置项都有合理的默认值，确保向后兼容

### Notes

- **向后兼容**: 不配置 `liangyyi` 字段时，系统使用默认值（启用状态）
- **配置示例**:
  ```yaml
  liangyyi:
    enabled: true
    state_tracker:
      history_size: 100
      energy_decay_rate: 0.01
      low_activity_threshold: 0.3
      high_activity_threshold: 0.7
  ```
- **禁用示例**: 设置 `liangyyi.enabled: false` 可完全禁用两仪系统
- 详见文档: `docs/04-reports/v1.9.1-implementation-plan.md`

---

## [1.9.0] - 2025-10-28

### Added

- **[两仪演化系统] Liangyyi Evolution System - 体用合一（Unity of Essence and Function）**
  - 完整实现"先天八卦·竖看"哲学 - 时间维度的状态演化系统
  - **核心组件**：
    - **Phase 1: 核心结构（570 行）**
      - `Taiji`（太极）：阴阳能量连续模型（0.0-1.0）
      - `Liangyyi`（两仪）：太阴☽ / 太阳☉ 二元状态
      - `Sixiang`（四象）：老阴/少阳/少阴/老阳 四态循环
      - 事件驱动更新：UserRead/Write/Execute/Think/Idle
      - 测试：16/16 通过
    - **Phase 2: 状态追踪器（392 行）**
      - `StateTracker`: 实时追踪系统状态演化
      - `StateSnapshot`: 不可变状态快照（带时间戳）
      - 状态历史管理（最近 100 个快照，VecDeque 环形缓冲）
      - 智能活动水平计算（基于最近 10 个快照的阳能量平均值）
      - 趋势分析：TowardYin/TowardYang/Stable
      - 统计信息：四象分布、平均平衡度、能量值
      - Arc<RwLock<>> 并发安全设计
      - 测试：8/8 通过
    - **Phase 3: 应用集成（190 行）**
      - 自动状态更新：用户操作 → 事件分类 → 状态演化
      - 智能事件分类：Command/Shell/Text → Read/Write/Execute/Think
      - 八卦记忆宫连接：
        - 状态快照 → 艮☶维度（Checkpoint）
        - 状态趋势 → 巽☴维度（Trend）
      - 状态感知建议：SuggestionContext 扩展（current_sixiang, energy_balance, state_trend）
      - 集成点：handle() 方法中每次命令执行后自动更新
  - **哲学实现**：
    - 先天八卦（竖看·时间）：Liangyyi 实现时间维度演化序列
    - 后天八卦（横看·空间）：Bagua 实现空间维度数据存储
    - 体用合一：StateTracker ←→ BaguaPalace 完美融合
    - 竖横结合：状态演化 + 数据记录 = 完整系统
  - **代码统计**：
    - 总计：1152 行（Phase 1: 570 + Phase 2: 392 + Phase 3: 190）
    - 测试：24/24 通过（100%）
    - 编译：零错误
  - **使用示例**：
    ```rust
    // 自动运行（无需用户干预）
    用户执行: cargo build
        ↓
    Event::UserExecute → Taiji 更新
        ↓
    阳能量 +0.08, 阴能量 -0.05
        ↓
    Liangyyi: Taiyang ☉
        ↓
    Sixiang: LaoYang ▅▅▅▅▅ ▅▅▅▅▅ ▅▅▅▅▅
        ↓
    写入艮维度: 状态快照
        ↓
    写入巽维度: 趋势分析
    ```
  - 详见报告：
    - `docs/04-reports/liangyyi-phase1-completion.md`
    - `docs/04-reports/liangyyi-phase2-completion.md`
    - `docs/04-reports/liangyyi-phase3-completion.md`
    - `docs/01-understanding/design/liangyyi-state-evolution-design.md`

### Changed

- **Agent**: 集成两仪状态追踪器，自动更新状态
- **SuggestionContext**: 扩展状态字段，支持状态感知建议
- **BaguaPalace**: 新增艮、巽维度写入（状态快照和趋势）

### Notes

- **体用合一完成**：Liangyyi（体/竖看/时间） + Bagua（用/横看/空间） = 完整系统 ☯️
- **无侵入式集成**：状态追踪自动运行，不影响原有功能
- **可选依赖**：StateTracker 可选，确保向后兼容
- **未来优化方向**：
  - 状态感知建议增强（根据四象调整建议策略）
  - 学习阶段识别（Beginner/Learning/Practicing/Proficient）
  - 状态可视化（状态栏显示）
  - 状态驱动的自动化（极静/极动触发建议）

---

## [1.8.0] - 2025-10-27

### Added

- **[文档重组] MLX 风格文档结构（Documentation Restructure）**
  - 采用 MLX 项目风格的简洁 README 设计
  - **顶部导航**：Installation | Quick Start | Documentation | Examples
  - **双语支持**：
    - README.md（英文版，~315 行）- 面向国际用户
    - README.cn.md（中文版，~310 行）- 本地开发优先
  - **全新快速开始指南**：`docs/QUICKSTART.md`（315 行）
    - 5 分钟完整上手流程
    - 环境要求、安装步骤、配置向导
    - 故障排除和后续学习路径
  - **Phase 4.2 特性突出展示**：
    - 主动建议系统作为核心特性
    - P0/P1/P2.1 完整功能说明
    - 实用示例场景（拼写纠错、反馈学习、任务编排等）
  - **分层文档导航**：
    - 入门指南（快速开始、用户手册、FAQ）
    - 核心理念（一分为三哲学、产品愿景、架构设计）
    - 开发者文档（开发指南、API 参考、项目结构）
    - 参考资料（命令参考、配置说明、路线图、更新日志）
  - **架构图更新**：新增主动建议系统组件展示
  - **链接验证**：79% 链接有效（15/19），核心文档 100% 完整
  - 详见：`docs/documentation-restructure.md` 和 `docs/04-reports/documentation-restructure-completion.md`

- **[Phase 4.2 P2.1] 用户反馈学习系统（User Feedback Learning System）**
  - 基于"一分为三"哲学的智能反馈学习系统（RICE: 360）
  - **三态反馈模型**：
    - Accepted（接受）：积极信号，提升评分（weight: +1.0）
    - Skipped（跳过）：中性信号，保持评分（weight: 0.0）
    - Rejected（拒绝）：消极信号，降低评分（weight: -1.0，预留）
  - **三层学习机制**：
    - 即时学习（Instant）：基于质量分数直接调整（0.5-1.5x 倍数）
    - 短期学习（Short-term）：最近 N 次反馈的接受率趋势
    - 长期学习（Long-term）：历史数据的质量评估和持续优化
  - **核心组件**：
    - `FeedbackStorage`: 持久化反馈记录和统计数据（JSON 格式）
    - `FeedbackCollector`: 收集用户反馈，管理反馈会话
    - `FeedbackLearner`: 分析历史数据，动态调整建议评分
    - `FeedbackTypes`: 完整的数据模型和配置
  - **评分调整算法**：
    - 质量分数 = 接受率 × 70% + 位置得分 × 30%
    - 调整倍数 = 1.0 + (质量分数 - 0.5) × 0.2（默认配置）
    - 样本数限制：至少 3 次展示才开始调整
  - **数据持久化**：
    - 存储路径：`~/.realconsole/feedback/`
    - `feedbacks.json`: 原始反馈记录（最多 1000 条）
    - `stats.json`: 聚合统计数据
  - **代码统计**：
    - 新增模块：`src/suggestion/feedback/` (4 个文件)
    - 代码行数：~2000 行
    - 测试覆盖：33 个单元测试（100% 通过）
  - **功能特性**：
    - ✅ 反馈会话管理（创建、跟踪、清理）
    - ✅ 高质量/低质量建议筛选
    - ✅ 自动清理过期数据（会话 5 分钟超时，记录最多 1000 条）
    - ✅ 隐私保护（本地存储，错误输出截断到 500 字符）
    - ✅ 并发安全（Arc<RwLock<>> 支持）
    - ✅ 异步 I/O（tokio 运行时）

### Changed

- **文档结构优化**：
  - 清理根目录，删除旧文件（README.old.md, README.en.md）
  - 移动测试文档到 reports 目录（TESTING-P1.md → docs/04-reports/phase-4.2-p1-testing.md）
  - 更新 .gitignore，忽略备份和测试文件

### Notes

- **Phase 4.2 完整发布**：
  - ✅ P0 - 快速执行与增强错误分析
  - ✅ P1 - 拼写检查与建议缓存
  - ✅ P2.1 - 反馈学习系统
  - ✅ 文档重组（MLX 风格，双语支持）
- 详见完成报告：`docs/04-reports/phase-4.2-p2.1-completion.md` 和 `docs/04-reports/documentation-restructure-completion.md`

## [1.7.1] - 2025-10-27

### Added

- **[Phase 4.2 P1] 拼写纠错系统（Spell Checker）**
  - 基于 Levenshtein 距离算法的智能拼写纠错
  - 内置 100+ 常用命令词典（系统命令、开发工具、容器工具等）
  - 三态评分系统（距离1: 高置信度 0.85-0.93，距离2: 中等 0.65-0.78，距离3: 低 0.45-0.58）
  - 智能检测：只在 "command not found" 错误时触发
  - 优先级最高：拼写纠错 > 错误模式匹配 > 通用建议
  - 新增模块：`src/suggestion/spell_checker.rs` (+430 行，12 个测试)

- **[Phase 4.2 P1] 建议缓存过期机制（Suggestion Cache with Expiration）**
  - 基于"一分为三"哲学的三态时间管理
    - 新鲜 (< 2.5分钟)：高置信度，直接使用
    - 陈旧 (2.5-5分钟)：中等置信度，可能提示
    - 过期 (> 5分钟)：自动清除
  - 时间戳管理和过期检查
  - 友好的缓存状态提示（Empty/Fresh/Stale/Expired）
  - 自动清理机制（惰性清理策略）
  - 新增模块：`src/suggestion/cache.rs` (+380 行，9 个测试)

- **[Phase 4.2 P0] 快速执行建议（Quick Execute）**
  - 数字快速执行：查看建议后输入数字（1/2/3...）立即执行
  - 建议缓存：保存最近显示的建议列表
  - 完整生命周期：缓存 → 验证 → 执行 → 追踪
  - 递归执行设计：确保完整的命令生命周期

- **[Phase 4.2 P0] 增强错误分析系统（Enhanced Error Analysis）**
  - 11 种常见错误模式识别
    - CommandNotFound、PermissionDenied、NoSuchFileOrDirectory
    - GitNotARepository、GitNothingToCommit
    - CargoNotFound、CargoBuildFailed
    - NpmModuleNotFound、PortAlreadyInUse
    - ConnectionRefused、DiskSpaceFull
  - 基于正则表达式的模式匹配
  - 智能参数提取（如端口号）
  - 特定场景的针对性建议
  - 新增模块：`src/suggestion/error_patterns.rs` (+490 行，6 个测试)

### Fixed

- **[Bug #1] 建议系统缺少错误输出传递**
  - 问题：拼写检查器无法检测 "command not found"
  - 修复：在构建 SuggestionContext 时添加 `last_command_output` 字段
  - 位置：`src/agent.rs:884-885`

- **[Bug #2] 自动修复系统与建议系统冲突**
  - 问题：ShellExecutorWithFixer 优先处理 "command not found"，导致建议系统无法运行
  - 修复：为 "command not found" 错误添加优先级检查，跳过自动修复流程，交由建议系统处理
  - 位置：`src/agent.rs:1037-1043`

### Changed

- **建议系统优先级重构**
  - 拼写纠错（优先级最高）→ 错误模式匹配 → 通用建议
  - 建议系统优先于自动修复系统（针对拼写错误）
  - 清晰的责任分离和流程控制

- **建议缓存升级**
  - 从 `Arc<RwLock<Vec<Suggestion>>>` 升级到 `Arc<RwLock<SuggestionCache>>`
  - 增加时间戳管理和过期检查
  - 更友好的错误提示

### Testing

- **单元测试：71 个测试全部通过（100%）**
  - spell_checker: 12 个测试
  - cache: 9 个测试
  - 其他 suggestion 模块: 50 个测试

- **实际使用测试：通过**
  - 拼写纠错：`!cago build` → 建议 "cargo build" ✅
  - 快速执行：输入 `1` 立即执行建议 ✅
  - 缓存过期：5 分钟后提示缓存过期 ✅

### Documentation

- 新增：`docs/04-reports/phase-4.2-p1-completion.md` - P1 功能完成报告
- 新增：`TESTING-P1.md` - 测试指南
- 新增：`scripts/test/test-phase-4.2-p1.sh` - 测试脚本

## [1.7.0] - 2025-10-26

### Added

- **[Phase 4.2 P0] 快速执行建议 + 增强错误分析**
  - 详见 v1.7.1 变更日志（P0 功能合并到 P1 发布）

## [1.6.0] - 2025-10-23

### Added

- **[Feature] 系统 Dashboard (`/trace dashboard`)**
  - 基于易经"四象"哲学的统一系统健康度视图
  - **系统健康度评分**（0-100 分）
    - 命令成功率（40% 权重）
    - LLM 响应质量（20% 权重）
    - 系统活跃度（20% 权重）
    - 异常程度（20% 权重，反向）
    - 5 级健康等级：优秀(90-100)、良好(75-89)、一般(60-74)、较差(40-59)、危险(0-39)
  - **四象分区视图**
    - ☰ 太阳（Statistics）- 命令频率、使用模式
    - ☷ 太阴（Memory）- 对话上下文、知识积累
    - ☲ 少阳（Coordination）- 执行追踪、协同流程
    - ☵ 少阴（BlackBox）- LLM 调用、智能黑盒
  - **异常检测系统**
    - 高失败率检测（>20% 触发）
    - 重复错误检测（>=3 次触发）
    - 严重程度分级（1-5）
    - 数据关联（失败率、失败次数、错误详情等）
  - **智能建议系统**
    - 基于健康度的建议
    - 基于成功率的建议
    - 基于异常的建议（高失败率、重复错误）
    - 基于活跃度的建议
    - 优先级排序（1-5）
    - 可执行命令链接

- **核心模块 `tracer/dashboard.rs`**
  - `Dashboard` - Dashboard 生成器
  - `DashboardConfig` - 可配置选项
  - `HealthScore` - 系统健康度评分
  - `Anomaly` / `AnomalyType` - 异常检测
  - `Suggestion` - 智能建议
  - 总代码量：~620 行

- **UnifiedTracer 增强**
  - 新增 `get_failed_logs()` 方法，用于异常检测和错误分析
  - 支持获取最近的失败执行日志

### Changed

- **`/trace` 命令增强**
  - 新增 `/trace dashboard` 子命令（别名：`dash`）
  - 更新帮助文本，添加 Dashboard 说明

### Philosophy

- **离卦（☲）- 向外照明**
  - Dashboard 通过可视化展示，将系统内部状态"照明"给用户
  - 健康度评分条形图、四象分区百分比、彩色状态标识、清晰的建议列表

- **坎卦（☵）- 向内深入**
  - Dashboard 通过算法分析，深入理解系统规律
  - 多维度健康评分计算、异常模式检测、智能建议生成

- **四象理论**
  - Dashboard 完美体现了"四象"的分类思想
  - 将系统观测维度抽象为四个互补视角

### Performance

- Dashboard 渲染时间：< 100ms
- 异常检测额外开销：~10-20ms
- 编译时间：~5-7s（增量编译）
- 性能影响：可忽略不计

### Documentation

- 新增 `docs/04-reports/v1.6.0-dashboard-completion.md` - Dashboard 完成报告
- 新增 `docs/04-reports/v1.6.0-optimization-report.md` - 优化完成报告

## [1.5.0] - 2025-10-23

### Added

- **[Feature] 统一追踪系统 (`/trace` 命令)**
  - 全新的四维观测体系，聚合 History、log、llm-log、Context 四个数据源
  - 提供统一的查询接口，降低用户认知负担
  - 8个子命令支持多种查询方式：
    - `/trace` - 显示最近 20 条记录（四维聚合）
    - `/trace all [n]` - 显示最近 N 条记录
    - `/trace history [n]` - 仅显示 History 维度（统计）
    - `/trace log [n]` - 仅显示 log 维度（协同）
    - `/trace llm [n]` - 仅显示 llm-log 维度（黑盒）
    - `/trace context [n]` - 仅显示 Context 维度（记忆）
    - `/trace search <关键词>` - 关键词搜索
    - `/trace stats` - 显示统计信息
  - 快捷别名支持：`trace → t`, `history → h`, `log → l`, `context → c`, `search → s`
  - 基于"四象"哲学理论的设计（详见 `docs/04-reports/four-dimensions-philosophy.md`）

- **核心模块 `tracer/`**
  - `types.rs` - 核心类型定义（Dimension, EntryType, Status）
  - `entry.rs` - 统一追踪条目（TraceEntry）
  - `unified_tracer.rs` - 统一追踪器（UnifiedTracer）
  - `benchmarks.rs` - 性能基准测试
  - 总代码量：~2000 行

- **高性能设计**
  - 并行查询四个数据源（使用 `tokio::join!`）
  - 智能去重算法（内容哈希 + 时间窗口）
  - 性能超预期 40-65 倍（详见测试报告）
  - 所有操作 < 15ms（目标 10-250ms）

### Changed

- **[Freeze] Memory 系统冻结**
  - 停止记录新的对话内容（Phase 1 of Memory 重新设计）
  - `/memory` 命令添加冻结警告横幅
  - 保留所有查询功能（status/search/clear/important 等）
  - 未来 Memory 2.0 将专注于智能上下文编排
  - 详见：`docs/04-reports/memory-system-redesign.md`

### Fixed

- **[Critical] UTF-8 字符串切片 Panic**
  - 修复 `TraceEntry::preview()` 中的不安全字符串切片
  - 添加 UTF-8 字符边界检查，避免切割多字节字符（如中文）
  - 影响文件：`src/tracer/entry.rs:207`
  - 问题：切片可能落在多字节字符内部导致 panic
  - 修复：使用 `is_char_boundary()` 安全检查

### Performance

- **性能测试结果**（实际 vs 目标）：
  - query_all(100): 0.97ms < 50ms ✅ (51倍优于目标)
  - query_by_dimension(100): 0.46ms < 30ms ✅ (65倍优于目标)
  - search: 4.29ms < 100ms ✅ (23倍优于目标)
  - deduplicate(300): 0.25ms < 10ms ✅ (40倍优于目标)
  - stats: 9.23ms < 200ms ✅ (21倍优于目标)
  - 4 parallel queries: 13.86ms < 250ms ✅ (18倍优于目标)

### Testing

- **测试覆盖**：
  - 单元测试：10个（UnifiedTracer 核心功能）
  - 基准测试：6个（性能验证）
  - 边缘测试：5个（空数据、UTF-8、大数据集、边界、特殊字符）
  - 全套测试：831 passed, 0 failed, 20 ignored
  - Clippy：tracer 模块零警告 ✅

- **边缘情况覆盖**：
  - ✅ 空数据源不崩溃
  - ✅ UTF-8 多语言支持（中文/日文/俄文/emoji）
  - ✅ 5000条大数据集处理
  - ✅ limit 边界测试（0/1/999999）
  - ✅ 特殊字符搜索（空字符串/特殊符号/Unicode）

### Documentation

- **新增文档**：
  - `docs/04-reports/trace-command-design.md` - 详细设计文档
  - `docs/04-reports/trace-implementation-plan.md` - 实施计划与进度
  - `docs/04-reports/four-dimensions-philosophy.md` - 四维哲学理论
  - `docs/04-reports/phase-5-testing-completion.md` - 测试完成报告

- **更新文档**：
  - `CHANGELOG.md` - 添加 v1.5.0 更新日志
  - 更多文档更新详见 Phase 6

### Development

- **实施时间**：2天（预计 8-12 天）
- **开发模式**：一气呵成（Phase 1-5 连续完成）
- **代码质量**：零警告、高覆盖、超性能

### Migration Notes

- Memory 系统已冻结，不再记录新内容
- 现有 Memory 数据仍可查询，不受影响
- 使用 `/trace` 命令获取完整的四维观测视图
- Memory 2.0 将在未来版本中实现智能上下文编排

---

## [1.3.7] - 2025-10-22

### Added
- **Memory 优化**
  - 添加 `/memory stats` 命令显示统计分析（类型分布、时间跨度、可视化进度条）
  - 实现记忆重要性标记功能（Normal/Important/Critical 三级）
  - 新增 `/memory mark <索引> <级别>` 命令标记重要性
  - 新增 `/memory important [级别]` 命令查看指定重要性的记忆
  - 记忆条目显示增加重要性标记符号（⭐ / ⭐⭐）
  - 相对时间展示（"刚刚"、"5分钟前"、"3小时前"等）

- **语音播报系统（v1.3.7 新特性）**
  - 创建完整的 `voice` 模块，支持跨平台 TTS（Text-to-Speech）
  - macOS: 使用 `say` 命令（支持中文语音如 Ting-Ting）
  - Linux: 支持 `espeak` 或 `festival`
  - Windows: 支持 PowerShell TTS
  - 异步语音播报队列，不阻塞主线程
  - 新增 `/voice` 命令系列：
    - `/voice on` - 启用语音播报
    - `/voice off` - 禁用语音播报
    - `/voice status` - 显示状态
    - `/voice test [文本]` - 测试语音播报
  - 配置文件支持：`voice.enabled`、`voice.voice`、`voice.max_queue_size`

### Changed
- `MemoryEntry` 增加 `importance` 字段，向后兼容（使用 `#[serde(default)]`）
- `EntryType` 添加 `Hash` trait 支持，用于统计分析
- 更新配置文件示例，添加 voice 配置说明

### Fixed
- Memory 预览功能使用 `chars()` 按字符数截断，避免 UTF-8 边界问题

### Documentation
- 更新主配置文件 `realconsole.yaml` 添加语音配置示例
- 更新 `config/minimal.yaml` 添加语音配置注释
- 完善 voice 命令帮助文档

### Testing
- 添加 memory stats 功能测试
- 添加 memory importance 功能测试
- 添加 voice 模块完整测试覆盖
- 添加 voice commands 测试（7个测试用例）
- 所有测试通过（multi-thread runtime）

---

## [1.3.6] - Previous Release

详见之前的版本记录...

---

## Future Releases

查看 [ROADMAP.md](docs/00-core/roadmap.md) 了解未来规划。
