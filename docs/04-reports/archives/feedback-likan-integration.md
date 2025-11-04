# 反馈系统 - 离坎炼化炉集成设计

**版本**: v1.8.3+
**日期**: 2025-10-27
**状态**: 设计中

---

## 🎯 目标

将现有的**用户反馈学习系统**集成到**离坎炼化炉**和**八卦记忆宫**，形成完整的自主学习闭环。

## 📊 当前状况

### 已有组件

1. **反馈系统** (`src/suggestion/feedback/`)
   - ✅ `FeedbackCollector` - 收集用户反馈
   - ✅ `FeedbackStorage` - 持久化反馈数据
   - ✅ `FeedbackLearner` - 基于反馈调整评分
   - ✅ `SuggestionStats` - 建议使用统计

2. **离坎炼化炉** (`src/likan/`)
   - ✅ 自主学习循环
   - ⚠️ 暂时使用空的 suggestion stats（src/likan/trigger.rs:62）

3. **八卦记忆宫** (`src/bagua/`)
   - ✅ 八维记忆结构
   - ✅ 兑（☱）维度用于存储用户反馈

### 集成点

```rust
// src/likan/trigger.rs:62
// 暂时使用空的 suggestion stats（Phase 4.4 可集成反馈系统）
let stats = std::collections::HashMap::new();
```

---

## 🏗️ 集成方案

### 架构图

```text
┌─────────────────────────────────────────────────────────────┐
│                     用户交互层                                │
│                                                               │
│  用户 → 接受/拒绝建议 → FeedbackCollector                    │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                   反馈存储层                                  │
│                                                               │
│  FeedbackStorage (JSON) ← → SuggestionStats                  │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ↓
┌─────────────────────────────────────────────────────────────┐
│                八卦记忆宫 - 兑维度                            │
│                                                               │
│  MemoryEntry {                                                │
│    dimension: Dui (☱),                                       │
│    content: Feedback { action, type, score }                 │
│  }                                                            │
└────────────────────────┬────────────────────────────────────┘
                         │
                         ↓
┌─────────────────────────────────────────────────────────────┐
│              离坎炼化炉 - 学习循环                            │
│                                                               │
│  1. Kan (☵) 提取模式 + 反馈统计                              │
│  2. Li (☲) 生成优化建议                                      │
│  3. 更新质量评分                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔧 实施步骤

### Phase 1: 反馈数据桥接（✨ 核心）

#### 1.1 在 LiKanTrigger 中添加 FeedbackStorage

```rust
// src/likan/trigger.rs
use crate::suggestion::feedback::FeedbackStorage;

pub struct LiKanTrigger {
    furnace: Arc<RwLock<LiKanFurnace>>,
    // ... 现有字段
    feedback_storage: Option<Arc<RwLock<FeedbackStorage>>>, // 新增
}
```

#### 1.2 从 FeedbackStorage 获取统计数据

```rust
impl LiKanTrigger {
    pub async fn trigger_once(&self) -> Result<CycleReport> {
        // ... 现有代码

        // 获取反馈统计（替换空HashMap）
        let stats = if let Some(ref storage) = self.feedback_storage {
            storage.read().await.get_all_stats().await?
        } else {
            std::collections::HashMap::new()
        };

        // 执行炼化循环
        let mut f = self.furnace.write().await;
        let report = f.cycle_once(&entries, &stats).await?;

        Ok(report)
    }
}
```

#### 1.3 在 Agent 中初始化 FeedbackStorage

```rust
// src/agent.rs:start_likan_background_cycle()

// 加载反馈存储
let feedback_storage = FeedbackStorage::from_default_location()
    .await
    .ok()
    .map(|s| Arc::new(RwLock::new(s)));

// 创建触发器时传递
let trigger = Arc::new(LiKanTrigger::new(
    Arc::clone(&furnace),
    Arc::clone(&history),
    Arc::clone(&exec_logger),
    llm_logger.clone(),
    Arc::clone(&conversation_context),
    feedback_storage, // 新增参数
));
```

### Phase 2: 反馈写入八卦记忆宫

#### 2.1 在 FeedbackCollector 中集成 BaguaMemoryPalace

```rust
// src/suggestion/feedback/collector.rs

use crate::bagua::{BaguaMemoryPalace, BaguaDimension, MemoryEntry, MemoryContent, FeedbackType as BaguaFeedbackType};

impl FeedbackCollector {
    /// 将反馈存入八卦记忆宫（可选）
    async fn store_to_bagua(&self, feedback: &SuggestionFeedback, palace: &BaguaMemoryPalace) -> Result<()> {
        let entry = MemoryEntry::new(
            BaguaDimension::Dui, // 兑维度
            MemoryContent::Feedback {
                action: feedback.suggestion.clone(),
                feedback_type: BaguaFeedbackType::from(&feedback.feedback_type),
                score: feedback.original_score,
            }
        );

        palace.store(entry).await
    }
}
```

#### 2.2 在 Agent 中添加 BaguaMemoryPalace

```rust
// src/agent.rs

pub struct Agent {
    // ... 现有字段
    pub bagua_palace: Option<Arc<BaguaMemoryPalace>>, // 新增
}
```

### Phase 3: 炼化炉使用反馈数据

#### 3.1 修改 LiEnhancer 使用反馈统计

```rust
// src/likan/li.rs

impl LiEnhancer {
    pub async fn enhance_suggestions(
        &mut self,
        suggestions: &mut Vec<Suggestion>,
        patterns: &[Pattern],
        stats: &HashMap<String, SuggestionStats>, // 新增参数
    ) -> Result<EnhanceReport> {
        // 基于反馈统计调整评分
        for suggestion in suggestions.iter_mut() {
            if let Some(stat) = stats.get(&suggestion.command) {
                // 质量评分影响最终排名
                suggestion.score *= stat.quality_score();

                // 低质量建议降权
                if stat.is_low_quality() {
                    suggestion.score *= 0.5;
                }

                // 高质量建议提权
                if stat.is_high_quality() {
                    suggestion.score *= 1.2;
                }
            }
        }

        // ... 其他增强逻辑
    }
}
```

---

## 📐 数据流

### 完整闭环

```text
1. 用户接受建议
   ↓
2. FeedbackCollector 记录 (Accepted)
   ↓
3. FeedbackStorage 持久化
   ↓
4. 写入 Bagua 兑维度
   ↓
5. 离坎炼化炉定时循环
   ↓
6. Kan 提取模式 + 读取反馈统计
   ↓
7. Li 基于反馈优化建议评分
   ↓
8. 新建议质量提升
   ↓
9. 用户继续使用
   ↓
(循环)
```

### 数据类型映射

| 反馈系统 | 八卦记忆宫 | 离坎炼化炉 |
|---------|-----------|-----------|
| `FeedbackType::Accepted` | `FeedbackType::Accept` | 模式权重 +1.0 |
| `FeedbackType::Skipped` | (不存储) | 模式权重 0.0 |
| `FeedbackType::Rejected` | `FeedbackType::Reject` | 模式权重 -1.0 |
| `SuggestionStats.quality_score()` | `energy` | `pattern.confidence` |

---

## 🎯 预期效果

### 量化指标

- **建议接受率**: 从 ~30% 提升到 ~60%
- **高质量模式比例**: 从 ~10% 提升到 ~40%
- **学习收敛时间**: ~100次交互后稳定

### 用户体验

- ✅ 建议越用越准
- ✅ 低质量建议自动淘汰
- ✅ 高频操作自动提权
- ✅ 完全自动，无需人工干预

---

## 🧪 测试策略

### 单元测试

```rust
#[tokio::test]
async fn test_feedback_integration() {
    // 1. 创建反馈
    let collector = FeedbackCollector::from_default_location().await?;

    // 2. 记录接受
    // ... 记录多次反馈

    // 3. 触发炼化
    let trigger = LiKanTrigger::new(...);
    let report = trigger.trigger_once().await?;

    // 4. 验证高质量模式数量增加
    assert!(report.high_confidence_patterns > 0);
}
```

### 集成测试

```bash
# 模拟用户使用场景
1. 启动 RealConsole
2. 重复执行相同操作 10次，每次接受建议
3. 触发炼化循环: /likan cycle
4. 验证该建议的评分提升
5. 验证建议排名靠前
```

---

## 📝 注意事项

### 性能

- FeedbackStorage 使用异步I/O，不阻塞主循环
- 统计数据缓存，避免重复计算
- Bagua 写入异步，不影响交互响应

### 兼容性

- 反馈系统可选，未启用时降级到空统计
- 八卦记忆宫可选，未启用时只用反馈统计
- 向后兼容现有配置

### 隐私

- 反馈数据仅存储本地
- 不包含敏感命令内容
- 可配置完全禁用

---

## 🚀 实施计划

1. ✅ 设计文档（本文档）
2. ⏳ Phase 1: 反馈数据桥接（1-2小时）
3. ⏳ Phase 2: 八卦记忆宫集成（1小时）
4. ⏳ Phase 3: 炼化炉使用反馈（1小时）
5. ⏳ 测试与文档（1小时）

---

**设计者**: RealConsole Team
**审核者**: 待定
**下一步**: 开始 Phase 1 实施 🚀
