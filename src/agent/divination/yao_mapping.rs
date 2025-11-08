//! 爻位映射
//!
//! 将 ExecutionPlan 的步骤映射到六爻位置
//!
//! # v1.36.1 重大更新
//!
//! 爻位不再是简单的序列位置，而是代表步骤的**语义角色**：
//! - 初爻：准备阶段，对应 StepNature::Preparation
//! - 二爻：初始执行，对应 StepNature::Execution
//! - 三爻：关键决策，对应 StepNature::Decision
//! - 四爻：深度处理，对应 StepNature::Processing
//! - 五爻：主要输出，对应 StepNature::Finalization
//! - 上爻：收尾清理，对应 StepNature::Cleanup

use serde::{Deserialize, Serialize};
use super::step_analyzer::StepNature;

/// 爻位
///
/// # 语义化定义（v1.36.1）
///
/// 每个爻位不是顺序编号，而是代表特定的语义角色：
/// - **初爻（Chu）**: 准备阶段 - "潜龙勿用"，打基础
/// - **二爻（Er）**: 初始执行 - "见龙在田"，开始行动
/// - **三爻（San）**: 关键决策 - "终日乾乾"，做出判断
/// - **四爻（Si）**: 深度处理 - "或跃在渊"，深度操作
/// - **五爻（Wu）**: 主要输出 - "飞龙在天"，产出结果
/// - **上爻（Shang）**: 收尾清理 - "亢龙有悔"，善始善终
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YaoPosition {
    /// 初爻：准备阶段
    ///
    /// 语义：读取、加载、初始化、验证前置条件
    /// 特征：潜龙勿用，打基础，阴爻
    Chu,

    /// 二爻：初始执行
    ///
    /// 语义：创建、写入、启动、开始主要操作
    /// 特征：见龙在田，初行动，阳爻
    Er,

    /// 三爻：关键决策
    ///
    /// 语义：搜索、查找、比对、条件判断
    /// 特征：终日乾乾，做决策，阴爻
    San,

    /// 四爻：深度处理
    ///
    /// 语义：转换、计算、排序、复杂业务逻辑
    /// 特征：或跃在渊，深处理，阳爻
    Si,

    /// 五爻：主要输出
    ///
    /// 语义：显示、返回、生成报告、输出结果
    /// 特征：飞龙在天，出结果，阳爻
    Wu,

    /// 上爻：收尾清理
    ///
    /// 语义：关闭、释放、保存、记录日志
    /// 特征：亢龙有悔，善收尾，阳爻
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

    /// **v1.36.1 新增**: 获取爻位期望的步骤性质
    ///
    /// 返回此爻位应该对应的步骤性质
    pub fn expected_nature(&self) -> StepNature {
        match self {
            YaoPosition::Chu => StepNature::Preparation,
            YaoPosition::Er => StepNature::Execution,
            YaoPosition::San => StepNature::Decision,
            YaoPosition::Si => StepNature::Processing,
            YaoPosition::Wu => StepNature::Finalization,
            YaoPosition::Shang => StepNature::Cleanup,
        }
    }

    /// **v1.36.1 新增**: 获取爻位的角色描述
    ///
    /// 返回符合易经哲学的角色特征描述
    pub fn role_description(&self) -> &'static str {
        match self {
            YaoPosition::Chu => "潜龙勿用，打基础。准备阶段，读取配置、加载数据、验证前置条件",
            YaoPosition::Er => "见龙在田，初行动。执行阶段，创建资源、写入数据、启动进程",
            YaoPosition::San => "终日乾乾，做决策。决策阶段，搜索信息、查找匹配、条件判断",
            YaoPosition::Si => "或跃在渊，深处理。处理阶段，转换格式、计算分析、排序过滤",
            YaoPosition::Wu => "飞龙在天，出结果。输出阶段，显示结果、生成报告、返回数据",
            YaoPosition::Shang => "亢龙有悔，善收尾。清理阶段，关闭连接、释放资源、保存日志",
        }
    }

    /// **v1.36.1 新增**: 从步骤性质创建爻位
    ///
    /// 根据步骤的语义性质确定其应该对应的爻位
    pub fn from_step_nature(nature: StepNature) -> Self {
        match nature {
            StepNature::Preparation => YaoPosition::Chu,
            StepNature::Execution => YaoPosition::Er,
            StepNature::Decision => YaoPosition::San,
            StepNature::Processing => YaoPosition::Si,
            StepNature::Finalization => YaoPosition::Wu,
            StepNature::Cleanup => YaoPosition::Shang,
        }
    }

    /// **v1.36.1 新增**: 检查步骤性质是否匹配爻位
    ///
    /// 返回 true 如果步骤性质符合此爻位的期望
    pub fn matches_nature(&self, nature: StepNature) -> bool {
        self.expected_nature() == nature
    }

    /// **v1.36.1 新增**: 获取语义化的爻位建议
    ///
    /// 根据爻位的角色返回更具体的建议
    pub fn semantic_advice(&self) -> &'static str {
        match self {
            YaoPosition::Chu => "此步骤处于准备阶段，建议确保所有前置条件就绪，数据完整可用",
            YaoPosition::Er => "此步骤处于执行阶段，建议稳健推进，注意错误处理",
            YaoPosition::San => "此步骤处于决策阶段，建议审慎判断，考虑多种可能",
            YaoPosition::Si => "此步骤处于处理阶段，建议注意数据转换的正确性和效率",
            YaoPosition::Wu => "此步骤处于输出阶段，建议确保结果格式正确、信息完整",
            YaoPosition::Shang => "此步骤处于清理阶段，建议确保资源正确释放，状态妥善保存",
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

    // v1.36.1 新增测试

    #[test]
    fn test_yao_expected_nature() {
        assert_eq!(YaoPosition::Chu.expected_nature(), StepNature::Preparation);
        assert_eq!(YaoPosition::Er.expected_nature(), StepNature::Execution);
        assert_eq!(YaoPosition::San.expected_nature(), StepNature::Decision);
        assert_eq!(YaoPosition::Si.expected_nature(), StepNature::Processing);
        assert_eq!(YaoPosition::Wu.expected_nature(), StepNature::Finalization);
        assert_eq!(YaoPosition::Shang.expected_nature(), StepNature::Cleanup);
    }

    #[test]
    fn test_yao_from_step_nature() {
        assert_eq!(YaoPosition::from_step_nature(StepNature::Preparation), YaoPosition::Chu);
        assert_eq!(YaoPosition::from_step_nature(StepNature::Execution), YaoPosition::Er);
        assert_eq!(YaoPosition::from_step_nature(StepNature::Decision), YaoPosition::San);
        assert_eq!(YaoPosition::from_step_nature(StepNature::Processing), YaoPosition::Si);
        assert_eq!(YaoPosition::from_step_nature(StepNature::Finalization), YaoPosition::Wu);
        assert_eq!(YaoPosition::from_step_nature(StepNature::Cleanup), YaoPosition::Shang);
    }

    #[test]
    fn test_yao_matches_nature() {
        assert!(YaoPosition::Chu.matches_nature(StepNature::Preparation));
        assert!(!YaoPosition::Chu.matches_nature(StepNature::Execution));

        assert!(YaoPosition::Wu.matches_nature(StepNature::Finalization));
        assert!(!YaoPosition::Wu.matches_nature(StepNature::Decision));
    }

    #[test]
    fn test_yao_role_description() {
        let desc = YaoPosition::Chu.role_description();
        assert!(desc.contains("潜龙勿用"));
        assert!(desc.contains("准备"));

        let desc = YaoPosition::Wu.role_description();
        assert!(desc.contains("飞龙在天"));
        assert!(desc.contains("输出"));
    }

    #[test]
    fn test_yao_semantic_advice() {
        let advice = YaoPosition::San.semantic_advice();
        assert!(advice.contains("决策"));

        let advice = YaoPosition::Shang.semantic_advice();
        assert!(advice.contains("清理"));
        assert!(advice.contains("资源"));
    }

    #[test]
    fn test_bidirectional_mapping() {
        // 测试 StepNature <-> YaoPosition 的双向映射
        for nature in [
            StepNature::Preparation,
            StepNature::Execution,
            StepNature::Decision,
            StepNature::Processing,
            StepNature::Finalization,
            StepNature::Cleanup,
        ] {
            let yao = YaoPosition::from_step_nature(nature);
            assert_eq!(yao.expected_nature(), nature);
        }
    }
}
