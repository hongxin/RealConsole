//! 软阈值函数库
//!
//! 基于 [docs/00-core/think.md](../../docs/00-core/think.md) 哲学：
//! 消除硬阈值带来的"断崖式"决策，使用平滑的连续函数。
//!
//! # 核心理念
//!
//! **硬阈值的问题**：
//! ```ignore
//! if score >= 0.7 {
//!     accept  // 0.71 → 100%
//! } else {
//!     reject  // 0.69 → 0%
//! }
//! ```
//! 在阈值边界处有明显的"跳变"，不符合自然规律。
//!
//! **软阈值的优势**：
//! ```ignore
//! let prob = sigmoid(score, threshold, softness);
//! // 0.69 → 48%
//! // 0.70 → 50%
//! // 0.71 → 52%
//! ```
//! 平滑过渡，更符合真实世界的"模糊性"。

use std::f64::consts::E;

/// Sigmoid 函数（平滑阶跃）
///
/// 用于将任意实数映射到 (0, 1) 区间，提供平滑的阈值决策。
///
/// # 数学定义
///
/// ```text
/// σ(x) = 1 / (1 + e^(-x))
/// ```
///
/// # 参数
/// - `x`: 输入值
///
/// # 返回值
/// - 输出在 (0, 1) 区间，x=0 时输出 0.5
///
/// # 示例
/// ```
/// use realconsole::utils::soft_threshold::sigmoid;
///
/// assert!((sigmoid(0.0) - 0.5).abs() < 1e-10);
/// assert!(sigmoid(5.0) > 0.99);   // 正数 → 接近 1
/// assert!(sigmoid(-5.0) < 0.01);  // 负数 → 接近 0
/// ```
pub fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + E.powf(-x))
}

/// 带中心和陡度的 Sigmoid 函数
///
/// 提供更灵活的控制：
/// - `center`: 阈值中心点（输出 0.5 的位置）
/// - `steepness`: 陡度（越大越陡峭，接近硬阈值；越小越平缓）
///
/// # 参数
/// - `x`: 输入值
/// - `center`: 中心点（默认 0.0）
/// - `steepness`: 陡度系数（默认 1.0）
///
/// # 返回值
/// - 输出在 (0, 1) 区间
///
/// # 示例
/// ```
/// use realconsole::utils::soft_threshold::sigmoid_with_params;
///
/// // 标准 sigmoid（中心 0，陡度 1）
/// assert!((sigmoid_with_params(0.0, 0.0, 1.0) - 0.5).abs() < 1e-10);
///
/// // 中心右移到 0.7
/// assert!((sigmoid_with_params(0.7, 0.7, 1.0) - 0.5).abs() < 1e-10);
///
/// // 陡度增加（更接近硬阈值）
/// let steep = sigmoid_with_params(0.71, 0.7, 10.0);
/// assert!(steep > 0.7);  // 快速上升
/// ```
pub fn sigmoid_with_params(x: f64, center: f64, steepness: f64) -> f64 {
    sigmoid((x - center) * steepness)
}

/// 接受概率函数（软阈值决策）
///
/// 用于替代硬阈值的软决策函数。
///
/// # 原理
///
/// **硬阈值**：
/// ```text
/// if score >= threshold { 100% }
/// else { 0% }
/// ```
///
/// **软阈值**：
/// ```text
/// probability = sigmoid((score - threshold) / softness)
/// ```
///
/// # 参数
/// - `score`: 评分（如置信度、匹配度等）
/// - `threshold`: 阈值中心点
/// - `softness`: 软化系数（控制过渡区宽度）
///   - `softness = 0.01`: 非常陡峭（接近硬阈值）
///   - `softness = 0.1`: 陡峭过渡
///   - `softness = 0.3`: 平缓过渡（推荐默认值）
///   - `softness = 0.5`: 非常平缓
///
/// # 返回值
/// - 接受概率 (0, 1)
///
/// # 示例
/// ```
/// use realconsole::utils::soft_threshold::acceptance_probability;
///
/// // 陡峭过渡（softness = 0.1）
/// let p1 = acceptance_probability(0.69, 0.7, 0.1);
/// let p2 = acceptance_probability(0.70, 0.7, 0.1);
/// let p3 = acceptance_probability(0.71, 0.7, 0.1);
///
/// assert!(p1 < 0.5 && p1 > 0.4);  // ~46%
/// assert!((p2 - 0.5).abs() < 0.01);  // ~50%
/// assert!(p3 > 0.5 && p3 < 0.6);  // ~54%
///
/// // 平缓过渡（softness = 0.3）
/// let p4 = acceptance_probability(0.5, 0.7, 0.3);
/// let p5 = acceptance_probability(0.7, 0.7, 0.3);
/// let p6 = acceptance_probability(0.9, 0.7, 0.3);
///
/// assert!(p4 > 0.2 && p4 < 0.3);  // ~25%
/// assert!((p5 - 0.5).abs() < 0.01);  // ~50%
/// assert!(p6 > 0.7 && p6 < 0.8);  // ~75%
/// ```
pub fn acceptance_probability(score: f64, threshold: f64, softness: f64) -> f64 {
    // softness 作为除数，值越大过渡越平缓
    sigmoid((score - threshold) / softness)
}

/// 清除概率函数（渐变清除策略）
///
/// 用于替代"超时立即清除"的硬策略。
///
/// # 原理
///
/// **硬超时**：
/// ```text
/// if idle_time >= timeout { clear(); }  // 突然清除
/// ```
///
/// **软超时**：
/// ```text
/// probability = smooth_clear_probability(idle_time, timeout);
/// if random() < probability { clear(); }  // 概率清除
/// ```
///
/// # 参数
/// - `idle_seconds`: 空闲时间（秒）
/// - `timeout`: 超时阈值（秒）
///
/// # 返回值
/// - 清除概率 (0, 1)
///   - `idle < timeout/2`: 接近 0（几乎不清除）
///   - `idle = timeout`: 0.5（50% 概率）
///   - `idle > timeout*2`: 接近 1（几乎必然清除）
///
/// # 示例
/// ```
/// use realconsole::utils::soft_threshold::smooth_clear_probability;
///
/// let timeout = 600.0;  // 10 分钟
///
/// // 前半段：几乎不清除
/// assert!(smooth_clear_probability(100.0, timeout) < 0.01);
/// assert!(smooth_clear_probability(300.0, timeout) < 0.5);
///
/// // 阈值附近：50% 概率
/// let p_threshold = smooth_clear_probability(timeout, timeout);
/// assert!((p_threshold - 0.5).abs() < 0.1);
///
/// // 远超阈值：接近 100%
/// assert!(smooth_clear_probability(1200.0, timeout) > 0.9);
/// ```
pub fn smooth_clear_probability(idle_seconds: f64, timeout: f64) -> f64 {
    if idle_seconds <= timeout / 2.0 {
        // 前半段：几乎不清除（二次函数，最大 0.1）
        let progress = idle_seconds / (timeout / 2.0);
        progress * progress * 0.1
    } else if idle_seconds <= timeout {
        // 过渡段（timeout/2 到 timeout）：从 0.1 线性增长到 0.5
        let progress = (idle_seconds - timeout / 2.0) / (timeout / 2.0);
        lerp(0.1, 0.5, progress)
    } else {
        // 后半段（timeout 之后）：从 0.5 使用 sigmoid 增长到接近 1.0
        let x = (idle_seconds - timeout) / timeout; // 归一化到 timeout 倍数
        0.5 + 0.48 * sigmoid(x * 3.0) // 从 0.5 平滑增长到 ~0.98
    }
}

/// 线性插值函数
///
/// 在两个值之间进行平滑插值。
///
/// # 参数
/// - `from`: 起始值
/// - `to`: 结束值
/// - `t`: 插值参数 (0.0 - 1.0)
///
/// # 返回值
/// - `t = 0.0`: 返回 `from`
/// - `t = 0.5`: 返回中间值
/// - `t = 1.0`: 返回 `to`
///
/// # 示例
/// ```
/// use realconsole::utils::soft_threshold::lerp;
///
/// assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
/// assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
/// assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);
/// ```
pub fn lerp(from: f64, to: f64, t: f64) -> f64 {
    from + (to - from) * t
}

/// 平滑阶跃函数（smoothstep）
///
/// 提供比线性插值更平滑的过渡，常用于动画和渐变。
///
/// # 参数
/// - `edge0`: 下边界
/// - `edge1`: 上边界
/// - `x`: 输入值
///
/// # 返回值
/// - `x <= edge0`: 返回 0.0
/// - `edge0 < x < edge1`: 平滑过渡（使用三次 Hermite 插值）
/// - `x >= edge1`: 返回 1.0
///
/// # 示例
/// ```
/// use realconsole::utils::soft_threshold::smoothstep;
///
/// assert_eq!(smoothstep(0.0, 1.0, -0.5), 0.0);
/// assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
/// assert!((smoothstep(0.0, 1.0, 0.5) - 0.5).abs() < 0.1);
/// assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
/// assert_eq!(smoothstep(0.0, 1.0, 1.5), 1.0);
/// ```
pub fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    // 裁剪到边界
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);

    // 三次 Hermite 插值: 3t² - 2t³
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid_basic() {
        // x = 0 应该输出 0.5
        let result = sigmoid(0.0);
        assert!((result - 0.5).abs() < 1e-10);

        // 正数应该 > 0.5
        assert!(sigmoid(1.0) > 0.5);
        assert!(sigmoid(5.0) > 0.99);

        // 负数应该 < 0.5
        assert!(sigmoid(-1.0) < 0.5);
        assert!(sigmoid(-5.0) < 0.01);
    }

    #[test]
    fn test_sigmoid_with_params() {
        // 中心点移到 0.7
        let result = sigmoid_with_params(0.7, 0.7, 1.0);
        assert!((result - 0.5).abs() < 1e-10);

        // 陡度测试
        let gentle = sigmoid_with_params(0.71, 0.7, 1.0);
        let steep = sigmoid_with_params(0.71, 0.7, 10.0);
        assert!(steep > gentle); // 陡度大 → 变化快
    }

    #[test]
    fn test_acceptance_probability() {
        let threshold = 0.7;
        let softness = 0.1;

        // 阈值处应该接近 0.5
        let p_threshold = acceptance_probability(threshold, threshold, softness);
        assert!((p_threshold - 0.5).abs() < 0.01);

        // 低于阈值
        let p_low = acceptance_probability(0.6, threshold, softness);
        assert!(p_low < 0.5);

        // 高于阈值
        let p_high = acceptance_probability(0.8, threshold, softness);
        assert!(p_high > 0.5);

        // 软化系数越大，过渡越平缓
        let p_gentle = acceptance_probability(0.5, threshold, 0.3);
        let p_steep = acceptance_probability(0.5, threshold, 0.1);
        assert!(p_gentle > p_steep); // 平缓时，低分也有更高概率
    }

    #[test]
    fn test_smooth_clear_probability() {
        let timeout = 600.0;

        // 前半段：几乎不清除
        assert!(smooth_clear_probability(100.0, timeout) < 0.1);
        assert!(smooth_clear_probability(300.0, timeout) < 0.5);

        // 阈值处：接近 50%
        let p_threshold = smooth_clear_probability(timeout, timeout);
        assert!((p_threshold - 0.5).abs() < 0.1);

        // 远超阈值：接近 100%
        assert!(smooth_clear_probability(1200.0, timeout) > 0.9);

        // 单调递增
        let p1 = smooth_clear_probability(300.0, timeout);
        let p2 = smooth_clear_probability(600.0, timeout);
        let p3 = smooth_clear_probability(900.0, timeout);
        assert!(p1 < p2);
        assert!(p2 < p3);
    }

    #[test]
    fn test_lerp() {
        assert_eq!(lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(lerp(0.0, 10.0, 1.0), 10.0);

        // 负数范围
        assert_eq!(lerp(-5.0, 5.0, 0.5), 0.0);
    }

    #[test]
    fn test_smoothstep() {
        // 边界外
        assert_eq!(smoothstep(0.0, 1.0, -0.5), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.5), 1.0);

        // 边界上
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);

        // 中点附近
        let mid = smoothstep(0.0, 1.0, 0.5);
        assert!((mid - 0.5).abs() < 0.1);

        // 平滑性测试：导数应该在边界处为 0
        let eps = 0.01;
        let left = smoothstep(0.0, 1.0, eps);
        let right = smoothstep(0.0, 1.0, 1.0 - eps);
        assert!(left < 0.1);      // 起始处平缓
        assert!(right > 0.9);     // 结束处平缓
    }
}
