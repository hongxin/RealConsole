//! 四象：老阴、少阳、少阴、老阳
//!
//! 两仪分化为四象，体现阴阳动静的四种状态

use super::types::Liangyyi;

/// 四象：老阴、少阳、少阴、老阳
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sixiang {
    /// 老阴 ▅▅ ▅▅ ▅▅ (极静)
    ///
    /// 特征：深度思考、数据沉淀、知识固化
    /// 例子：长时间未操作、深度学习、规划设计
    LaoYin,

    /// 少阳 ▅▅▅▅▅ ▅▅ ▅▅ (动中有静)
    ///
    /// 特征：探索尝试、初次使用、实验性操作
    /// 例子：首次运行命令、探索新功能、查看文档
    ShaoYang,

    /// 少阴 ▅▅ ▅▅ ▅▅▅▅▅ (静中有动)
    ///
    /// 特征：准备阶段、蓄势待发、确认意图
    /// 例子：思考命令、检查状态、分析问题
    ShaoYin,

    /// 老阳 ▅▅▅▅▅ ▅▅▅▅▅ ▅▅▅▅▅ (极动)
    ///
    /// 特征：高频操作、连续执行、快速迭代
    /// 例子：批量处理、紧急修复、自动化脚本
    LaoYang,
}

impl Sixiang {
    /// 从两仪和活动水平推导
    ///
    /// # Arguments
    /// * `liangyyi` - 两仪状态（太阴/太阳）
    /// * `activity_level` - 活动水平 (0.0-1.0)
    ///   - < 0.3: 低活动
    ///   - 0.3-0.7: 中等活动
    ///   - > 0.7: 高活动
    pub fn from_liangyyi_and_activity(liangyyi: Liangyyi, activity_level: f64) -> Self {
        match liangyyi {
            Liangyyi::Taiyin => {
                // 阴主导
                if activity_level < 0.3 {
                    Sixiang::LaoYin // 极静
                } else {
                    Sixiang::ShaoYin // 静中有动
                }
            }
            Liangyyi::Taiyang => {
                // 阳主导
                if activity_level > 0.7 {
                    Sixiang::LaoYang // 极动
                } else {
                    Sixiang::ShaoYang // 动中有静
                }
            }
        }
    }

    /// 自然转换（按周期）
    ///
    /// 遵循"静极生动、动极生静"的规律
    pub fn transition(&self) -> Self {
        match self {
            Sixiang::LaoYin => Sixiang::ShaoYang,  // 静极生动
            Sixiang::ShaoYang => Sixiang::LaoYang, // 动渐增
            Sixiang::LaoYang => Sixiang::ShaoYin,  // 动极生静
            Sixiang::ShaoYin => Sixiang::LaoYin,   // 静渐增
        }
    }

    /// 描述文本
    pub fn description(&self) -> &'static str {
        match self {
            Sixiang::LaoYin => "极静·深思·沉淀",
            Sixiang::ShaoYang => "探索·尝试·初发",
            Sixiang::ShaoYin => "蓄势·准备·确认",
            Sixiang::LaoYang => "极动·快速·连续",
        }
    }

    /// 符号表示
    pub fn symbol(&self) -> &'static str {
        match self {
            Sixiang::LaoYin => "▅▅ ▅▅ ▅▅",
            Sixiang::ShaoYang => "▅▅▅▅▅ ▅▅ ▅▅",
            Sixiang::ShaoYin => "▅▅ ▅▅ ▅▅▅▅▅",
            Sixiang::LaoYang => "▅▅▅▅▅ ▅▅▅▅▅ ▅▅▅▅▅",
        }
    }

    /// 是否为阴象（老阴、少阴）
    pub fn is_yin(&self) -> bool {
        matches!(self, Sixiang::LaoYin | Sixiang::ShaoYin)
    }

    /// 是否为阳象（老阳、少阳）
    pub fn is_yang(&self) -> bool {
        matches!(self, Sixiang::LaoYang | Sixiang::ShaoYang)
    }

    /// 是否为老（老阴、老阳）
    pub fn is_lao(&self) -> bool {
        matches!(self, Sixiang::LaoYin | Sixiang::LaoYang)
    }

    /// 是否为少（少阴、少阳）
    pub fn is_shao(&self) -> bool {
        matches!(self, Sixiang::ShaoYin | Sixiang::ShaoYang)
    }

    /// 活动等级 (1-4)
    pub fn activity_level(&self) -> u8 {
        match self {
            Sixiang::LaoYin => 1,    // 最静
            Sixiang::ShaoYin => 2,   // 较静
            Sixiang::ShaoYang => 3,  // 较动
            Sixiang::LaoYang => 4,   // 最动
        }
    }

    /// 所有四象
    pub fn all() -> [Self; 4] {
        [
            Sixiang::LaoYin,
            Sixiang::ShaoYang,
            Sixiang::LaoYang,
            Sixiang::ShaoYin,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_liangyyi_taiyin_low_activity() {
        let sixiang = Sixiang::from_liangyyi_and_activity(Liangyyi::Taiyin, 0.2);
        assert_eq!(sixiang, Sixiang::LaoYin);
        assert!(sixiang.is_yin());
        assert!(sixiang.is_lao());
    }

    #[test]
    fn test_from_liangyyi_taiyin_mid_activity() {
        let sixiang = Sixiang::from_liangyyi_and_activity(Liangyyi::Taiyin, 0.5);
        assert_eq!(sixiang, Sixiang::ShaoYin);
        assert!(sixiang.is_yin());
        assert!(sixiang.is_shao());
    }

    #[test]
    fn test_from_liangyyi_taiyang_mid_activity() {
        let sixiang = Sixiang::from_liangyyi_and_activity(Liangyyi::Taiyang, 0.5);
        assert_eq!(sixiang, Sixiang::ShaoYang);
        assert!(sixiang.is_yang());
        assert!(sixiang.is_shao());
    }

    #[test]
    fn test_from_liangyyi_taiyang_high_activity() {
        let sixiang = Sixiang::from_liangyyi_and_activity(Liangyyi::Taiyang, 0.8);
        assert_eq!(sixiang, Sixiang::LaoYang);
        assert!(sixiang.is_yang());
        assert!(sixiang.is_lao());
    }

    #[test]
    fn test_transition_cycle() {
        let lao_yin = Sixiang::LaoYin;
        let shao_yang = lao_yin.transition();
        let lao_yang = shao_yang.transition();
        let shao_yin = lao_yang.transition();
        let back_to_lao_yin = shao_yin.transition();

        assert_eq!(shao_yang, Sixiang::ShaoYang);
        assert_eq!(lao_yang, Sixiang::LaoYang);
        assert_eq!(shao_yin, Sixiang::ShaoYin);
        assert_eq!(back_to_lao_yin, Sixiang::LaoYin);
    }

    #[test]
    fn test_activity_level() {
        assert_eq!(Sixiang::LaoYin.activity_level(), 1);
        assert_eq!(Sixiang::ShaoYin.activity_level(), 2);
        assert_eq!(Sixiang::ShaoYang.activity_level(), 3);
        assert_eq!(Sixiang::LaoYang.activity_level(), 4);
    }

    #[test]
    fn test_symbol() {
        assert_eq!(Sixiang::LaoYin.symbol(), "▅▅ ▅▅ ▅▅");
        assert_eq!(Sixiang::ShaoYang.symbol(), "▅▅▅▅▅ ▅▅ ▅▅");
        assert_eq!(Sixiang::ShaoYin.symbol(), "▅▅ ▅▅ ▅▅▅▅▅");
        assert_eq!(Sixiang::LaoYang.symbol(), "▅▅▅▅▅ ▅▅▅▅▅ ▅▅▅▅▅");
    }
}
