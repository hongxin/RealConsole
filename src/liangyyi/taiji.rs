//! 太极：系统的统一状态
//!
//! 阴阳能量的连续表示，体现系统的本质状态

use chrono::{DateTime, Utc};

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
        }
    }

    /// 创建指定上下文的太极
    pub fn with_context(context: TaijiContext) -> Self {
        Self {
            yin_energy: 0.5,
            yang_energy: 0.5,
            timestamp: Utc::now(),
            context,
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
            Event::SystemIdle => {
                // 空闲时向平衡态衰减
                self.decay_to_balance(0.02);
            }
        }

        self.timestamp = Utc::now();
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
}
