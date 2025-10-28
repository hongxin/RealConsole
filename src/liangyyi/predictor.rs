//! # StatePredictor - 状态预测器
//!
//! ## 设计理念
//!
//! StatePredictor 基于历史 StateVector 序列预测未来状态，体现"易"（变化）的哲学：
//! - **观往知来**：通过历史观测预测未来趋势
//! - **动态演化**：状态不是静止的，而是持续变化的
//! - **多维预测**：对每个维度独立预测，支持精细分析
//!
//! ## v1.12.0 核心特性
//!
//! - **线性趋势预测**：基于最近观测计算趋势斜率，外推到未来
//! - **指数加权移动平均**：近期观测权重更高，适合平滑预测
//! - **趋势分析**：识别上升/下降/稳定趋势
//! - **置信度评估**：基于历史波动性计算预测可信度
//!
//! ## 使用场景
//!
//! ```rust
//! use realconsole::liangyyi::{StatePredictor, StateVector};
//!
//! // 1. 创建预测器
//! let mut predictor = StatePredictor::new(10); // 保留最近 10 个观测
//!
//! // 2. 添加观测值
//! for snapshot in history {
//!     let vec = StateVector::from_snapshot(&snapshot);
//!     predictor.add_observation(vec);
//! }
//!
//! // 3. 预测未来
//! if let Some(predicted) = predictor.predict_linear(1) {
//!     println!("预测下一个状态: {:?}", predicted);
//! }
//!
//! // 4. 趋势分析
//! let trends = predictor.analyze_trends();
//! ```

use std::collections::VecDeque;

use super::state_vector::StateVector;

/// 状态预测器
///
/// 基于历史 StateVector 序列预测未来状态
#[derive(Debug, Clone)]
pub struct StatePredictor {
    /// 历史状态序列（按时间顺序，最新的在队尾）
    history: VecDeque<StateVector>,

    /// 最大历史长度（超过后自动删除最旧的）
    max_history: usize,
}

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    /// 上升趋势
    Rising,
    /// 下降趋势
    Falling,
    /// 稳定（变化很小）
    Stable,
}

/// 维度趋势分析结果
#[derive(Debug, Clone)]
pub struct DimensionTrend {
    /// 维度名称
    pub dimension: String,

    /// 趋势方向
    pub direction: TrendDirection,

    /// 趋势强度（0.0-1.0，越大表示趋势越明显）
    pub strength: f64,

    /// 平均变化率（每步的变化量）
    pub change_rate: f64,
}

impl StatePredictor {
    // ========== 构造函数 ==========

    /// 创建新的状态预测器
    ///
    /// ## 参数
    ///
    /// - `max_history`: 最大历史长度（建议 5-20）
    ///   - 太小：预测不准确
    ///   - 太大：对最新趋势不敏感
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            max_history: max_history.max(2), // 至少需要 2 个点
        }
    }

    /// 使用默认参数创建（历史长度 = 10）
    pub fn default() -> Self {
        Self::new(10)
    }

    // ========== 数据管理 ==========

    /// 添加新观测值
    ///
    /// 如果历史已满，自动删除最旧的观测
    pub fn add_observation(&mut self, state: StateVector) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(state);
    }

    /// 清空所有历史
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// 获取历史长度
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// 检查是否有足够的数据进行预测（至少 2 个观测）
    pub fn can_predict(&self) -> bool {
        self.history.len() >= 2
    }

    // ========== 预测方法 ==========

    /// 线性趋势预测
    ///
    /// ## 算法
    ///
    /// 对每个维度：
    /// 1. 计算最近 N 个观测的平均变化率（斜率）
    /// 2. 从最后一个观测外推 `steps` 步
    ///
    /// ## 参数
    ///
    /// - `steps`: 预测多少步（1 = 下一个状态）
    ///
    /// ## 返回
    ///
    /// - `Some(StateVector)`: 预测的状态
    /// - `None`: 历史不足（需要至少 2 个观测）
    pub fn predict_linear(&self, steps: usize) -> Option<StateVector> {
        if !self.can_predict() || steps == 0 {
            return None;
        }

        let last = self.history.back()?;
        let mut predicted = last.clone();

        // 对每个维度独立预测
        for dim in last.dimension_names() {
            // 计算平均变化率
            let change_rate = self.calculate_change_rate(&dim);

            // 外推 steps 步
            if let Some(current_value) = predicted.get(&dim) {
                let new_value = current_value + change_rate * steps as f64;
                predicted.set(&dim, new_value);
            }
        }

        Some(predicted)
    }

    /// 指数加权移动平均预测
    ///
    /// ## 算法
    ///
    /// EWMA: v_t = α * x_t + (1-α) * v_{t-1}
    /// - 近期观测权重更高（α 越大，权重越高）
    /// - 适合平滑预测，减少噪声影响
    ///
    /// ## 参数
    ///
    /// - `steps`: 预测多少步（通常为 1）
    /// - `alpha`: 平滑系数（0.0-1.0，默认 0.3）
    ///   - 0.1-0.2: 非常平滑，适合长期趋势
    ///   - 0.3-0.5: 平衡，推荐
    ///   - 0.6-0.9: 快速响应，适合短期波动
    ///
    /// ## 返回
    ///
    /// - `Some(StateVector)`: 预测的状态
    /// - `None`: 历史不足
    pub fn predict_ewma(&self, steps: usize, alpha: f64) -> Option<StateVector> {
        if !self.can_predict() || steps == 0 {
            return None;
        }

        let alpha = alpha.clamp(0.0, 1.0);
        let last = self.history.back()?;
        let mut predicted = last.clone();

        // 对每个维度独立计算 EWMA
        for dim in last.dimension_names() {
            let ewma = self.calculate_ewma(&dim, alpha)?;

            // 简单外推：假设未来继续按当前 EWMA 方向移动
            let last_value = last.get(&dim)?;
            let delta = ewma - last_value;
            let new_value = last_value + delta * steps as f64;

            predicted.set(&dim, new_value);
        }

        Some(predicted)
    }

    // ========== 趋势分析 ==========

    /// 分析所有维度的趋势
    ///
    /// ## 返回
    ///
    /// 每个维度的趋势分析结果
    pub fn analyze_trends(&self) -> Vec<DimensionTrend> {
        if !self.can_predict() {
            return vec![];
        }

        let last = match self.history.back() {
            Some(v) => v,
            None => return vec![],
        };

        let mut trends = Vec::new();

        for dim in last.dimension_names() {
            let change_rate = self.calculate_change_rate(&dim);
            let volatility = self.calculate_volatility(&dim);

            // 判断趋势方向
            let direction = if change_rate.abs() < 0.01 {
                TrendDirection::Stable
            } else if change_rate > 0.0 {
                TrendDirection::Rising
            } else {
                TrendDirection::Falling
            };

            // 趋势强度：变化率相对于波动性
            let strength = if volatility > 0.0 {
                (change_rate.abs() / volatility).min(1.0)
            } else {
                0.0
            };

            trends.push(DimensionTrend {
                dimension: dim.clone(),
                direction,
                strength,
                change_rate,
            });
        }

        trends
    }

    /// 检测异常（预测值与实际值差异过大）
    ///
    /// ## 参数
    ///
    /// - `actual`: 实际观测到的状态
    /// - `threshold`: 异常阈值（距离超过此值视为异常）
    ///
    /// ## 返回
    ///
    /// - `true`: 检测到异常
    /// - `false`: 正常
    pub fn detect_anomaly(&self, actual: &StateVector, threshold: f64) -> bool {
        if let Some(predicted) = self.predict_linear(1) {
            let distance = predicted.distance_to(actual);
            distance > threshold
        } else {
            false // 无法预测，不判断为异常
        }
    }

    // ========== 内部辅助方法 ==========

    /// 计算指定维度的平均变化率
    ///
    /// 变化率 = Σ(v[i+1] - v[i]) / (N - 1)
    fn calculate_change_rate(&self, dimension: &str) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }

        let mut total_change = 0.0;
        let mut count = 0;

        for i in 1..self.history.len() {
            if let (Some(prev), Some(curr)) = (
                self.history[i - 1].get(dimension),
                self.history[i].get(dimension),
            ) {
                total_change += curr - prev;
                count += 1;
            }
        }

        if count > 0 {
            total_change / count as f64
        } else {
            0.0
        }
    }

    /// 计算指定维度的 EWMA
    fn calculate_ewma(&self, dimension: &str, alpha: f64) -> Option<f64> {
        if self.history.is_empty() {
            return None;
        }

        let mut ewma = self.history[0].get(dimension)?;

        for state in self.history.iter().skip(1) {
            if let Some(value) = state.get(dimension) {
                ewma = alpha * value + (1.0 - alpha) * ewma;
            }
        }

        Some(ewma)
    }

    /// 计算指定维度的波动性（标准差）
    fn calculate_volatility(&self, dimension: &str) -> f64 {
        if self.history.len() < 2 {
            return 0.0;
        }

        // 收集所有值
        let values: Vec<f64> = self
            .history
            .iter()
            .filter_map(|state| state.get(dimension))
            .collect();

        if values.is_empty() {
            return 0.0;
        }

        // 计算均值
        let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

        // 计算标准差
        let variance: f64 = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
            / values.len() as f64;

        variance.sqrt()
    }
}

// ========== 单元测试 ==========

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_vector(value: f64) -> StateVector {
        let mut vec = StateVector::new();
        vec.set("x", value);
        vec.set("y", value * 2.0);
        vec
    }

    #[test]
    fn test_new_predictor() {
        let predictor = StatePredictor::new(5);
        assert_eq!(predictor.history_len(), 0);
        assert!(!predictor.can_predict());
    }

    #[test]
    fn test_add_observation() {
        let mut predictor = StatePredictor::new(3);

        predictor.add_observation(create_test_vector(0.1));
        assert_eq!(predictor.history_len(), 1);

        predictor.add_observation(create_test_vector(0.2));
        assert_eq!(predictor.history_len(), 2);
        assert!(predictor.can_predict());

        // 超过最大长度，应该删除最旧的
        predictor.add_observation(create_test_vector(0.3));
        predictor.add_observation(create_test_vector(0.4));
        assert_eq!(predictor.history_len(), 3);

        // 最旧的 0.1 应该被删除
        assert_eq!(predictor.history[0].get("x"), Some(0.2));
    }

    #[test]
    fn test_predict_linear_rising() {
        let mut predictor = StatePredictor::new(5);

        // 添加上升趋势：0.1, 0.2, 0.3
        predictor.add_observation(create_test_vector(0.1));
        predictor.add_observation(create_test_vector(0.2));
        predictor.add_observation(create_test_vector(0.3));

        let predicted = predictor.predict_linear(1).unwrap();

        // 趋势斜率 ≈ 0.1，预测下一个应该 ≈ 0.4
        assert!((predicted.get("x").unwrap() - 0.4).abs() < 0.01);
        assert!((predicted.get("y").unwrap() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_predict_linear_multi_step() {
        let mut predictor = StatePredictor::new(5);

        predictor.add_observation(create_test_vector(0.0));
        predictor.add_observation(create_test_vector(0.1));

        // 预测 3 步
        let predicted = predictor.predict_linear(3).unwrap();

        // 趋势斜率 = 0.1，3步后应该 ≈ 0.4
        assert!((predicted.get("x").unwrap() - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_predict_ewma() {
        let mut predictor = StatePredictor::new(5);

        predictor.add_observation(create_test_vector(0.5));
        predictor.add_observation(create_test_vector(0.6));
        predictor.add_observation(create_test_vector(0.7));

        let predicted = predictor.predict_ewma(1, 0.3).unwrap();

        // EWMA 应该产生合理的预测值
        let x = predicted.get("x").unwrap();
        // EWMA 平滑处理，可能低于最后值（0.7），但应该在合理范围内 [0.5, 0.8]
        assert!(x >= 0.5 && x <= 0.8, "Expected x in [0.5, 0.8], got {}", x);
    }

    #[test]
    fn test_analyze_trends_rising() {
        let mut predictor = StatePredictor::new(5);

        predictor.add_observation(create_test_vector(0.2));
        predictor.add_observation(create_test_vector(0.4));
        predictor.add_observation(create_test_vector(0.6));

        let trends = predictor.analyze_trends();

        assert_eq!(trends.len(), 2);

        let x_trend = trends.iter().find(|t| t.dimension == "x").unwrap();
        assert_eq!(x_trend.direction, TrendDirection::Rising);
        assert!(x_trend.change_rate > 0.0);
    }

    #[test]
    fn test_analyze_trends_stable() {
        let mut predictor = StatePredictor::new(5);

        // 稳定状态
        predictor.add_observation(create_test_vector(0.5));
        predictor.add_observation(create_test_vector(0.5));
        predictor.add_observation(create_test_vector(0.5));

        let trends = predictor.analyze_trends();

        let x_trend = trends.iter().find(|t| t.dimension == "x").unwrap();
        assert_eq!(x_trend.direction, TrendDirection::Stable);
        assert!(x_trend.change_rate.abs() < 0.01);
    }

    #[test]
    fn test_detect_anomaly() {
        let mut predictor = StatePredictor::new(5);

        // 稳定趋势
        predictor.add_observation(create_test_vector(0.5));
        predictor.add_observation(create_test_vector(0.5));
        predictor.add_observation(create_test_vector(0.5));

        // 正常观测
        let normal = create_test_vector(0.5);
        assert!(!predictor.detect_anomaly(&normal, 0.1));

        // 异常观测（突然跳到 0.9）
        let anomaly = create_test_vector(0.9);
        assert!(predictor.detect_anomaly(&anomaly, 0.1));
    }

    #[test]
    fn test_clear() {
        let mut predictor = StatePredictor::new(5);

        predictor.add_observation(create_test_vector(0.1));
        predictor.add_observation(create_test_vector(0.2));
        assert_eq!(predictor.history_len(), 2);

        predictor.clear();
        assert_eq!(predictor.history_len(), 0);
        assert!(!predictor.can_predict());
    }

    #[test]
    fn test_insufficient_data() {
        let mut predictor = StatePredictor::new(5);

        // 只有一个观测，无法预测
        predictor.add_observation(create_test_vector(0.5));

        assert!(!predictor.can_predict());
        assert!(predictor.predict_linear(1).is_none());
        assert!(predictor.predict_ewma(1, 0.3).is_none());
    }
}
