//! 爻位映射
//!
//! 将 ExecutionPlan 的步骤映射到六爻位置

use serde::{Deserialize, Serialize};

/// 爻位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YaoPosition {
    /// 初爻（第1步）
    Chu,
    /// 二爻（第2步）
    Er,
    /// 三爻（第3步）
    San,
    /// 四爻（第4步）
    Si,
    /// 五爻（第5步）
    Wu,
    /// 上爻（第6步）
    Shang,
}

impl YaoPosition {
    /// 获取爻位名称
    pub fn name(&self) -> &'static str {
        match self {
            YaoPosition::Chu => "初爻",
            YaoPosition::Er => "二爻",
            YaoPosition::San => "三爻",
            YaoPosition::Si => "四爻",
            YaoPosition::Wu => "五爻",
            YaoPosition::Shang => "上爻",
        }
    }

    /// 获取爻辞（简化版）
    pub fn line_statement(&self) -> &'static str {
        match self {
            YaoPosition::Chu => "初爻，事之初始，当顺势而为",
            YaoPosition::Er => "二爻，事之发展，当稳步推进",
            YaoPosition::San => "三爻，事之转折，当审慎决断",
            YaoPosition::Si => "四爻，事之高升，当谦和处世",
            YaoPosition::Wu => "五爻，事之鼎盛，当守正持中",
            YaoPosition::Shang => "上爻，事之终结，当功成身退",
        }
    }

    /// 从步骤索引创建爻位
    pub fn from_step_index(index: usize) -> Self {
        match index % 6 {
            0 => YaoPosition::Chu,
            1 => YaoPosition::Er,
            2 => YaoPosition::San,
            3 => YaoPosition::Si,
            4 => YaoPosition::Wu,
            5 => YaoPosition::Shang,
            _ => unreachable!(),
        }
    }

    /// 获取爻位的序号（从1开始）
    pub fn order(&self) -> usize {
        match self {
            YaoPosition::Chu => 1,
            YaoPosition::Er => 2,
            YaoPosition::San => 3,
            YaoPosition::Si => 4,
            YaoPosition::Wu => 5,
            YaoPosition::Shang => 6,
        }
    }

    /// 获取爻位的建议
    pub fn advice(&self) -> &'static str {
        match self {
            YaoPosition::Chu => "开始阶段，保持谨慎，稳扎稳打",
            YaoPosition::Er => "发展阶段，持续努力，积累力量",
            YaoPosition::San => "转折阶段，思虑周全，把握时机",
            YaoPosition::Si => "上升阶段，保持谦逊，警惕风险",
            YaoPosition::Wu => "鼎盛阶段，守正不阿，发挥优势",
            YaoPosition::Shang => "终结阶段，急流勇退，善始善终",
        }
    }
}

/// 步骤到爻位的映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepYaoMapping {
    /// 步骤索引（从 0 开始）
    pub step_index: usize,
    /// 爻位
    pub yao: YaoPosition,
    /// 对应的卦象
    pub trigram: super::trigram::Trigram,
    /// 爻辞
    pub line_statement: String,
    /// 建议
    pub advice: String,
}

impl StepYaoMapping {
    /// 创建新的映射
    pub fn new(
        step_index: usize,
        yao: YaoPosition,
        trigram: super::trigram::Trigram,
    ) -> Self {
        Self {
            step_index,
            yao,
            trigram,
            line_statement: yao.line_statement().to_string(),
            advice: yao.advice().to_string(),
        }
    }

    /// 获取完整描述
    pub fn full_description(&self) -> String {
        format!(
            "【{}】{} ({})\n{}\n建议：{}",
            self.yao.name(),
            self.trigram.name(),
            self.trigram.symbol(),
            self.line_statement,
            self.advice
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yao_position_from_index() {
        assert_eq!(YaoPosition::from_step_index(0), YaoPosition::Chu);
        assert_eq!(YaoPosition::from_step_index(1), YaoPosition::Er);
        assert_eq!(YaoPosition::from_step_index(2), YaoPosition::San);
        assert_eq!(YaoPosition::from_step_index(3), YaoPosition::Si);
        assert_eq!(YaoPosition::from_step_index(4), YaoPosition::Wu);
        assert_eq!(YaoPosition::from_step_index(5), YaoPosition::Shang);

        // 测试循环
        assert_eq!(YaoPosition::from_step_index(6), YaoPosition::Chu);
        assert_eq!(YaoPosition::from_step_index(7), YaoPosition::Er);
    }

    #[test]
    fn test_yao_position_names() {
        assert_eq!(YaoPosition::Chu.name(), "初爻");
        assert_eq!(YaoPosition::Wu.name(), "五爻");
        assert_eq!(YaoPosition::Shang.name(), "上爻");
    }

    #[test]
    fn test_yao_position_statements() {
        let statement = YaoPosition::Chu.line_statement();
        assert!(statement.contains("初始"));

        let statement = YaoPosition::Wu.line_statement();
        assert!(statement.contains("鼎盛"));
    }

    #[test]
    fn test_yao_position_order() {
        assert_eq!(YaoPosition::Chu.order(), 1);
        assert_eq!(YaoPosition::Er.order(), 2);
        assert_eq!(YaoPosition::Shang.order(), 6);
    }

    #[test]
    fn test_step_yao_mapping() {
        use super::super::trigram::Trigram;

        let mapping = StepYaoMapping::new(
            0,
            YaoPosition::Chu,
            Trigram::Qian,
        );

        assert_eq!(mapping.step_index, 0);
        assert_eq!(mapping.yao, YaoPosition::Chu);
        assert!(!mapping.line_statement.is_empty());
        assert!(!mapping.advice.is_empty());
    }

    #[test]
    fn test_mapping_full_description() {
        use super::super::trigram::Trigram;

        let mapping = StepYaoMapping::new(
            0,
            YaoPosition::Chu,
            Trigram::Li,
        );

        let desc = mapping.full_description();
        assert!(desc.contains("初爻"));
        assert!(desc.contains("离"));
        assert!(desc.contains("☲"));
    }
}
