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

/// 状态快照
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub taiji: Taiji,
    pub liangyyi: Liangyyi,
    pub sixiang: Sixiang,
    pub timestamp: DateTime<Utc>,
}

/// 状态追踪器配置
#[derive(Debug, Clone)]
pub struct StateTrackerConfig {
    /// 历史记录大小
    pub history_size: usize,

    /// 快照间隔（秒）
    pub snapshot_interval: u64,

    /// 能量衰减率（每秒）
    pub energy_decay_rate: f64,

    /// 低活动阈值
    pub low_activity_threshold: f64,

    /// 高活动阈值
    pub high_activity_threshold: f64,
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

impl StateTracker {
    /// 创建新的追踪器
    pub fn new(config: StateTrackerConfig) -> Self {
        Self {
            current_taiji: Arc::new(RwLock::new(Taiji::new())),
            current_sixiang: Arc::new(RwLock::new(Sixiang::LaoYin)),
            state_history: Arc::new(RwLock::new(VecDeque::with_capacity(
                config.history_size,
            ))),
            config,
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

        StateSnapshot {
            taiji,
            liangyyi,
            sixiang,
            timestamp: Utc::now(),
        }
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

        let count = history.len().min(10) as f64;
        (recent_yang / count).clamp(0.0, 1.0)
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
        taiji.decay_to_balance(self.config.energy_decay_rate);
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

        StateStats {
            total_snapshots: history.len(),
            current_sixiang: current.sixiang,
            sixiang_counts,
            avg_balance,
            current_yin_energy: current.taiji.yin_energy,
            current_yang_energy: current.taiji.yang_energy,
        }
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
}
