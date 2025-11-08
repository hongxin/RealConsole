//! 意图占卜引擎
//!
//! 统一入口，为 ExecutionPlan 生成完整的占卜结果

use super::*;
use serde::{Deserialize, Serialize};

/// 完整的占卜结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DivinationResult {
    /// 卦象
    pub hexagram: hexagram::Hexagram,
    /// 演算步骤（用于动画）
    pub yarrow_steps: Vec<yarrow_stalks::YarrowStep>,
    /// 步骤到爻位的映射
    pub step_mappings: Vec<yao_mapping::StepYaoMapping>,
    /// 生成的卦辞
    pub full_judgement: String,
}

/// 占卜引擎
pub struct DivinationEngine;

impl DivinationEngine {
    /// 为 ExecutionPlan 生成占卜结果
    ///
    /// # 参数
    /// - `plan`: 执行计划
    ///
    /// # 返回
    /// 完整的占卜结果，包含卦象、演算步骤、爻位映射和卦辞
    pub fn divine(plan: &crate::agent::decomposition::ExecutionPlan) -> DivinationResult {
        // 1. 确定下卦（基于第一个步骤）
        let lower = if !plan.steps.is_empty() {
            trigram::Trigram::from_tool_name(&plan.steps[0].tool)
        } else {
            trigram::Trigram::Kun
        };

        // 2. 确定上卦（基于最后一个步骤）
        let upper = if plan.steps.len() > 1 {
            trigram::Trigram::from_tool_name(&plan.steps[plan.steps.len() - 1].tool)
        } else {
            trigram::Trigram::Qian
        };

        // 3. 生成卦象
        let hexagram = hexagram::Hexagram::new(upper, lower);

        // 4. 模拟演算过程
        let yarrow_steps = yarrow_stalks::YarrowStalksSimulator::simulate(plan.steps.len());

        // 5. 映射爻位
        let step_mappings = plan
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let yao = yao_mapping::YaoPosition::from_step_index(i);
                let trigram = trigram::Trigram::from_tool_name(&step.tool);
                yao_mapping::StepYaoMapping::new(i, yao, trigram)
            })
            .collect();

        // 6. 生成完整卦辞
        let full_judgement = judgement_generator::JudgementGenerator::generate(&hexagram, plan);

        DivinationResult {
            hexagram,
            yarrow_steps,
            step_mappings,
            full_judgement,
        }
    }

    /// 生成完整占卜报告（包含爻位分析）
    pub fn divine_with_full_report(
        plan: &crate::agent::decomposition::ExecutionPlan,
    ) -> DivinationResult {
        let mut result = Self::divine(plan);

        // 如果步骤数适合详细分析（<=6），生成完整报告
        if plan.steps.len() <= 6 {
            result.full_judgement = judgement_generator::JudgementGenerator::generate_full_report(
                &result.hexagram,
                plan,
                &result.step_mappings,
            );
        }

        result
    }

    /// 快速占卜（仅生成卦象和基本卦辞，不生成动画数据）
    pub fn divine_quick(plan: &crate::agent::decomposition::ExecutionPlan) -> DivinationResult {
        let lower = if !plan.steps.is_empty() {
            trigram::Trigram::from_tool_name(&plan.steps[0].tool)
        } else {
            trigram::Trigram::Kun
        };

        let upper = if plan.steps.len() > 1 {
            trigram::Trigram::from_tool_name(&plan.steps[plan.steps.len() - 1].tool)
        } else {
            trigram::Trigram::Qian
        };

        let hexagram = hexagram::Hexagram::new(upper, lower);

        // 快速模式：空的演算步骤和爻位映射
        DivinationResult {
            hexagram: hexagram.clone(),
            yarrow_steps: vec![],
            step_mappings: vec![],
            full_judgement: judgement_generator::JudgementGenerator::generate(&hexagram, plan),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::decomposition::{ExecutionPlan, ExecutionStep};

    fn create_test_plan(step_count: usize) -> ExecutionPlan {
        let steps: Vec<ExecutionStep> = (0..step_count)
            .map(|i| ExecutionStep::new(
                format!("步骤 {}", i + 1),
                format!("test_tool_{}", i),
                1.0,
            ))
            .collect();

        ExecutionPlan::new("测试意图拆解".to_string(), steps)
    }

    #[test]
    fn test_divine_basic() {
        let plan = create_test_plan(3);
        let result = DivinationEngine::divine(&plan);

        // 验证卦象生成
        assert!(!result.hexagram.name.is_empty());
        assert!(!result.hexagram.judgement.is_empty());

        // 验证演算步骤
        assert!(!result.yarrow_steps.is_empty());

        // 验证爻位映射
        assert_eq!(result.step_mappings.len(), 3);

        // 验证卦辞
        assert!(!result.full_judgement.is_empty());
        assert!(result.full_judgement.contains("AI 理解"));
    }

    #[test]
    fn test_divine_single_step() {
        let plan = create_test_plan(1);
        let result = DivinationEngine::divine(&plan);

        assert_eq!(result.step_mappings.len(), 1);
        assert!(!result.hexagram.name.is_empty());
    }

    #[test]
    fn test_divine_with_tool_names() {
        let mut steps = vec![
            ExecutionStep::new("列出文件".to_string(), "list_directory".to_string(), 1.0),
            ExecutionStep::new("搜索内容".to_string(), "search_text".to_string(), 1.0),
        ];

        let plan = ExecutionPlan::new("查找文件内容".to_string(), steps);
        let result = DivinationEngine::divine(&plan);

        // 验证卦象基于工具名称生成
        assert!(!result.hexagram.name.is_empty());

        // 验证爻位映射包含正确的卦象
        assert_eq!(result.step_mappings.len(), 2);
    }

    #[test]
    fn test_divine_empty_plan() {
        let plan = ExecutionPlan::new("空计划".to_string(), vec![]);
        let result = DivinationEngine::divine(&plan);

        // 应该使用默认卦象
        assert!(!result.hexagram.name.is_empty());
        assert_eq!(result.step_mappings.len(), 0);
    }

    #[test]
    fn test_divine_quick() {
        let plan = create_test_plan(5);
        let result = DivinationEngine::divine_quick(&plan);

        // 快速模式不生成动画数据
        assert!(result.yarrow_steps.is_empty());
        assert!(result.step_mappings.is_empty());

        // 但应该有卦象和卦辞
        assert!(!result.hexagram.name.is_empty());
        assert!(!result.full_judgement.is_empty());
    }

    #[test]
    fn test_divine_with_full_report() {
        let plan = create_test_plan(4);
        let result = DivinationEngine::divine_with_full_report(&plan);

        // 应该包含爻位分析
        assert!(result.full_judgement.contains("步骤爻位分析") || result.full_judgement.contains("任务评估"));
    }

    #[test]
    fn test_yarrow_steps_generation() {
        let plan = create_test_plan(3);
        let result = DivinationEngine::divine(&plan);

        // 演算步骤应该包含：大衍 + 操作步骤 + 成卦
        assert!(result.yarrow_steps.len() > 0);

        // 第一步应该是大衍
        assert_eq!(result.yarrow_steps[0].operation, "大衍");

        // 最后一步应该是成卦
        assert_eq!(result.yarrow_steps.last().unwrap().operation, "成卦");
    }

    #[test]
    fn test_step_mappings_correctness() {
        let plan = create_test_plan(6);
        let result = DivinationEngine::divine(&plan);

        // 验证每个步骤都有对应的爻位
        for (i, mapping) in result.step_mappings.iter().enumerate() {
            assert_eq!(mapping.step_index, i);
            assert!(!mapping.line_statement.is_empty());
            assert!(!mapping.advice.is_empty());
        }
    }
}
