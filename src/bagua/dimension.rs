//! 八卦维度定义
//!
//! 基于易经八卦的八维记忆空间映射

use serde::{Deserialize, Serialize};
use std::fmt;

/// 八卦记忆维度
///
/// 每个维度对应一个卦象，代表不同类型的记忆
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BaguaDimension {
    /// 乾 ☰ (天): 意图目标 - Goal Memory
    ///
    /// 代表系统和用户的最高目标、长期意图
    Qian,

    /// 坤 ☷ (地): 原始数据 - Raw Memory
    ///
    /// 被动承载，记录所有原始交互数据
    Kun,

    /// 震 ☳ (雷): 触发行动 - Action Memory
    ///
    /// 主动触发，记录系统执行的所有操作
    Zhen,

    /// 巽 ☴ (风): 趋势变化 - Trend Memory
    ///
    /// 渐进演化，记录长期变化和趋势
    Xun,

    /// 坎 ☵ (水): 深层模式 - Pattern Memory ⭐
    ///
    /// 向下流动，沉淀深层规律和隐性知识
    /// **离坎循环核心**
    Kan,

    /// 离 ☲ (火): 显性知识 - Knowledge Memory ⭐
    ///
    /// 向上照亮，输出显性知识和主动建议
    /// **离坎循环核心**
    Li,

    /// 艮 ☶ (山): 状态检查 - Checkpoint Memory
    ///
    /// 稳定边界，记录关键时刻的系统快照
    Gen,

    /// 兑 ☱ (泽): 交互反馈 - Feedback Memory
    ///
    /// 愉悦交流，记录用户反馈和系统响应
    Dui,
}

impl BaguaDimension {
    /// 获取卦象符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Qian => "☰",
            Self::Kun => "☷",
            Self::Zhen => "☳",
            Self::Xun => "☴",
            Self::Kan => "☵",
            Self::Li => "☲",
            Self::Gen => "☶",
            Self::Dui => "☱",
        }
    }

    /// 获取中文名称
    pub fn name_zh(&self) -> &'static str {
        match self {
            Self::Qian => "乾",
            Self::Kun => "坤",
            Self::Zhen => "震",
            Self::Xun => "巽",
            Self::Kan => "坎",
            Self::Li => "离",
            Self::Gen => "艮",
            Self::Dui => "兑",
        }
    }

    /// 获取英文名称
    pub fn name_en(&self) -> &'static str {
        match self {
            Self::Qian => "Qian",
            Self::Kun => "Kun",
            Self::Zhen => "Zhen",
            Self::Xun => "Xun",
            Self::Kan => "Kan",
            Self::Li => "Li",
            Self::Gen => "Gen",
            Self::Dui => "Dui",
        }
    }

    /// 获取描述
    pub fn description(&self) -> &'static str {
        match self {
            Self::Qian => "意图目标 - 系统和用户的最高目标",
            Self::Kun => "原始数据 - 被动承载所有交互数据",
            Self::Zhen => "触发行动 - 主动触发的系统操作",
            Self::Xun => "趋势变化 - 渐进演化的长期趋势",
            Self::Kan => "深层模式 - 沉淀的隐性规律",
            Self::Li => "显性知识 - 照亮的显性建议",
            Self::Gen => "状态检查 - 关键时刻的系统快照",
            Self::Dui => "交互反馈 - 用户反馈和响应",
        }
    }

    /// 获取对偶维度（阴阳对立）
    pub fn opposite(&self) -> Self {
        match self {
            Self::Qian => Self::Kun,  // 天地对立
            Self::Kun => Self::Qian,
            Self::Zhen => Self::Xun,  // 动静对立
            Self::Xun => Self::Zhen,
            Self::Kan => Self::Li,    // 内外对立 ⭐ 离坎核心
            Self::Li => Self::Kan,
            Self::Gen => Self::Dui,   // 守放对立
            Self::Dui => Self::Gen,
        }
    }

    /// 是否为离坎核心维度
    pub fn is_likan_core(&self) -> bool {
        matches!(self, Self::Kan | Self::Li)
    }

    /// 获取所有维度
    pub fn all() -> [Self; 8] {
        [
            Self::Qian,
            Self::Kun,
            Self::Zhen,
            Self::Xun,
            Self::Kan,
            Self::Li,
            Self::Gen,
            Self::Dui,
        ]
    }
}

impl fmt::Display for BaguaDimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} ({})", self.symbol(), self.name_zh(), self.name_en())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opposite() {
        assert_eq!(BaguaDimension::Qian.opposite(), BaguaDimension::Kun);
        assert_eq!(BaguaDimension::Kan.opposite(), BaguaDimension::Li);
        assert_eq!(BaguaDimension::Zhen.opposite(), BaguaDimension::Xun);
        assert_eq!(BaguaDimension::Gen.opposite(), BaguaDimension::Dui);
    }

    #[test]
    fn test_likan_core() {
        assert!(BaguaDimension::Kan.is_likan_core());
        assert!(BaguaDimension::Li.is_likan_core());
        assert!(!BaguaDimension::Qian.is_likan_core());
    }

    #[test]
    fn test_display() {
        let dim = BaguaDimension::Kan;
        assert_eq!(dim.to_string(), "☵ 坎 (Kan)");
    }
}
