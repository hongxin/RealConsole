//! 状态追踪器
//!
//! 追踪系统状态的演化历史

use super::liangyyi::Liangyyi;
use super::sixiang::Sixiang;
use super::taiji::{Event, Taiji};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::RwLock;

// ✨ v1.10.0: 八卦宫殿集成
use crate::bagua::{BaguaDimension, BaguaMemoryPalace, MemoryContent, MemoryEntry};
use uuid::Uuid;

// ✨ v1.14.0: 自适应系统集成
use super::adaptive::{AdaptiveSystem, Recommendation, RecommendationAction};

/// 状态追踪器
pub struct StateTracker {
    /// 当前太极状态
    current_taiji: Arc<RwLock<Taiji>>,

    /// 当前四象状态
    current_sixiang: Arc<RwLock<Sixiang>>,

    /// 状态历史（最近 N 个）
    state_history: Arc<RwLock<VecDeque<StateSnapshot>>>,

    /// 配置
    config: Arc<RwLock<StateTrackerConfig>>,

    /// ✨ v1.14.0: 自适应系统（可选）
    ///
    /// 启用后，系统可以根据状态预测自动调整配置参数
    adaptive_system: Option<Arc<RwLock<AdaptiveSystem>>>,
}

/// 状态快照
///
/// ## v1.9.6 扩展
///
/// 增加了更多观测维度，为多维状态空间打基础：
/// - `user_activity_level`: 用户活跃度（基于阳能量）
/// - `system_load`: 系统负载（基于上下文强度）
/// - `learning_efficiency`: 学习效率（基于状态稳定性）
/// - `decision_confidence`: 决策置信度（基于平衡度）
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub taiji: Taiji,
    pub liangyyi: Liangyyi,
    pub sixiang: Sixiang,
    pub timestamp: DateTime<Utc>,

    // ✨ v1.9.6: 新增观测维度
    /// 用户活跃度（0.0-1.0）
    ///
    /// 基于阳能量计算，反映用户的交互强度
    pub user_activity_level: f64,

    /// 系统负载（0.0-1.0）
    ///
    /// 基于上下文强度，反映系统当前的工作负荷
    pub system_load: f64,

    /// 学习效率（0.0-1.0）
    ///
    /// 基于状态稳定性，稳定时学习效率更高
    pub learning_efficiency: f64,

    /// 决策置信度（0.0-1.0）
    ///
    /// 基于阴阳平衡度，越平衡决策越有信心
    pub decision_confidence: f64,
}

/// 状态追踪器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StateTrackerConfig {
    /// 历史记录大小
    #[serde(default = "default_history_size")]
    pub history_size: usize,

    /// 快照间隔（秒）
    #[serde(default = "default_snapshot_interval")]
    pub snapshot_interval: u64,

    /// 能量衰减率（每秒）
    #[serde(default = "default_energy_decay_rate")]
    pub energy_decay_rate: f64,

    /// 低活动阈值
    #[serde(default = "default_low_activity_threshold")]
    pub low_activity_threshold: f64,

    /// 高活动阈值
    #[serde(default = "default_high_activity_threshold")]
    pub high_activity_threshold: f64,
}

fn default_history_size() -> usize {
    100
}

fn default_snapshot_interval() -> u64 {
    60
}

fn default_energy_decay_rate() -> f64 {
    0.01
}

fn default_low_activity_threshold() -> f64 {
    0.3
}

fn default_high_activity_threshold() -> f64 {
    0.7
}

impl Default for StateTrackerConfig {
    fn default() -> Self {
        Self {
            history_size: 100,
            snapshot_interval: 60,
            energy_decay_rate: 0.01,
            low_activity_threshold: 0.3,
            high_activity_threshold: 0.7,
        }
    }
}

// ✨ v1.9.6: StateSnapshot 扩展方法
impl StateSnapshot {
    /// 从当前状态创建快照
    ///
    /// 自动计算所有观测维度
    pub fn from_current_state(taiji: Taiji, liangyyi: Liangyyi, sixiang: Sixiang) -> Self {
        // 计算用户活跃度（基于阳能量）
        let user_activity_level = taiji.yang_energy;

        // 计算系统负载（基于上下文强度）
        let system_load = taiji.context_intensity;

        // 计算学习效率（基于平衡度，平衡时学习效率高）
        let learning_efficiency = taiji.balance();

        // 计算决策置信度（也基于平衡度）
        let decision_confidence = taiji.balance();

        Self {
            taiji,
            liangyyi,
            sixiang,
            timestamp: Utc::now(),
            user_activity_level,
            system_load,
            learning_efficiency,
            decision_confidence,
        }
    }

    /// 获取综合状态得分（0.0-1.0）
    ///
    /// 综合考虑所有维度
    pub fn overall_score(&self) -> f64 {
        (self.user_activity_level * 0.25
            + self.system_load * 0.25
            + self.learning_efficiency * 0.25
            + self.decision_confidence * 0.25)
            .clamp(0.0, 1.0)
    }

    /// 判断是否处于最佳状态
    ///
    /// 各维度都较高时为最佳状态
    pub fn is_optimal(&self) -> bool {
        self.user_activity_level > 0.7
            && self.system_load < 0.8  // 负载不能太高
            && self.learning_efficiency > 0.6
            && self.decision_confidence > 0.6
    }

    // ========== ✨ v1.10.0: 八卦宫殿集成 ==========

    /// 转换为检查点格式（用于存储到艮卦）
    ///
    /// 提取关键状态信息，避免完整序列化
    pub fn to_checkpoint_state(&self) -> String {
        serde_json::json!({
            "yin_energy": self.taiji.yin_energy,
            "yang_energy": self.taiji.yang_energy,
            "context_intensity": self.taiji.context_intensity,
            "context_duration_secs": self.taiji.context_duration.num_seconds(),
            "liangyyi": format!("{:?}", self.liangyyi),
            "sixiang": format!("{:?}", self.sixiang),
            "user_activity_level": self.user_activity_level,
            "system_load": self.system_load,
            "learning_efficiency": self.learning_efficiency,
            "decision_confidence": self.decision_confidence,
            "timestamp": self.timestamp.to_rfc3339(),
        })
        .to_string()
    }

    /// 从检查点恢复（从艮卦读取）
    ///
    /// 重建关键状态，其他部分使用默认值
    pub fn from_checkpoint_state(state: &str) -> anyhow::Result<Self> {
        let data: serde_json::Value = serde_json::from_str(state)?;

        // 重建 Taiji（只恢复关键字段）
        let mut taiji = Taiji::new();
        taiji.yin_energy = data["yin_energy"].as_f64().unwrap_or(0.5);
        taiji.yang_energy = data["yang_energy"].as_f64().unwrap_or(0.5);
        taiji.context_intensity = data["context_intensity"].as_f64().unwrap_or(0.5);

        // 恢复上下文持续时间
        if let Some(secs) = data["context_duration_secs"].as_i64() {
            taiji.context_duration = chrono::Duration::seconds(secs);
        }

        // 解析 Liangyyi（从 Debug 格式）
        let liangyyi = match data["liangyyi"].as_str().unwrap_or("Taiyin") {
            "Taiyin" => Liangyyi::Taiyin,
            "Taiyang" => Liangyyi::Taiyang,
            _ => Liangyyi::Taiyin,
        };

        // 解析 Sixiang（从 Debug 格式）
        let sixiang = match data["sixiang"].as_str().unwrap_or("LaoYin") {
            "LaoYin" => Sixiang::LaoYin,
            "ShaoYang" => Sixiang::ShaoYang,
            "ShaoYin" => Sixiang::ShaoYin,
            "LaoYang" => Sixiang::LaoYang,
            _ => Sixiang::LaoYin,
        };

        // 解析时间戳
        let timestamp = if let Some(ts_str) = data["timestamp"].as_str() {
            chrono::DateTime::parse_from_rfc3339(ts_str)
                .ok()
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(Utc::now)
        } else {
            Utc::now()
        };

        Ok(Self {
            taiji,
            liangyyi,
            sixiang,
            timestamp,
            user_activity_level: data["user_activity_level"].as_f64().unwrap_or(0.5),
            system_load: data["system_load"].as_f64().unwrap_or(0.5),
            learning_efficiency: data["learning_efficiency"].as_f64().unwrap_or(0.5),
            decision_confidence: data["decision_confidence"].as_f64().unwrap_or(0.5),
        })
    }
}

impl StateTracker {
    /// 创建新的追踪器
    pub fn new(config: StateTrackerConfig) -> Self {
        let history_size = config.history_size;
        Self {
            current_taiji: Arc::new(RwLock::new(Taiji::new())),
            current_sixiang: Arc::new(RwLock::new(Sixiang::LaoYin)),
            state_history: Arc::new(RwLock::new(VecDeque::with_capacity(history_size))),
            config: Arc::new(RwLock::new(config)),
            adaptive_system: None,
        }
    }

    /// 使用默认配置创建
    pub fn with_default() -> Self {
        Self::new(StateTrackerConfig::default())
    }

    /// 更新状态（基于事件）
    pub async fn update_from_event(&self, event: Event) {
        // 更新太极
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

        // ✨ v1.9.6: 使用新的构造函数，自动计算所有维度
        StateSnapshot::from_current_state(taiji, liangyyi, sixiang)
    }

    /// 获取状态历史
    pub async fn history(&self) -> Vec<StateSnapshot> {
        self.state_history.read().await.iter().cloned().collect()
    }

    /// 获取最近的 N 个状态
    pub async fn recent_states(&self, count: usize) -> Vec<StateSnapshot> {
        let history = self.state_history.read().await;
        history
            .iter()
            .rev()
            .take(count)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// 计算活动水平（基于最近历史）
    pub async fn calculate_activity_level(&self) -> f64 {
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

        let count = history.len().min(10) as f64;
        (recent_yang / count).clamp(0.0, 1.0)
    }

    /// 记录快照
    async fn record_snapshot(&self, taiji: Taiji, liangyyi: Liangyyi, sixiang: Sixiang) {
        let mut history = self.state_history.write().await;

        // ✨ v1.9.6: 使用新的构造函数，自动计算四个观测维度
        let snapshot = StateSnapshot::from_current_state(taiji, liangyyi, sixiang);

        history.push_back(snapshot);

        // 限制大小
        let config = self.config.read().await;
        if history.len() > config.history_size {
            history.pop_front();
        }
    }

    /// 应用能量衰减
    pub async fn apply_decay(&self) {
        let mut taiji = self.current_taiji.write().await;
        let config = self.config.read().await;
        taiji.decay_to_balance(config.energy_decay_rate);
    }

    /// 分析状态趋势
    pub async fn analyze_trend(&self) -> StateTrend {
        let history = self.state_history.read().await;

        if history.len() < 2 {
            return StateTrend::Stable;
        }

        // 分析最近 5 个状态的趋势
        let recent: Vec<_> = history.iter().rev().take(5).collect();

        let mut yin_increasing = 0;
        let mut yang_increasing = 0;

        for i in 0..recent.len() - 1 {
            let curr = &recent[i].taiji;
            let prev = &recent[i + 1].taiji;

            if curr.yin_energy > prev.yin_energy {
                yin_increasing += 1;
            }
            if curr.yang_energy > prev.yang_energy {
                yang_increasing += 1;
            }
        }

        match (yin_increasing, yang_increasing) {
            (y, _) if y >= 3 => StateTrend::TowardYin,
            (_, y) if y >= 3 => StateTrend::TowardYang,
            _ => StateTrend::Stable,
        }
    }

    /// 清空历史
    pub async fn clear_history(&self) {
        let mut history = self.state_history.write().await;
        history.clear();
    }

    /// 检测学习阶段
    ///
    /// 基于状态历史的波动性和变化率判断用户所处的学习阶段
    pub async fn detect_learning_phase(&self) -> (LearningPhase, f64, f64) {
        let history = self.state_history.read().await;

        if history.len() < 10 {
            // 数据不足，默认为探索期
            return (LearningPhase::Exploration, 0.0, 0.0);
        }

        // 1. 计算能量波动性（使用增量的标准差，而非绝对值）
        // 这能区分"稳定趋势"（低波动）和"无规律摆动"（高波动）
        let energies: Vec<f64> = history
            .iter()
            .map(|s| s.taiji.yang_energy - s.taiji.yin_energy)
            .collect();

        // 计算相邻快照的能量差值变化（二阶导数）
        let mut deltas = Vec::new();
        for i in 1..energies.len() {
            deltas.push(energies[i] - energies[i - 1]);
        }

        let volatility = if !deltas.is_empty() {
            let mean_delta = deltas.iter().sum::<f64>() / deltas.len() as f64;
            let variance: f64 = deltas
                .iter()
                .map(|d| (d - mean_delta).powi(2))
                .sum::<f64>()
                / deltas.len() as f64;
            variance.sqrt()
        } else {
            0.0
        };

        // 2. 计算四象变化率
        let recent: Vec<_> = history.iter().rev().take(20).collect();
        let mut changes = 0;
        for i in 0..recent.len() - 1 {
            if recent[i].sixiang != recent[i + 1].sixiang {
                changes += 1;
            }
        }
        let change_rate = changes as f64 / (recent.len() - 1) as f64;

        // 3. 判断学习阶段
        // 调整阈值以更好地反映实际状态变化
        let phase = if volatility > 0.12 || change_rate > 0.4 {
            LearningPhase::Exploration
        } else if volatility < 0.06 && change_rate < 0.2 {
            LearningPhase::Stability
        } else {
            LearningPhase::Transition
        };

        (phase, volatility, change_rate)
    }

    /// 统计信息
    pub async fn stats(&self) -> StateStats {
        let history = self.state_history.read().await;
        let current = self.current_state().await;

        // 统计各四象出现次数
        let mut sixiang_counts = std::collections::HashMap::new();
        for snapshot in history.iter() {
            *sixiang_counts.entry(snapshot.sixiang).or_insert(0) += 1;
        }

        // 计算平均平衡度
        let total_balance: f64 = history.iter().map(|s| s.taiji.balance()).sum();
        let avg_balance = if !history.is_empty() {
            total_balance / history.len() as f64
        } else {
            current.taiji.balance()
        };

        // 释放历史锁，避免死锁
        drop(history);

        // 检测学习阶段
        let (learning_phase, volatility, sixiang_change_rate) =
            self.detect_learning_phase().await;

        StateStats {
            total_snapshots: self.state_history.read().await.len(),
            current_sixiang: current.sixiang,
            sixiang_counts,
            avg_balance,
            current_yin_energy: current.taiji.yin_energy,
            current_yang_energy: current.taiji.yang_energy,
            learning_phase,
            volatility,
            sixiang_change_rate,
        }
    }

    // ========== ✨ v1.10.0: 八卦宫殿集成方法 ==========

    /// 同步当前状态到八卦宫殿
    ///
    /// ## 存储策略
    ///
    /// - **艮卦（Gen）**: 存储状态快照（检查点）
    /// - **巽卦（Xun）**: 存储状态趋势（模式）
    ///
    /// ## 使用示例
    ///
    /// ```ignore
    /// tracker.sync_to_bagua(&mut palace).await?;
    /// ```
    pub async fn sync_to_bagua(&self, palace: &mut BaguaMemoryPalace) -> anyhow::Result<()> {
        // 1. 获取当前状态快照
        let snapshot = self.current_state().await;

        // 2. 将快照存储到艮卦（Gen - 检查点维度）
        let checkpoint_content = MemoryContent::Checkpoint {
            state: snapshot.to_checkpoint_state(),
            snapshot_id: Uuid::new_v4().to_string(),
            metadata: Some(format!(
                "activity:{:.2},load:{:.2},efficiency:{:.2},confidence:{:.2}",
                snapshot.user_activity_level,
                snapshot.system_load,
                snapshot.learning_efficiency,
                snapshot.decision_confidence
            )),
        };

        let checkpoint_entry = MemoryEntry::new(BaguaDimension::Gen, checkpoint_content)
            .with_energy(snapshot.system_load); // 使用系统负载作为能量值

        palace.store(checkpoint_entry).await?;

        // 3. 分析并存储趋势到巽卦（Xun - 趋势维度）
        let trend = self.analyze_trend().await;
        let trend_str = format!("{:?}", trend);

        // 计算变化率（最近10个快照的阳能量活跃度）
        let change_rate = self.calculate_activity_level().await;

        let trend_content = MemoryContent::Trend {
            pattern: trend_str,
            frequency: self.state_history.read().await.len(),
            change_rate,
        };

        let trend_entry = MemoryEntry::new(BaguaDimension::Xun, trend_content)
            .with_energy(change_rate); // 使用变化率作为能量值

        palace.store(trend_entry).await?;

        Ok(())
    }

    /// 从八卦宫殿恢复状态
    ///
    /// ## 恢复策略
    ///
    /// - 从艮卦（Gen）读取最后的检查点
    /// - 恢复太极、两仪、四象状态
    /// - 恢复观测维度
    ///
    /// ## 使用示例
    ///
    /// ```ignore
    /// let tracker = StateTracker::restore_from_bagua(&palace, config).await?;
    /// ```
    pub async fn restore_from_bagua(
        palace: &BaguaMemoryPalace,
        config: StateTrackerConfig,
    ) -> anyhow::Result<Self> {
        // 1. 从艮卦读取最后的检查点
        let checkpoints = palace.retrieve(BaguaDimension::Gen, None).await?;

        let checkpoint = checkpoints
            .last()
            .ok_or_else(|| anyhow::anyhow!("No checkpoint found in Gen dimension"))?;

        // 2. 提取状态数据
        let state_json = match &checkpoint.content {
            MemoryContent::Checkpoint { state, .. } => state,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid checkpoint format in Gen dimension"
                ))
            }
        };

        // 3. 恢复状态快照
        let snapshot = StateSnapshot::from_checkpoint_state(state_json.as_str())?;

        // 4. 创建新的追踪器并恢复状态
        let tracker = Self::new(config);

        // 恢复当前状态
        *tracker.current_taiji.write().await = snapshot.taiji.clone();
        *tracker.current_sixiang.write().await = snapshot.sixiang;

        // 将恢复的快照添加到历史中
        tracker.state_history.write().await.push_back(snapshot);

        Ok(tracker)
    }

    /// 检查八卦宫殿中是否有可恢复的检查点
    ///
    /// 用于判断是否可以从八卦宫殿恢复状态
    pub async fn has_checkpoint(palace: &BaguaMemoryPalace) -> bool {
        palace.retrieve(BaguaDimension::Gen, Some(1))
            .await
            .map(|entries| !entries.is_empty())
            .unwrap_or(false)
    }

    // ========== ✨ v1.11.0: StateVector 多维状态空间 ==========

    /// 将当前状态导出为 StateVector
    ///
    /// ## 使用场景
    ///
    /// - 多维状态分析
    /// - 状态距离计算
    /// - 状态演化模拟
    ///
    /// ## 示例
    ///
    /// ```ignore
    /// let vec1 = tracker.to_state_vector().await;
    /// // ... 一段时间后 ...
    /// let vec2 = tracker.to_state_vector().await;
    /// let distance = vec1.distance_to(&vec2);
    /// println!("状态变化距离: {:.3}", distance);
    /// ```
    pub async fn to_state_vector(&self) -> crate::liangyyi::StateVector {
        let snapshot = self.current_state().await;
        crate::liangyyi::StateVector::from_snapshot(&snapshot)
    }
}

/// 状态趋势
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateTrend {
    /// 趋向阴（变静）
    TowardYin,
    /// 趋向阳（变动）
    TowardYang,
    /// 稳定
    Stable,
}

/// 学习阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearningPhase {
    /// 探索期：高波动性，用户在尝试新操作
    Exploration,
    /// 稳定期：低波动性，用户形成稳定的工作模式
    Stability,
    /// 转变期：中等波动性，工作模式正在改变
    Transition,
}

/// 状态统计
#[derive(Debug, Clone)]
pub struct StateStats {
    /// 总快照数
    pub total_snapshots: usize,

    /// 当前四象
    pub current_sixiang: Sixiang,

    /// 各四象出现次数
    pub sixiang_counts: std::collections::HashMap<Sixiang, usize>,

    /// 平均平衡度
    pub avg_balance: f64,

    /// 当前阴能量
    pub current_yin_energy: f64,

    /// 当前阳能量
    pub current_yang_energy: f64,

    /// 学习阶段
    pub learning_phase: LearningPhase,

    /// 波动性指标（标准差）
    pub volatility: f64,

    /// 四象变化率
    pub sixiang_change_rate: f64,
}

// ========== ✨ v1.14.0: 自适应系统集成 ==========

impl StateTracker {
    /// 启用自适应优化
    ///
    /// 使用指定的目标状态启用自适应系统
    ///
    /// ## 示例
    ///
    /// ```rust,no_run
    /// use realconsole::liangyyi::{StateTracker, StateTrackerConfig};
    /// use realconsole::liangyyi::adaptive::TargetState;
    ///
    /// # tokio_test::block_on(async {
    /// let mut tracker = StateTracker::new(StateTrackerConfig::default());
    /// tracker.enable_adaptive(TargetState::balanced());
    ///
    /// // 系统将自动根据状态预测调整配置参数
    /// tracker.auto_optimize().await.unwrap();
    /// # });
    /// ```
    pub fn enable_adaptive(&mut self, target: super::adaptive::TargetState) {
        let adaptive = AdaptiveSystem::new(target);
        self.adaptive_system = Some(Arc::new(RwLock::new(adaptive)));
    }

    /// 检查是否启用了自适应
    pub fn is_adaptive_enabled(&self) -> bool {
        self.adaptive_system.is_some()
    }

    /// 应用建议到配置
    ///
    /// 根据建议调整 StateTrackerConfig 的参数
    ///
    /// ## 映射规则
    ///
    /// - `efficiency` → `energy_decay_rate`: 效率低时增加衰减率（快速重置）
    /// - `activity` → `low/high_activity_threshold`: 调整活动阈值
    /// - `load` → `snapshot_interval`: 负载高时减少间隔（更频繁观测）
    /// - `context` → `history_size`: 上下文高时增加历史大小
    pub async fn apply_recommendations(
        &self,
        recommendations: &[Recommendation],
    ) -> anyhow::Result<()> {
        let mut config = self.config.write().await;

        for rec in recommendations {
            match rec.dimension.as_str() {
                "efficiency" => {
                    // 效率低 → 增加衰减率（让系统更快重置）
                    // 效率高 → 减少衰减率（保持状态稳定）
                    match rec.action {
                        RecommendationAction::Enhance => {
                            // 需要提高效率 → 减少衰减
                            config.energy_decay_rate =
                                (config.energy_decay_rate * 0.9).max(0.005);
                        }
                        RecommendationAction::Reduce => {
                            // 需要降低效率（实际是重置系统）→ 增加衰减
                            config.energy_decay_rate =
                                (config.energy_decay_rate * 1.1).min(0.05);
                        }
                        RecommendationAction::Maintain => {}
                    }
                }
                "activity" => {
                    // 活动低 → 降低阈值（更容易触发）
                    // 活动高 → 提高阈值（保持高活动）
                    match rec.action {
                        RecommendationAction::Enhance => {
                            // 需要提高活动 → 降低阈值
                            config.low_activity_threshold =
                                (config.low_activity_threshold * 0.9).max(0.1);
                            config.high_activity_threshold =
                                (config.high_activity_threshold * 0.95).max(0.5);
                        }
                        RecommendationAction::Reduce => {
                            // 需要降低活动 → 提高阈值
                            config.low_activity_threshold =
                                (config.low_activity_threshold * 1.1).min(0.5);
                            config.high_activity_threshold =
                                (config.high_activity_threshold * 1.05).min(0.9);
                        }
                        RecommendationAction::Maintain => {}
                    }
                }
                "load" => {
                    // 负载低 → 增加间隔（节省资源）
                    // 负载高 → 减少间隔（更频繁观测）
                    match rec.action {
                        RecommendationAction::Enhance => {
                            // 需要提高负载处理能力 → 减少间隔
                            config.snapshot_interval = (config.snapshot_interval * 9 / 10).max(10);
                        }
                        RecommendationAction::Reduce => {
                            // 需要降低负载 → 增加间隔
                            config.snapshot_interval =
                                (config.snapshot_interval * 11 / 10).min(300);
                        }
                        RecommendationAction::Maintain => {}
                    }
                }
                "context" => {
                    // 上下文高 → 增加历史（保留更多上下文）
                    // 上下文低 → 减少历史（减少内存）
                    match rec.action {
                        RecommendationAction::Enhance => {
                            // 需要更多上下文 → 增加历史
                            config.history_size = (config.history_size * 11 / 10).min(500);
                        }
                        RecommendationAction::Reduce => {
                            // 需要减少上下文 → 减少历史
                            config.history_size = (config.history_size * 9 / 10).max(20);
                        }
                        RecommendationAction::Maintain => {}
                    }
                }
                _ => {
                    // 其他维度（yin, yang, confidence）暂不映射到配置
                }
            }
        }

        Ok(())
    }

    /// 自动优化配置
    ///
    /// 执行完整的自适应优化循环：
    /// 1. 获取当前状态向量
    /// 2. 添加到自适应系统观测
    /// 3. 生成调整建议
    /// 4. 应用建议到配置
    ///
    /// ## 返回
    ///
    /// 返回应用的建议列表
    ///
    /// ## 示例
    ///
    /// ```rust,no_run
    /// use realconsole::liangyyi::{StateTracker, StateTrackerConfig};
    /// use realconsole::liangyyi::adaptive::TargetState;
    ///
    /// # tokio_test::block_on(async {
    /// let mut tracker = StateTracker::new(StateTrackerConfig::default());
    /// tracker.enable_adaptive(TargetState::balanced());
    ///
    /// // 定期调用自动优化
    /// let recommendations = tracker.auto_optimize().await.unwrap();
    /// for rec in recommendations {
    ///     println!("应用建议: {} - {}", rec.dimension, rec.reason);
    /// }
    /// # });
    /// ```
    pub async fn auto_optimize(&self) -> anyhow::Result<Vec<Recommendation>> {
        let adaptive = self
            .adaptive_system
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("自适应系统未启用，请先调用 enable_adaptive()"))?;

        // 1. 获取当前状态向量
        let state_vector = self.to_state_vector().await;

        // 2. 添加观测
        {
            let mut adaptive = adaptive.write().await;
            adaptive.add_observation(state_vector);
        }

        // 3. 生成建议
        let recommendations = {
            let adaptive = adaptive.read().await;
            adaptive.generate_recommendations()
        };

        // 4. 应用建议
        self.apply_recommendations(&recommendations).await?;

        Ok(recommendations)
    }

    /// 获取自适应系统的建议（不应用）
    ///
    /// 生成建议但不修改配置，用于预览或分析
    pub async fn get_recommendations(&self) -> anyhow::Result<Vec<Recommendation>> {
        let adaptive = self
            .adaptive_system
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("自适应系统未启用"))?;

        let adaptive = adaptive.read().await;
        Ok(adaptive.generate_recommendations())
    }

    /// 获取当前配置（只读）
    pub async fn get_config(&self) -> StateTrackerConfig {
        self.config.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tracker_creation() {
        let tracker = StateTracker::with_default();
        let state = tracker.current_state().await;

        assert_eq!(state.taiji.yin_energy, 0.5);
        assert_eq!(state.taiji.yang_energy, 0.5);
        assert_eq!(state.sixiang, Sixiang::LaoYin);
    }

    #[tokio::test]
    async fn test_update_from_event() {
        let tracker = StateTracker::with_default();

        tracker.update_from_event(Event::UserExecute).await;
        let state = tracker.current_state().await;

        // 执行应该增加阳能量
        assert!(state.taiji.yang_energy > 0.5);
        assert_eq!(state.liangyyi, Liangyyi::Taiyang);
    }

    #[tokio::test]
    async fn test_history_recording() {
        let tracker = StateTracker::with_default();

        for _ in 0..5 {
            tracker.update_from_event(Event::UserRead).await;
        }

        let history = tracker.history().await;
        assert_eq!(history.len(), 5);
    }

    #[tokio::test]
    async fn test_recent_states() {
        let tracker = StateTracker::with_default();

        for _ in 0..10 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        let recent = tracker.recent_states(3).await;
        assert_eq!(recent.len(), 3);
    }

    #[tokio::test]
    async fn test_activity_level_calculation() {
        let tracker = StateTracker::with_default();

        // 多次执行事件，应该提升活动水平
        for _ in 0..5 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        let state = tracker.current_state().await;
        // 高阳能量 + 高活动水平 → 老阳或少阳
        assert!(state.sixiang.is_yang());
    }

    #[tokio::test]
    async fn test_analyze_trend() {
        let tracker = StateTracker::with_default();

        // 连续增加阳能量
        for _ in 0..5 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        let trend = tracker.analyze_trend().await;
        assert_eq!(trend, StateTrend::TowardYang);
    }

    #[tokio::test]
    async fn test_stats() {
        let tracker = StateTracker::with_default();

        tracker.update_from_event(Event::UserRead).await;
        tracker.update_from_event(Event::UserExecute).await;
        tracker.update_from_event(Event::UserThink).await;

        let stats = tracker.stats().await;
        assert_eq!(stats.total_snapshots, 3);
        assert!(stats.sixiang_counts.len() > 0);
    }

    #[tokio::test]
    async fn test_clear_history() {
        let tracker = StateTracker::with_default();

        for _ in 0..5 {
            tracker.update_from_event(Event::UserRead).await;
        }

        assert_eq!(tracker.history().await.len(), 5);

        tracker.clear_history().await;
        assert_eq!(tracker.history().await.len(), 0);
    }

    #[tokio::test]
    async fn test_learning_phase_exploration() {
        let tracker = StateTracker::with_default();

        // 模拟探索期：交替执行和思考，创建大幅度能量摆动
        // UserExecute: yang+0.08, yin-0.05 (向阳摆动)
        // UserThink: yin+0.08, yang-0.05 (向阴摆动)
        // 这会创建高波动性
        for _ in 0..12 {
            tracker.update_from_event(Event::UserExecute).await;
            tracker.update_from_event(Event::UserThink).await;
        }

        let (phase, volatility, change_rate) = tracker.detect_learning_phase().await;

        // 探索期应该有较高的波动性或变化率
        assert!(
            phase == LearningPhase::Exploration,
            "Expected Exploration, got {:?} (volatility={:.3}, change_rate={:.3})",
            phase,
            volatility,
            change_rate
        );
        assert!(
            volatility > 0.08 || change_rate > 0.3,
            "Expected high volatility or change_rate, got volatility={:.3}, change_rate={:.3}",
            volatility,
            change_rate
        );
    }

    #[tokio::test]
    async fn test_learning_phase_stability() {
        let tracker = StateTracker::with_default();

        // 模拟稳定期：重复相同操作，保持四象稳定
        // UserRead: yin+0.05, yang-0.03
        // 持续Read会让系统稳定在太阴-老阴状态
        for _ in 0..24 {
            tracker.update_from_event(Event::UserRead).await;
        }

        let (phase, volatility, change_rate) = tracker.detect_learning_phase().await;

        // 稳定期应该有较低的波动性和变化率
        assert!(
            phase == LearningPhase::Stability,
            "Expected Stability, got {:?} (volatility={:.3}, change_rate={:.3})",
            phase,
            volatility,
            change_rate
        );
        assert!(
            volatility < 0.08 && change_rate < 0.3,
            "Expected low volatility and change_rate, got volatility={:.3}, change_rate={:.3}",
            volatility,
            change_rate
        );
    }

    #[tokio::test]
    async fn test_stats_includes_learning_phase() {
        let tracker = StateTracker::with_default();

        // 添加一些事件
        for _ in 0..10 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        let stats = tracker.stats().await;

        // 验证stats包含学习阶段信息
        assert!(stats.volatility >= 0.0);
        assert!(stats.sixiang_change_rate >= 0.0);
        assert!(stats.sixiang_change_rate <= 1.0);
    }

    // ========== ✨ v1.10.0: 八卦宫殿集成测试 ==========

    #[tokio::test]
    async fn test_checkpoint_state_conversion() {
        use crate::liangyyi::taiji::{Event, Taiji};
        use crate::liangyyi::liangyyi::Liangyyi;
        use crate::liangyyi::sixiang::Sixiang;

        // 创建一个测试状态
        let mut taiji = Taiji::new();
        taiji.update_from_event(&Event::UserExecute);
        taiji.update_from_event(&Event::UserExecute);

        let liangyyi = Liangyyi::from_taiji(&taiji);
        let sixiang = Sixiang::LaoYang;

        let snapshot = StateSnapshot::from_current_state(taiji, liangyyi, sixiang);

        // 测试序列化
        let checkpoint_json = snapshot.to_checkpoint_state();
        assert!(checkpoint_json.contains("yin_energy"));
        assert!(checkpoint_json.contains("yang_energy"));
        assert!(checkpoint_json.contains("context_intensity"));
        assert!(checkpoint_json.contains("user_activity_level"));

        // 测试反序列化
        let restored = StateSnapshot::from_checkpoint_state(&checkpoint_json).unwrap();
        assert!((restored.taiji.yin_energy - snapshot.taiji.yin_energy).abs() < 0.01);
        assert!((restored.taiji.yang_energy - snapshot.taiji.yang_energy).abs() < 0.01);
        assert_eq!(restored.liangyyi, snapshot.liangyyi);
        assert_eq!(restored.sixiang, snapshot.sixiang);
    }

    #[tokio::test]
    async fn test_sync_to_bagua() {
        use crate::bagua::{BaguaDimension, BaguaMemoryPalace};

        let tracker = StateTracker::with_default();

        // 更新一些状态
        for _ in 0..5 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        // 创建八卦宫殿
        let mut palace = BaguaMemoryPalace::new();

        // 同步状态
        tracker.sync_to_bagua(&mut palace).await.unwrap();

        // 验证艮卦（检查点）有数据
        let checkpoints = palace.retrieve(BaguaDimension::Gen, None).await.unwrap();
        assert_eq!(checkpoints.len(), 1);

        // 验证巽卦（趋势）有数据
        let trends = palace.retrieve(BaguaDimension::Xun, None).await.unwrap();
        assert_eq!(trends.len(), 1);
    }

    #[tokio::test]
    async fn test_restore_from_bagua() {
        use crate::bagua::{BaguaDimension, BaguaMemoryPalace};

        let tracker1 = StateTracker::with_default();

        // 更新状态到特定值
        for _ in 0..3 {
            tracker1.update_from_event(Event::UserExecute).await;
        }
        for _ in 0..2 {
            tracker1.update_from_event(Event::UserRead).await;
        }

        let original_state = tracker1.current_state().await;

        // 同步到八卦宫殿
        let mut palace = BaguaMemoryPalace::new();
        tracker1.sync_to_bagua(&mut palace).await.unwrap();

        // 从八卦宫殿恢复新的tracker
        let tracker2 = StateTracker::restore_from_bagua(&palace, StateTrackerConfig::default())
            .await
            .unwrap();

        let restored_state = tracker2.current_state().await;

        // 验证关键状态被正确恢复
        assert!(
            (restored_state.taiji.yin_energy - original_state.taiji.yin_energy).abs() < 0.01,
            "Yin energy should match"
        );
        assert!(
            (restored_state.taiji.yang_energy - original_state.taiji.yang_energy).abs() < 0.01,
            "Yang energy should match"
        );
        assert_eq!(
            restored_state.liangyyi, original_state.liangyyi,
            "Liangyyi should match"
        );
        assert_eq!(
            restored_state.sixiang, original_state.sixiang,
            "Sixiang should match"
        );
    }

    #[tokio::test]
    async fn test_has_checkpoint() {
        use crate::bagua::BaguaMemoryPalace;

        // 空宫殿没有检查点
        let empty_palace = BaguaMemoryPalace::new();
        assert!(!StateTracker::has_checkpoint(&empty_palace).await);

        // 同步后有检查点
        let tracker = StateTracker::with_default();
        tracker.update_from_event(Event::UserRead).await;

        let mut palace = BaguaMemoryPalace::new();
        tracker.sync_to_bagua(&mut palace).await.unwrap();

        assert!(StateTracker::has_checkpoint(&palace).await);
    }

    #[tokio::test]
    async fn test_multiple_syncs() {
        use crate::bagua::{BaguaDimension, BaguaMemoryPalace};

        let tracker = StateTracker::with_default();
        let mut palace = BaguaMemoryPalace::new();

        // 多次同步
        for i in 0..3 {
            tracker.update_from_event(Event::UserExecute).await;
            tracker.sync_to_bagua(&mut palace).await.unwrap();

            // 验证艮卦记录累积
            let checkpoints = palace.retrieve(BaguaDimension::Gen, None).await.unwrap();
            assert_eq!(checkpoints.len(), i + 1);
        }
    }

    #[tokio::test]
    async fn test_checkpoint_metadata() {
        use crate::bagua::{BaguaDimension, BaguaMemoryPalace, MemoryContent};

        let tracker = StateTracker::with_default();
        for _ in 0..5 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        let mut palace = BaguaMemoryPalace::new();
        tracker.sync_to_bagua(&mut palace).await.unwrap();

        let checkpoints = palace.retrieve(BaguaDimension::Gen, Some(1)).await.unwrap();
        assert_eq!(checkpoints.len(), 1);

        // 验证元数据格式
        if let MemoryContent::Checkpoint { metadata, .. } = &checkpoints[0].content {
            assert!(metadata.is_some());
            let meta = metadata.as_ref().unwrap();
            assert!(meta.contains("activity:"));
            assert!(meta.contains("load:"));
            assert!(meta.contains("efficiency:"));
            assert!(meta.contains("confidence:"));
        } else {
            panic!("Expected Checkpoint content");
        }
    }

    // ========== ✨ v1.14.0: 自适应系统测试 ==========

    #[tokio::test]
    async fn test_enable_adaptive() {
        let mut tracker = StateTracker::with_default();
        assert!(!tracker.is_adaptive_enabled());

        tracker.enable_adaptive(crate::liangyyi::adaptive::TargetState::balanced());
        assert!(tracker.is_adaptive_enabled());
    }

    #[tokio::test]
    async fn test_apply_recommendations_efficiency() {
        let tracker = StateTracker::with_default();
        let initial_decay = tracker.get_config().await.energy_decay_rate;

        // 创建提高效率的建议
        let recommendations = vec![crate::liangyyi::adaptive::Recommendation {
            dimension: "efficiency".to_string(),
            action: crate::liangyyi::adaptive::RecommendationAction::Enhance,
            current_value: 0.3,
            target_range: (0.6, 0.8),
            priority: 0.3,
            reason: "Test".to_string(),
        }];

        tracker.apply_recommendations(&recommendations).await.unwrap();

        let new_decay = tracker.get_config().await.energy_decay_rate;
        // 提高效率 → 减少衰减
        assert!(new_decay < initial_decay);
    }

    #[tokio::test]
    async fn test_apply_recommendations_activity() {
        let tracker = StateTracker::with_default();
        let initial_low = tracker.get_config().await.low_activity_threshold;

        // 创建提高活动的建议
        let recommendations = vec![crate::liangyyi::adaptive::Recommendation {
            dimension: "activity".to_string(),
            action: crate::liangyyi::adaptive::RecommendationAction::Enhance,
            current_value: 0.2,
            target_range: (0.5, 0.7),
            priority: 0.3,
            reason: "Test".to_string(),
        }];

        tracker.apply_recommendations(&recommendations).await.unwrap();

        let new_low = tracker.get_config().await.low_activity_threshold;
        // 提高活动 → 降低阈值
        assert!(new_low < initial_low);
    }

    #[tokio::test]
    async fn test_apply_recommendations_load() {
        let tracker = StateTracker::with_default();
        let initial_interval = tracker.get_config().await.snapshot_interval;

        // 创建提高负载处理的建议
        let recommendations = vec![crate::liangyyi::adaptive::Recommendation {
            dimension: "load".to_string(),
            action: crate::liangyyi::adaptive::RecommendationAction::Enhance,
            current_value: 0.8,
            target_range: (0.3, 0.5),
            priority: 0.3,
            reason: "Test".to_string(),
        }];

        tracker.apply_recommendations(&recommendations).await.unwrap();

        let new_interval = tracker.get_config().await.snapshot_interval;
        // 提高负载处理 → 减少间隔
        assert!(new_interval < initial_interval);
    }

    #[tokio::test]
    async fn test_apply_recommendations_context() {
        let tracker = StateTracker::with_default();
        let initial_history = tracker.get_config().await.history_size;

        // 创建增加上下文的建议
        let recommendations = vec![crate::liangyyi::adaptive::Recommendation {
            dimension: "context".to_string(),
            action: crate::liangyyi::adaptive::RecommendationAction::Enhance,
            current_value: 0.3,
            target_range: (0.5, 0.7),
            priority: 0.2,
            reason: "Test".to_string(),
        }];

        tracker.apply_recommendations(&recommendations).await.unwrap();

        let new_history = tracker.get_config().await.history_size;
        // 增加上下文 → 增加历史
        assert!(new_history > initial_history);
    }

    #[tokio::test]
    async fn test_auto_optimize_without_adaptive() {
        let tracker = StateTracker::with_default();

        // 未启用自适应系统时应该返回错误
        let result = tracker.auto_optimize().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("未启用"));
    }

    #[tokio::test]
    async fn test_auto_optimize_with_adaptive() {
        let mut tracker = StateTracker::with_default();
        tracker.enable_adaptive(crate::liangyyi::adaptive::TargetState::balanced());

        // 添加多种不同的状态历史（创造偏离目标的情况）
        for _ in 0..10 {
            tracker.update_from_event(Event::UserExecute).await;
        }
        // 添加一些 SystemIdle 事件来降低活动度
        for _ in 0..5 {
            tracker.update_from_event(Event::SystemIdle).await;
        }

        // 多次调用 auto_optimize 来累积观测（至少2次才能预测）
        let result1 = tracker.auto_optimize().await;
        assert!(result1.is_ok(), "第一次调用应该成功");

        // 再次更新状态
        for _ in 0..3 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        // 第二次调用应该能生成建议了
        let recommendations = tracker.auto_optimize().await.unwrap();

        // 验证生成了建议（所有7个标准维度都会生成）
        assert_eq!(recommendations.len(), 7, "应该为所有7个标准维度生成建议");

        // 验证建议包含优先级信息
        for rec in &recommendations {
            assert!(rec.priority >= 0.0 && rec.priority <= 1.0);
        }
    }

    #[tokio::test]
    async fn test_get_recommendations() {
        let mut tracker = StateTracker::with_default();
        tracker.enable_adaptive(crate::liangyyi::adaptive::TargetState::balanced());

        // 添加多种观测（创造偏离目标的情况）
        for _ in 0..10 {
            tracker.update_from_event(Event::UserExecute).await;
        }
        for _ in 0..5 {
            tracker.update_from_event(Event::SystemIdle).await;
        }

        // 需要多次添加观测才能预测（累积足够观测）
        let _result1 = tracker.auto_optimize().await;
        for _ in 0..3 {
            tracker.update_from_event(Event::UserExecute).await;
        }

        // 再次调用 auto_optimize 以应用调整，然后获取基准配置
        let _ = tracker.auto_optimize().await;

        // 现在获取基准配置
        let initial_config = tracker.get_config().await;

        // 获取建议（不应用）- 这个方法不会修改配置
        let recommendations = tracker.get_recommendations().await.unwrap();

        // 配置不应改变（get_recommendations 不修改配置）
        let final_config = tracker.get_config().await;
        assert_eq!(initial_config.energy_decay_rate, final_config.energy_decay_rate);

        // 应该有建议（所有7个标准维度）
        assert_eq!(recommendations.len(), 7, "应该为所有7个标准维度生成建议");

        // 验证建议包含优先级信息
        for rec in &recommendations {
            assert!(rec.priority >= 0.0 && rec.priority <= 1.0);
        }
    }
}
