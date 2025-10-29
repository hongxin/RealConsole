//! # Adaptive System - 自适应系统
//!
//! ## 设计理念
//!
//! 自适应系统实现智能闭环：观测 → 预测 → 决策 → 建议，体现"自我调节"的智慧：
//! - **目标导向**：定义理想状态，自动向目标演化
//! - **预测驱动**：基于 StatePredictor 的趋势预测
//! - **自适应调整**：根据当前状态自动生成优化建议
//!
//! ## v1.13.0 核心特性
//!
//! - **目标状态定义**：每个维度的理想范围
//! - **自适应建议生成**：基于预测和目标差距
//! - **调整策略**：激进/平衡/保守三种模式
//! - **优先级评估**：自动计算建议的紧急程度
//!
//! ## 使用场景
//!
//! ```rust
//! use realconsole::liangyyi::{AdaptiveSystem, TargetState};
//!
//! // 1. 定义目标状态
//! let mut target = TargetState::balanced(); // 所有维度 0.5-0.7
//! target.set_range("efficiency", 0.7, 0.9); // 高效率目标
//!
//! // 2. 创建自适应系统
//! let mut adaptive = AdaptiveSystem::new(target);
//!
//! // 3. 添加观测
//! adaptive.add_observation(current_state);
//!
//! // 4. 获取建议
//! let recommendations = adaptive.generate_recommendations();
//! for rec in recommendations {
//!     println!("{}", rec.description());
//! }
//! ```

use super::predictor::{StatePredictor, TrendDirection};
use super::state_vector::StateVector;
use std::collections::HashMap;

/// 目标状态定义
///
/// 为每个维度定义理想范围 [min, max]
#[derive(Debug, Clone)]
pub struct TargetState {
    /// 维度名称 -> (最小值, 最大值)
    ranges: HashMap<String, (f64, f64)>,
}

impl TargetState {
    /// 创建空的目标状态
    pub fn new() -> Self {
        Self {
            ranges: HashMap::new(),
        }
    }

    /// 创建平衡的目标状态（所有标准维度 0.5-0.7）
    pub fn balanced() -> Self {
        let mut target = Self::new();
        for dim in &["yin", "yang", "context", "activity", "load", "efficiency", "confidence"] {
            target.set_range(dim, 0.5, 0.7);
        }
        target
    }

    /// 创建高性能目标状态
    pub fn high_performance() -> Self {
        let mut target = Self::new();
        target.set_range("activity", 0.7, 0.9);    // 高活跃
        target.set_range("efficiency", 0.7, 0.9);  // 高效率
        target.set_range("confidence", 0.7, 0.9);  // 高信心
        target.set_range("load", 0.3, 0.6);        // 中等负载
        target
    }

    /// 创建省电模式目标状态
    pub fn power_save() -> Self {
        let mut target = Self::new();
        target.set_range("activity", 0.2, 0.4);    // 低活跃
        target.set_range("load", 0.2, 0.4);        // 低负载
        target.set_range("efficiency", 0.5, 0.7);  // 保持效率
        target
    }

    /// 设置维度范围
    pub fn set_range(&mut self, dimension: &str, min: f64, max: f64) {
        let min = min.clamp(0.0, 1.0);
        let max = max.clamp(0.0, 1.0);
        self.ranges.insert(dimension.to_string(), (min, max));
    }

    /// 检查值是否在目标范围内
    pub fn is_in_range(&self, dimension: &str, value: f64) -> bool {
        if let Some(&(min, max)) = self.ranges.get(dimension) {
            value >= min && value <= max
        } else {
            true // 未定义的维度视为满足
        }
    }

    /// 计算与目标的距离（所有维度的偏离程度之和）
    pub fn distance_to(&self, state: &StateVector) -> f64 {
        let mut total_deviation = 0.0;
        let mut count = 0;

        for (dim, &(min, max)) in &self.ranges {
            if let Some(value) = state.get(dim) {
                let deviation = if value < min {
                    min - value
                } else if value > max {
                    value - max
                } else {
                    0.0 // 在范围内
                };
                total_deviation += deviation;
                count += 1;
            }
        }

        if count > 0 {
            total_deviation / count as f64
        } else {
            0.0
        }
    }

    /// 获取所有定义的维度
    pub fn dimensions(&self) -> Vec<String> {
        let mut dims: Vec<String> = self.ranges.keys().cloned().collect();
        dims.sort();
        dims
    }
}

impl Default for TargetState {
    fn default() -> Self {
        Self::balanced()
    }
}

/// 自适应建议
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// 维度名称
    pub dimension: String,

    /// 建议类型
    pub action: RecommendationAction,

    /// 当前值
    pub current_value: f64,

    /// 目标范围
    pub target_range: (f64, f64),

    /// 优先级 (0.0-1.0，越高越紧急)
    pub priority: f64,

    /// 理由
    pub reason: String,
}

/// 建议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecommendationAction {
    /// 增强（提高维度值）
    Enhance,
    /// 减少（降低维度值）
    Reduce,
    /// 保持（已在目标范围内）
    Maintain,
}

impl Recommendation {
    /// 生成建议描述
    pub fn description(&self) -> String {
        match self.action {
            RecommendationAction::Enhance => {
                format!(
                    "[优先级 {:.2}] 建议增强 {} (当前 {:.2} → 目标 {:.2}-{:.2}): {}",
                    self.priority,
                    self.dimension,
                    self.current_value,
                    self.target_range.0,
                    self.target_range.1,
                    self.reason
                )
            }
            RecommendationAction::Reduce => {
                format!(
                    "[优先级 {:.2}] 建议减少 {} (当前 {:.2} → 目标 {:.2}-{:.2}): {}",
                    self.priority,
                    self.dimension,
                    self.current_value,
                    self.target_range.0,
                    self.target_range.1,
                    self.reason
                )
            }
            RecommendationAction::Maintain => {
                format!(
                    "[优先级 {:.2}] 保持 {} (当前 {:.2} 在目标范围内)",
                    self.priority, self.dimension, self.current_value
                )
            }
        }
    }
}

/// 调整策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdaptiveStrategy {
    /// 激进：快速调整（步长 0.2）
    Aggressive,
    /// 平衡：渐进调整（步长 0.1，默认）
    Balanced,
    /// 保守：缓慢调整（步长 0.05）
    Conservative,
}

impl AdaptiveStrategy {
    /// 获取调整步长
    pub fn step_size(&self) -> f64 {
        match self {
            AdaptiveStrategy::Aggressive => 0.2,
            AdaptiveStrategy::Balanced => 0.1,
            AdaptiveStrategy::Conservative => 0.05,
        }
    }
}

/// 自适应系统
#[derive(Debug, Clone)]
pub struct AdaptiveSystem {
    /// 目标状态
    target: TargetState,

    /// 状态预测器
    predictor: StatePredictor,

    /// 调整策略
    strategy: AdaptiveStrategy,
}

impl AdaptiveSystem {
    // ========== 构造函数 ==========

    /// 创建新的自适应系统
    pub fn new(target: TargetState) -> Self {
        Self {
            target,
            predictor: StatePredictor::new(10),
            strategy: AdaptiveStrategy::Balanced,
        }
    }

    /// 创建带策略的自适应系统
    pub fn with_strategy(target: TargetState, strategy: AdaptiveStrategy) -> Self {
        Self {
            target,
            predictor: StatePredictor::new(10),
            strategy,
        }
    }

    // ========== 数据管理 ==========

    /// 添加观测值
    pub fn add_observation(&mut self, state: StateVector) {
        self.predictor.add_observation(state);
    }

    /// 获取当前距离目标的偏离程度
    pub fn current_deviation(&self) -> Option<f64> {
        let history = &self.predictor;
        if history.history_len() == 0 {
            return None;
        }

        // 获取最后一个观测
        // 由于 predictor.history 是私有的，我们通过预测来间接获取
        // 实际上应该添加一个 get_last_observation 方法，但为了简化先这样
        None // TODO: 需要 predictor 提供获取最后观测的方法
    }

    // ========== 建议生成 ==========

    /// 生成自适应建议
    ///
    /// 基于当前状态、预测趋势和目标状态生成优化建议
    pub fn generate_recommendations(&self) -> Vec<Recommendation> {
        if !self.predictor.can_predict() {
            return vec![];
        }

        let mut recommendations = Vec::new();

        // 预测未来状态
        let predicted = match self.predictor.predict_linear(1) {
            Some(p) => p,
            None => return vec![],
        };

        // 分析趋势
        let trends = self.predictor.analyze_trends();

        // 对每个目标维度生成建议
        for dimension in self.target.dimensions() {
            if let Some((min, max)) = self.target.ranges.get(&dimension).copied() {
                if let Some(current_value) = predicted.get(&dimension) {
                    let (action, priority, reason) = self.analyze_dimension(
                        &dimension,
                        current_value,
                        (min, max),
                        &trends,
                    );

                    recommendations.push(Recommendation {
                        dimension: dimension.clone(),
                        action,
                        current_value,
                        target_range: (min, max),
                        priority,
                        reason,
                    });
                }
            }
        }

        // 按优先级排序
        recommendations.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        recommendations
    }

    /// 分析单个维度，返回（动作, 优先级, 理由）
    fn analyze_dimension(
        &self,
        dimension: &str,
        current_value: f64,
        target_range: (f64, f64),
        trends: &[super::predictor::DimensionTrend],
    ) -> (RecommendationAction, f64, String) {
        let (min, max) = target_range;

        // 找到该维度的趋势
        let trend = trends.iter().find(|t| t.dimension == dimension);

        // 判断当前位置
        if current_value < min {
            // 低于目标
            let deviation = min - current_value;
            let priority = (deviation * 2.0).min(1.0); // 偏离越大优先级越高

            let reason = if let Some(t) = trend {
                match t.direction {
                    TrendDirection::Falling => {
                        format!("当前偏低且趋势下降（强度 {:.2}），需要紧急干预", t.strength)
                    }
                    TrendDirection::Stable => {
                        "当前偏低但趋势稳定，建议适度增强".to_string()
                    }
                    TrendDirection::Rising => {
                        format!("当前偏低但趋势上升（强度 {:.2}），可能自然恢复", t.strength)
                    }
                }
            } else {
                "当前低于目标范围".to_string()
            };

            (RecommendationAction::Enhance, priority, reason)
        } else if current_value > max {
            // 高于目标
            let deviation = current_value - max;
            let priority = (deviation * 2.0).min(1.0);

            let reason = if let Some(t) = trend {
                match t.direction {
                    TrendDirection::Rising => {
                        format!("当前偏高且趋势上升（强度 {:.2}），需要紧急控制", t.strength)
                    }
                    TrendDirection::Stable => {
                        "当前偏高但趋势稳定，建议适度减少".to_string()
                    }
                    TrendDirection::Falling => {
                        format!("当前偏高但趋势下降（强度 {:.2}），可能自然回落", t.strength)
                    }
                }
            } else {
                "当前高于目标范围".to_string()
            };

            (RecommendationAction::Reduce, priority, reason)
        } else {
            // 在目标范围内
            let priority = 0.1; // 低优先级

            let reason = if let Some(t) = trend {
                match t.direction {
                    TrendDirection::Rising if current_value > (min + max) / 2.0 => {
                        "当前良好但上升趋势可能导致超标，建议监控".to_string()
                    }
                    TrendDirection::Falling if current_value < (min + max) / 2.0 => {
                        "当前良好但下降趋势可能导致不足，建议监控".to_string()
                    }
                    _ => "当前在目标范围内，保持即可".to_string(),
                }
            } else {
                "当前在目标范围内".to_string()
            };

            (RecommendationAction::Maintain, priority, reason)
        }
    }

    /// 计算建议的调整向量
    ///
    /// 返回一个 StateVector，表示应该如何调整当前状态
    pub fn calculate_adjustment(&self, current: &StateVector) -> StateVector {
        let mut adjustment = StateVector::new();
        let step_size = self.strategy.step_size();

        for (dim, &(min, max)) in &self.target.ranges {
            if let Some(current_value) = current.get(dim) {
                let target_center = (min + max) / 2.0;
                let delta = target_center - current_value;

                // 按策略调整
                let adjusted_delta = delta * step_size;
                let new_value = current_value + adjusted_delta;

                adjustment.set(dim, new_value);
            }
        }

        adjustment
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vector(activity: f64, efficiency: f64) -> StateVector {
        let mut vec = StateVector::new();
        vec.set("activity", activity);
        vec.set("efficiency", efficiency);
        vec.set("load", 0.5);
        vec
    }

    #[test]
    fn test_target_state_balanced() {
        let target = TargetState::balanced();
        assert_eq!(target.dimensions().len(), 7);
        assert!(target.is_in_range("activity", 0.6));
        assert!(!target.is_in_range("activity", 0.8));
    }

    #[test]
    fn test_target_state_high_performance() {
        let target = TargetState::high_performance();
        assert!(target.is_in_range("activity", 0.8));
        assert!(target.is_in_range("efficiency", 0.8));
        assert!(!target.is_in_range("activity", 0.5));
    }

    #[test]
    fn test_target_distance() {
        let mut target = TargetState::new();
        target.set_range("activity", 0.6, 0.8);
        target.set_range("efficiency", 0.6, 0.8);

        let state1 = create_test_vector(0.7, 0.7); // 在范围内
        assert!((target.distance_to(&state1) - 0.0).abs() < 0.01);

        let state2 = create_test_vector(0.4, 0.4); // 两个维度都低于范围
        let distance = target.distance_to(&state2);
        assert!(distance > 0.15); // 平均偏离约 0.2
    }

    #[test]
    fn test_adaptive_system_creation() {
        let target = TargetState::balanced();
        let adaptive = AdaptiveSystem::new(target);
        assert_eq!(adaptive.strategy, AdaptiveStrategy::Balanced);
    }

    #[test]
    fn test_generate_recommendations() {
        let mut target = TargetState::new();
        target.set_range("activity", 0.6, 0.8);
        target.set_range("efficiency", 0.6, 0.8);

        let mut adaptive = AdaptiveSystem::new(target);

        // 添加观测：activity 低，efficiency 正常
        adaptive.add_observation(create_test_vector(0.3, 0.7));
        adaptive.add_observation(create_test_vector(0.35, 0.7));

        let recommendations = adaptive.generate_recommendations();

        assert!(!recommendations.is_empty());

        // 应该有 activity 的 Enhance 建议
        let activity_rec = recommendations
            .iter()
            .find(|r| r.dimension == "activity")
            .unwrap();
        assert_eq!(activity_rec.action, RecommendationAction::Enhance);
        assert!(activity_rec.priority > 0.3); // 偏离较大，优先级应该高
    }

    #[test]
    fn test_calculate_adjustment() {
        let mut target = TargetState::new();
        target.set_range("activity", 0.6, 0.8);

        let adaptive = AdaptiveSystem::new(target);
        let current = create_test_vector(0.3, 0.7);

        let adjustment = adaptive.calculate_adjustment(&current);

        // 应该向 0.7（中心）调整
        let adjusted_activity = adjustment.get("activity").unwrap();
        assert!(adjusted_activity > 0.3);
        assert!(adjusted_activity < 0.7);
    }

    #[test]
    fn test_strategy_step_size() {
        assert_eq!(AdaptiveStrategy::Aggressive.step_size(), 0.2);
        assert_eq!(AdaptiveStrategy::Balanced.step_size(), 0.1);
        assert_eq!(AdaptiveStrategy::Conservative.step_size(), 0.05);
    }

    #[test]
    fn test_recommendation_description() {
        let rec = Recommendation {
            dimension: "efficiency".to_string(),
            action: RecommendationAction::Enhance,
            current_value: 0.4,
            target_range: (0.6, 0.8),
            priority: 0.8,
            reason: "效率偏低".to_string(),
        };

        let desc = rec.description();
        assert!(desc.contains("efficiency"));
        assert!(desc.contains("0.4"));
        assert!(desc.contains("增强"));
    }
}
