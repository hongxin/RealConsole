//! 太极：系统的统一状态
//!
//! 阴阳能量的连续表示，体现系统的本质状态

use chrono::{DateTime, Duration, Utc};

/// 太极：系统的统一状态
///
/// 阴阳能量的连续表示，0.0-1.0
///
/// ## v1.9.6 扩展
///
/// 增加了上下文强度和持续时间，为多维状态空间打基础：
/// - `context_intensity`: 上下文的强度（0.0-1.0）
/// - `context_duration`: 上下文持续时间
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

    /// ✨ v1.9.6: 上下文强度（0.0-1.0）
    ///
    /// 表示当前上下文的强度：
    /// - 0.0: 弱上下文（刚开始、不确定）
    /// - 0.5: 中等上下文
    /// - 1.0: 强上下文（深度投入、确定性高）
    pub context_intensity: f64,

    /// ✨ v1.9.6: 上下文持续时间
    ///
    /// 记录当前上下文已经持续的时间
    /// 用于分析状态稳定性和转换周期
    pub context_duration: Duration,
}

/// 太极上下文
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
            context_intensity: 0.5,                    // ✨ v1.9.6: 默认中等强度
            context_duration: Duration::zero(),        // ✨ v1.9.6: 初始持续时间为0
        }
    }

    /// 创建指定上下文的太极
    pub fn with_context(context: TaijiContext) -> Self {
        Self {
            yin_energy: 0.5,
            yang_energy: 0.5,
            timestamp: Utc::now(),
            context,
            context_intensity: 0.5,
            context_duration: Duration::zero(),
        }
    }

    /// ✨ v1.9.6: 创建指定上下文和强度的太极
    pub fn with_context_and_intensity(context: TaijiContext, intensity: f64) -> Self {
        Self {
            yin_energy: 0.5,
            yang_energy: 0.5,
            timestamp: Utc::now(),
            context,
            context_intensity: intensity.clamp(0.0, 1.0),
            context_duration: Duration::zero(),
        }
    }

    /// ✨ v1.9.6: 切换上下文
    ///
    /// 切换到新的上下文时，重置持续时间和强度
    pub fn switch_context(&mut self, new_context: TaijiContext) {
        if self.context != new_context {
            self.context = new_context;
            self.context_intensity = 0.3;  // 新上下文从较低强度开始
            self.context_duration = Duration::zero();
        }
    }

    /// ✨ v1.9.6: 增强上下文
    ///
    /// 随着时间推移，上下文强度会自然增长
    pub fn enhance_context(&mut self, delta: f64) {
        self.context_intensity = (self.context_intensity + delta).clamp(0.0, 1.0);
    }

    /// ✨ v1.9.6: 更新上下文持续时间
    fn update_context_duration(&mut self, elapsed: Duration) {
        self.context_duration += elapsed;
    }

    /// 更新能量（基于事件）
    pub fn update_from_event(&mut self, event: &Event) {
        // ✨ v1.9.6: 计算时间差，更新上下文持续时间
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.timestamp);
        self.update_context_duration(elapsed);

        // ✨ v1.9.6: 根据持续时间增强上下文强度
        // 持续时间越长，上下文越强（但有上限）
        if elapsed.num_seconds() > 0 {
            let enhancement = (elapsed.num_seconds() as f64 / 60.0).min(0.1);  // 每分钟最多增强 0.1
            self.enhance_context(enhancement);
        }

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
            Event::SystemIdle => {
                // 空闲时向平衡态衰减
                self.decay_to_balance(0.02);
                // ✨ v1.9.6: 空闲时上下文强度也衰减
                if self.context_intensity > 0.1 {
                    self.context_intensity -= 0.02;
                }
            }
        }

        self.timestamp = now;
        self.normalize();
    }

    /// 归一化能量到 [0, 1]
    fn normalize(&mut self) {
        self.yin_energy = self.yin_energy.clamp(0.0, 1.0);
        self.yang_energy = self.yang_energy.clamp(0.0, 1.0);
    }

    /// 向平衡态衰减
    pub fn decay_to_balance(&mut self, rate: f64) {
        if self.yin_energy > 0.5 {
            self.yin_energy -= rate;
        } else {
            self.yin_energy += rate;
        }

        if self.yang_energy > 0.5 {
            self.yang_energy -= rate;
        } else {
            self.yang_energy += rate;
        }

        self.normalize();
    }

    /// 平衡度（0.0-1.0，1.0 表示完全平衡）
    pub fn balance(&self) -> f64 {
        1.0 - (self.yin_energy - self.yang_energy).abs()
    }

    /// 主导能量类型
    pub fn dominant_energy(&self) -> EnergyType {
        if self.yin_energy > self.yang_energy {
            EnergyType::Yin
        } else if self.yang_energy > self.yin_energy {
            EnergyType::Yang
        } else {
            EnergyType::Balanced
        }
    }

    /// 能量强度（0.0-1.0）
    pub fn intensity(&self) -> f64 {
        (self.yin_energy + self.yang_energy) / 2.0
    }
}

impl Default for Taiji {
    fn default() -> Self {
        Self::new()
    }
}

/// 能量类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyType {
    /// 阴主导
    Yin,
    /// 阳主导
    Yang,
    /// 平衡
    Balanced,
}

/// 系统事件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taiji_creation() {
        let taiji = Taiji::new();
        assert_eq!(taiji.yin_energy, 0.5);
        assert_eq!(taiji.yang_energy, 0.5);
        assert_eq!(taiji.balance(), 1.0);
    }

    #[test]
    fn test_update_from_read() {
        let mut taiji = Taiji::new();
        taiji.update_from_event(&Event::UserRead);

        // 读取增加阴能量
        assert!(taiji.yin_energy > 0.5);
        assert!(taiji.yang_energy < 0.5);
        assert_eq!(taiji.dominant_energy(), EnergyType::Yin);
    }

    #[test]
    fn test_update_from_execute() {
        let mut taiji = Taiji::new();
        taiji.update_from_event(&Event::UserExecute);

        // 执行增加阳能量
        assert!(taiji.yin_energy < 0.5);
        assert!(taiji.yang_energy > 0.5);
        assert_eq!(taiji.dominant_energy(), EnergyType::Yang);
    }

    #[test]
    fn test_decay_to_balance() {
        let mut taiji = Taiji::new();
        taiji.yin_energy = 0.8;
        taiji.yang_energy = 0.3;

        taiji.decay_to_balance(0.1);

        // 应该向 0.5 衰减
        assert!(taiji.yin_energy < 0.8);
        assert!(taiji.yang_energy > 0.3);
    }

    #[test]
    fn test_balance_calculation() {
        let mut taiji = Taiji::new();

        // 完全平衡
        taiji.yin_energy = 0.5;
        taiji.yang_energy = 0.5;
        assert_eq!(taiji.balance(), 1.0);

        // 极端不平衡
        taiji.yin_energy = 1.0;
        taiji.yang_energy = 0.0;
        assert_eq!(taiji.balance(), 0.0);

        // 中等不平衡
        taiji.yin_energy = 0.7;
        taiji.yang_energy = 0.3;
        assert!((taiji.balance() - 0.6).abs() < 0.01);
    }

    // ========== ✨ v1.9.6 新增测试 ==========

    #[test]
    fn test_context_intensity_initialization() {
        let taiji = Taiji::new();
        assert_eq!(taiji.context_intensity, 0.5);
        assert_eq!(taiji.context_duration, Duration::zero());
    }

    #[test]
    fn test_with_context_and_intensity() {
        let taiji = Taiji::with_context_and_intensity(
            TaijiContext::UserInteraction,
            0.8
        );

        assert_eq!(taiji.context, TaijiContext::UserInteraction);
        assert_eq!(taiji.context_intensity, 0.8);
        assert_eq!(taiji.context_duration, Duration::zero());
    }

    #[test]
    fn test_context_intensity_clamping() {
        // 测试超出范围的强度会被限制
        let taiji1 = Taiji::with_context_and_intensity(
            TaijiContext::DecisionMaking,
            1.5  // > 1.0
        );
        assert_eq!(taiji1.context_intensity, 1.0);

        let taiji2 = Taiji::with_context_and_intensity(
            TaijiContext::DecisionMaking,
            -0.5  // < 0.0
        );
        assert_eq!(taiji2.context_intensity, 0.0);
    }

    #[test]
    fn test_switch_context() {
        let mut taiji = Taiji::new();
        taiji.context_intensity = 0.9;
        taiji.context_duration = Duration::minutes(5);

        // 切换到新上下文
        taiji.switch_context(TaijiContext::UserInteraction);

        assert_eq!(taiji.context, TaijiContext::UserInteraction);
        assert_eq!(taiji.context_intensity, 0.3);  // 重置为低强度
        assert_eq!(taiji.context_duration, Duration::zero());  // 重置持续时间
    }

    #[test]
    fn test_switch_context_no_change() {
        let mut taiji = Taiji::new();
        let initial_intensity = taiji.context_intensity;
        let initial_duration = taiji.context_duration;

        // 切换到相同的上下文（不应该改变）
        taiji.switch_context(TaijiContext::SystemRunning);

        assert_eq!(taiji.context_intensity, initial_intensity);
        assert_eq!(taiji.context_duration, initial_duration);
    }

    #[test]
    fn test_enhance_context() {
        let mut taiji = Taiji::new();
        taiji.context_intensity = 0.5;

        taiji.enhance_context(0.2);
        assert_eq!(taiji.context_intensity, 0.7);

        // 测试上限
        taiji.enhance_context(0.5);
        assert_eq!(taiji.context_intensity, 1.0);  // 不会超过 1.0
    }

    #[test]
    fn test_enhance_context_negative() {
        let mut taiji = Taiji::new();
        taiji.context_intensity = 0.5;

        taiji.enhance_context(-0.3);  // 可以用负值减弱
        assert_eq!(taiji.context_intensity, 0.2);

        taiji.enhance_context(-0.5);
        assert_eq!(taiji.context_intensity, 0.0);  // 不会低于 0.0
    }

    #[test]
    fn test_context_duration_update() {
        use std::thread::sleep;
        use std::time::Duration as StdDuration;

        let mut taiji = Taiji::new();
        let initial_duration = taiji.context_duration;

        // 等待一小段时间
        sleep(StdDuration::from_millis(100));

        // 触发事件更新
        taiji.update_from_event(&Event::UserRead);

        // 持续时间应该增加
        assert!(taiji.context_duration > initial_duration);
    }

    #[test]
    fn test_idle_reduces_context_intensity() {
        let mut taiji = Taiji::new();
        taiji.context_intensity = 0.8;

        // 连续多次空闲事件
        for _ in 0..5 {
            taiji.update_from_event(&Event::SystemIdle);
        }

        // 强度应该降低
        assert!(taiji.context_intensity < 0.8);
    }
}
