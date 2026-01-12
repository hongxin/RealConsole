//! # StateVector - 多维状态空间
//!
//! ## 设计理念
//!
//! StateVector 实现"一分为三"哲学在多维状态空间中的应用：
//! - **不是二元对立**：状态不是"好/坏"二分，而是多维连续空间
//! - **向量化表示**：每个维度都是 0.0-1.0 的连续值
//! - **动态演化**：支持状态向目标状态的渐进演化
//!
//! ## v1.11.0 核心特性
//!
//! - **多维观测**：基于 v1.9.6 的四个观测维度
//! - **向量运算**：欧几里得距离、状态演化
//! - **灵活扩展**：易于添加新维度
//! - **轻量设计**：最小化依赖，专注核心功能
//!
//! ## 使用场景
//!
//! ```ignore
//! use realconsole::liangyyi::{StateVector, StateSnapshot};
//!
//! // 1. 从快照创建向量
//! let snapshot = tracker.current_state().await;
//! let vec1 = StateVector::from_snapshot(&snapshot);
//!
//! // 2. 计算状态距离
//! let vec2 = StateVector::from_snapshot(&another_snapshot);
//! let distance = vec1.distance_to(&vec2);
//!
//! // 3. 状态演化模拟
//! let mut current = vec1.clone();
//! current.evolve_towards(&vec2, 0.1); // 向 vec2 演化 10%
//! ```

use std::collections::HashMap;

use super::tracker::StateSnapshot;

/// 多维状态向量
///
/// ## 核心设计
///
/// StateVector 使用 HashMap 存储多个维度，每个维度是 [0.0, 1.0] 范围的浮点数。
/// 这种设计允许：
/// - 动态添加/删除维度
/// - 灵活的维度命名
/// - 高效的维度访问
///
/// ## 标准维度（基于 v1.9.6）
///
/// - `yin`: 阴能量
/// - `yang`: 阳能量
/// - `activity`: 用户活跃度
/// - `load`: 系统负载
/// - `efficiency`: 学习效率
/// - `confidence`: 决策置信度
#[derive(Debug, Clone, PartialEq)]
pub struct StateVector {
    /// 维度名称 -> 值（0.0-1.0）
    dimensions: HashMap<String, f64>,
}

impl StateVector {
    // ========== 构造函数 ==========

    /// 创建空向量
    pub fn new() -> Self {
        Self {
            dimensions: HashMap::new(),
        }
    }

    /// 从 StateSnapshot 创建向量
    ///
    /// ## 维度映射
    ///
    /// - `yin` <- taiji.yin_energy
    /// - `yang` <- taiji.yang_energy
    /// - `context` <- taiji.context_intensity
    /// - `activity` <- user_activity_level
    /// - `load` <- system_load
    /// - `efficiency` <- learning_efficiency
    /// - `confidence` <- decision_confidence
    pub fn from_snapshot(snapshot: &StateSnapshot) -> Self {
        let mut dimensions = HashMap::new();

        // 基础维度：阴阳能量
        dimensions.insert("yin".to_string(), snapshot.taiji.yin_energy);
        dimensions.insert("yang".to_string(), snapshot.taiji.yang_energy);
        dimensions.insert("context".to_string(), snapshot.taiji.context_intensity);

        // 观测维度（v1.9.6）
        dimensions.insert("activity".to_string(), snapshot.user_activity_level);
        dimensions.insert("load".to_string(), snapshot.system_load);
        dimensions.insert("efficiency".to_string(), snapshot.learning_efficiency);
        dimensions.insert("confidence".to_string(), snapshot.decision_confidence);

        Self { dimensions }
    }

    /// 创建标准维度的向量（所有维度初始化为 0.5）
    pub fn standard() -> Self {
        let mut dimensions = HashMap::new();
        for dim in &["yin", "yang", "context", "activity", "load", "efficiency", "confidence"] {
            dimensions.insert(dim.to_string(), 0.5);
        }
        Self { dimensions }
    }

    // ========== 维度访问 ==========

    /// 获取维度值
    ///
    /// 如果维度不存在，返回 None
    pub fn get(&self, dimension: &str) -> Option<f64> {
        self.dimensions.get(dimension).copied()
    }

    /// 设置维度值（自动 clamp 到 [0.0, 1.0]）
    pub fn set(&mut self, dimension: &str, value: f64) {
        self.dimensions.insert(dimension.to_string(), value.clamp(0.0, 1.0));
    }

    /// 获取所有维度名称
    pub fn dimension_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.dimensions.keys().cloned().collect();
        names.sort();
        names
    }

    /// 获取维度数量
    pub fn dimension_count(&self) -> usize {
        self.dimensions.len()
    }

    // ========== 向量运算 ==========

    /// 计算到另一个向量的欧几里得距离
    ///
    /// ## 算法
    ///
    /// 只考虑两个向量共有的维度：
    /// ```text
    /// distance = sqrt(Σ(v1[i] - v2[i])²)
    /// ```
    ///
    /// ## 注意
    ///
    /// - 如果没有共同维度，返回 0.0
    /// - 距离范围：[0.0, sqrt(n)]，其中 n 是共同维度数
    pub fn distance_to(&self, other: &StateVector) -> f64 {
        let mut sum_squared = 0.0;
        let mut count = 0;

        // 遍历当前向量的所有维度
        for (dim, &value) in &self.dimensions {
            // 只考虑两个向量都有的维度
            if let Some(&other_value) = other.dimensions.get(dim) {
                let diff = value - other_value;
                sum_squared += diff * diff;
                count += 1;
            }
        }

        if count == 0 {
            return 0.0;
        }

        sum_squared.sqrt()
    }

    /// 向目标向量演化一步
    ///
    /// ## 参数
    ///
    /// - `target`: 目标向量
    /// - `step`: 步长（0.0-1.0），0.0 表示不动，1.0 表示直接到达
    ///
    /// ## 行为
    ///
    /// 对于每个共同维度：
    /// ```text
    /// new_value = current_value + (target_value - current_value) * step
    /// ```
    ///
    /// ## 示例
    ///
    /// ```ignore
    /// let mut vec = StateVector::standard();
    /// vec.set("yin", 0.3);
    ///
    /// let mut target = StateVector::standard();
    /// target.set("yin", 0.7);
    ///
    /// vec.evolve_towards(&target, 0.5); // 向目标移动 50%
    /// assert!((vec.get("yin").unwrap() - 0.5).abs() < 0.001);
    /// ```
    pub fn evolve_towards(&mut self, target: &StateVector, step: f64) {
        let step = step.clamp(0.0, 1.0);

        for (dim, current_value) in &mut self.dimensions {
            if let Some(&target_value) = target.dimensions.get(dim) {
                let delta = (target_value - *current_value) * step;
                *current_value = (*current_value + delta).clamp(0.0, 1.0);
            }
        }
    }

    /// 向量加法（逐维度相加，自动 clamp）
    ///
    /// 只对共同维度进行操作
    pub fn add(&mut self, other: &StateVector) {
        for (dim, other_value) in &other.dimensions {
            if let Some(current_value) = self.dimensions.get_mut(dim) {
                *current_value = (*current_value + *other_value).clamp(0.0, 1.0);
            }
        }
    }

    /// 向量数乘（每个维度乘以标量，自动 clamp）
    pub fn scale(&mut self, scalar: f64) {
        for value in self.dimensions.values_mut() {
            *value = (*value * scalar).clamp(0.0, 1.0);
        }
    }

    // ========== 分析方法 ==========

    /// 计算向量的模（欧几里得范数）
    ///
    /// ```text
    /// norm = sqrt(Σ value[i]²)
    /// ```
    pub fn norm(&self) -> f64 {
        let sum_squared: f64 = self.dimensions.values().map(|v| v * v).sum();
        sum_squared.sqrt()
    }

    /// 计算向量的平均值
    pub fn mean(&self) -> f64 {
        if self.dimensions.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.dimensions.values().sum();
        sum / self.dimensions.len() as f64
    }

    /// 获取最大维度值
    pub fn max_dimension(&self) -> Option<(&str, f64)> {
        self.dimensions
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// 获取最小维度值
    pub fn min_dimension(&self) -> Option<(&str, f64)> {
        self.dimensions
            .iter()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(k, v)| (k.as_str(), *v))
    }

    /// 判断向量是否平衡（所有维度差异小于阈值）
    ///
    /// ## 参数
    ///
    /// - `threshold`: 最大允许差异（默认 0.2）
    ///
    /// ## 返回
    ///
    /// 如果 max(dimensions) - min(dimensions) <= threshold，返回 true
    pub fn is_balanced(&self, threshold: f64) -> bool {
        if self.dimensions.is_empty() {
            return true;
        }

        let max = self.max_dimension().map(|(_, v)| v).unwrap_or(0.0);
        let min = self.min_dimension().map(|(_, v)| v).unwrap_or(0.0);

        (max - min) <= threshold
    }
}

impl Default for StateVector {
    fn default() -> Self {
        Self::new()
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liangyyi::{Sixiang, Taiji, Liangyyi};

    #[test]
    fn test_new_vector() {
        let vec = StateVector::new();
        assert_eq!(vec.dimension_count(), 0);
        assert!(vec.dimension_names().is_empty());
    }

    #[test]
    fn test_standard_vector() {
        let vec = StateVector::standard();
        assert_eq!(vec.dimension_count(), 7);
        assert_eq!(vec.get("yin"), Some(0.5));
        assert_eq!(vec.get("yang"), Some(0.5));
        assert_eq!(vec.get("activity"), Some(0.5));
    }

    #[test]
    fn test_from_snapshot() {
        let taiji = Taiji::new();
        let liangyyi = Liangyyi::from_taiji(&taiji);
        let sixiang = Sixiang::from_liangyyi_and_activity(liangyyi, 0.5);
        let snapshot = StateSnapshot::from_current_state(taiji, liangyyi, sixiang);

        let vec = StateVector::from_snapshot(&snapshot);

        assert_eq!(vec.dimension_count(), 7);
        assert_eq!(vec.get("yin"), Some(snapshot.taiji.yin_energy));
        assert_eq!(vec.get("yang"), Some(snapshot.taiji.yang_energy));
        assert_eq!(vec.get("activity"), Some(snapshot.user_activity_level));
    }

    #[test]
    fn test_get_set_dimension() {
        let mut vec = StateVector::new();
        vec.set("test", 0.7);
        assert_eq!(vec.get("test"), Some(0.7));

        // 测试 clamp
        vec.set("test", 1.5);
        assert_eq!(vec.get("test"), Some(1.0));

        vec.set("test", -0.5);
        assert_eq!(vec.get("test"), Some(0.0));

        // 不存在的维度
        assert_eq!(vec.get("nonexistent"), None);
    }

    #[test]
    fn test_dimension_names() {
        let mut vec = StateVector::new();
        vec.set("c", 0.3);
        vec.set("a", 0.1);
        vec.set("b", 0.2);

        let names = vec.dimension_names();
        assert_eq!(names, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_distance_to() {
        let mut vec1 = StateVector::new();
        vec1.set("x", 0.0);
        vec1.set("y", 0.0);

        let mut vec2 = StateVector::new();
        vec2.set("x", 0.3);
        vec2.set("y", 0.4);

        let distance = vec1.distance_to(&vec2);
        assert!((distance - 0.5).abs() < 0.001); // 3-4-5 三角形

        // 测试没有共同维度
        let mut vec3 = StateVector::new();
        vec3.set("z", 0.5);
        assert_eq!(vec1.distance_to(&vec3), 0.0);
    }

    #[test]
    fn test_evolve_towards() {
        let mut vec = StateVector::new();
        vec.set("x", 0.0);

        let mut target = StateVector::new();
        target.set("x", 1.0);

        // 向目标演化 50%
        vec.evolve_towards(&target, 0.5);
        assert!((vec.get("x").unwrap() - 0.5).abs() < 0.001);

        // 再演化 50%（剩余距离的一半）
        vec.evolve_towards(&target, 0.5);
        assert!((vec.get("x").unwrap() - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_evolve_towards_clamp() {
        let mut vec = StateVector::new();
        vec.set("x", 0.9);

        let mut target = StateVector::new();
        target.set("x", 1.2); // 超出范围的目标

        vec.evolve_towards(&target, 1.0);
        assert_eq!(vec.get("x"), Some(1.0)); // 应该被 clamp 到 1.0
    }

    #[test]
    fn test_add() {
        let mut vec1 = StateVector::new();
        vec1.set("x", 0.3);
        vec1.set("y", 0.5);

        let mut vec2 = StateVector::new();
        vec2.set("x", 0.4);
        vec2.set("z", 0.2); // vec1 没有这个维度

        vec1.add(&vec2);
        assert_eq!(vec1.get("x"), Some(0.7));
        assert_eq!(vec1.get("y"), Some(0.5)); // 不变
        assert_eq!(vec1.get("z"), None); // 不添加新维度
    }

    #[test]
    fn test_scale() {
        let mut vec = StateVector::new();
        vec.set("x", 0.5);
        vec.set("y", 0.8);

        vec.scale(0.5);
        assert!((vec.get("x").unwrap() - 0.25).abs() < 0.001);
        assert!((vec.get("y").unwrap() - 0.4).abs() < 0.001);

        // 测试 clamp
        vec.scale(10.0);
        assert_eq!(vec.get("x"), Some(1.0));
        assert_eq!(vec.get("y"), Some(1.0));
    }

    #[test]
    fn test_norm() {
        let mut vec = StateVector::new();
        vec.set("x", 0.3);
        vec.set("y", 0.4);

        let norm = vec.norm();
        assert!((norm - 0.5).abs() < 0.001); // sqrt(0.09 + 0.16) = 0.5
    }

    #[test]
    fn test_mean() {
        let mut vec = StateVector::new();
        vec.set("a", 0.2);
        vec.set("b", 0.4);
        vec.set("c", 0.6);

        let mean = vec.mean();
        assert!((mean - 0.4).abs() < 0.001);

        // 空向量
        let empty = StateVector::new();
        assert_eq!(empty.mean(), 0.0);
    }

    #[test]
    fn test_max_min_dimension() {
        let mut vec = StateVector::new();
        vec.set("low", 0.2);
        vec.set("mid", 0.5);
        vec.set("high", 0.8);

        let (max_name, max_val) = vec.max_dimension().unwrap();
        assert_eq!(max_name, "high");
        assert_eq!(max_val, 0.8);

        let (min_name, min_val) = vec.min_dimension().unwrap();
        assert_eq!(min_name, "low");
        assert_eq!(min_val, 0.2);

        // 空向量
        let empty = StateVector::new();
        assert!(empty.max_dimension().is_none());
        assert!(empty.min_dimension().is_none());
    }

    #[test]
    fn test_is_balanced() {
        let mut vec = StateVector::new();
        vec.set("a", 0.5);
        vec.set("b", 0.5);
        vec.set("c", 0.5);

        assert!(vec.is_balanced(0.2)); // 完全平衡

        vec.set("c", 0.7);
        assert!(vec.is_balanced(0.3)); // 差异 0.2，在阈值内
        assert!(!vec.is_balanced(0.1)); // 超出阈值
    }

    #[test]
    fn test_clone_and_equality() {
        let mut vec1 = StateVector::new();
        vec1.set("x", 0.5);

        let vec2 = vec1.clone();
        assert_eq!(vec1, vec2);

        let mut vec3 = StateVector::new();
        vec3.set("x", 0.6);
        assert_ne!(vec1, vec3);
    }
}
