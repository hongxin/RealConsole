//! 卦辞生成器
//!
//! 根据 ExecutionPlan 动态生成卦辞

use super::hexagram::Hexagram;

pub struct JudgementGenerator;

impl JudgementGenerator {
    /// 生成动态卦辞
    pub fn generate(
        hexagram: &Hexagram,
        plan: &crate::agent::decomposition::ExecutionPlan,
    ) -> String {
        let complexity = Self::assess_complexity(plan);
        let advice = Self::generate_advice(plan);
        let timing = Self::assess_timing(&plan.steps.len());

        format!(
            "{}\n\n💭 AI 理解：\n{}\n\n📊 任务评估：\n此任务{}，{}步骤，{}。\n\n🔮 占卜建议：\n{}",
            hexagram.full_description(),
            plan.understanding,
            complexity,
            plan.steps.len(),
            timing,
            advice
        )
    }

    /// 评估任务复杂度
    fn assess_complexity(plan: &crate::agent::decomposition::ExecutionPlan) -> &'static str {
        match plan.steps.len() {
            1 => "简单直接",
            2..=3 => "较为简单",
            4..=6 => "需循序而进",
            7..=10 => "较为复杂",
            _ => "错综复杂",
        }
    }

    /// 生成执行建议
    fn generate_advice(plan: &crate::agent::decomposition::ExecutionPlan) -> &'static str {
        // 基于步骤数和复杂度生成建议
        let step_count = plan.steps.len();
        let estimated_time = plan.total_estimated_time;

        if step_count == 1 {
            "可直接执行，一气呵成"
        } else if step_count <= 2 {
            "步骤简单，顺势而为"
        } else if step_count <= 4 {
            "稳步推进，循序渐进"
        } else if step_count <= 6 {
            "审慎对待，步步为营"
        } else if estimated_time > 10.0 {
            "任务繁重，当分段处理，勿求速成"
        } else {
            "事务繁杂，当逐一验证，确保无误"
        }
    }

    /// 评估时机
    fn assess_timing(step_count: &usize) -> &'static str {
        match step_count {
            1 => "当机立断",
            2..=3 => "时机正好",
            4..=6 => "需耐心等待",
            _ => "宜分步实施",
        }
    }

    /// 根据卦象生成更具体的建议
    #[allow(dead_code)]
    pub fn generate_hexagram_specific_advice(hexagram: &Hexagram) -> String {
        use super::trigram::Trigram::*;

        // 根据上下卦的组合生成建议
        match (&hexagram.upper, &hexagram.lower) {
            (Qian, Qian) => {
                "乾卦：天行健，君子以自强不息。任务执行需要刚健之气，持续推进，不可懈怠。".to_string()
            }
            (Kun, Kun) => {
                "坤卦：地势坤，君子以厚德载物。任务需要稳重承载，踏实执行，厚积薄发。".to_string()
            }
            (Kan, Li) => {
                "既济卦：事已成就，但需保持警惕。执行顺利时更要谨慎，防止乐极生悲。".to_string()
            }
            (Li, Kan) => {
                "未济卦：事未完成，当循序渐进。执行需要耐心，一步一个脚印，不可急躁。".to_string()
            }
            (Qian, Kun) => {
                "否卦：天地不交，当守正待时。执行可能遇阻，需要耐心等待时机成熟。".to_string()
            }
            (Kun, Qian) => {
                "泰卦：天地交泰，通达顺畅。执行将会顺利，但仍需保持谦逊谨慎。".to_string()
            }
            (Li, _) => {
                format!("上卦为离（火），主明照。任务中需要清晰的认知和洞察，{}应当仔细检查。",
                    hexagram.lower.name())
            }
            (Kan, _) => {
                format!("上卦为坎（水），主流动。任务可能遇到险阻，{}需要灵活应对。",
                    hexagram.lower.name())
            }
            (_, Li) => {
                format!("下卦为离（火），主明照。任务基础需要扎实，{}要确保起点正确。",
                    hexagram.upper.name())
            }
            (_, Kan) => {
                format!("下卦为坎（水），主流动。任务开始可能有阻碍，{}需要稳步推进。",
                    hexagram.upper.name())
            }
            _ => {
                format!("{}{}相配，当顺应卦象之理，稳步执行。",
                    hexagram.upper.name(),
                    hexagram.lower.name())
            }
        }
    }

    /// 生成完整的占卜报告
    pub fn generate_full_report(
        hexagram: &Hexagram,
        plan: &crate::agent::decomposition::ExecutionPlan,
        step_mappings: &[super::yao_mapping::StepYaoMapping],
    ) -> String {
        let basic_judgement = Self::generate(hexagram, plan);
        let hexagram_advice = Self::generate_hexagram_specific_advice(hexagram);

        let mut report = format!("{}\n\n🎯 卦象解析：\n{}\n", basic_judgement, hexagram_advice);

        // 如果步骤数<=6，添加爻位分析
        if step_mappings.len() <= 6 {
            report.push_str("\n📍 步骤爻位分析：\n");
            for mapping in step_mappings {
                report.push_str(&format!(
                    "  {}. {} - {} ({})\n     {}\n",
                    mapping.step_index + 1,
                    mapping.yao.name(),
                    mapping.trigram.name(),
                    mapping.trigram.symbol(),
                    mapping.line_statement
                ));
            }
        }

        report
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
                "test_tool".to_string(),
                1.0,
            ))
            .collect();

        ExecutionPlan::new("测试计划".to_string(), steps)
    }

    #[test]
    fn test_complexity_assessment() {
        assert_eq!(JudgementGenerator::assess_complexity(&create_test_plan(1)), "简单直接");
        assert_eq!(JudgementGenerator::assess_complexity(&create_test_plan(3)), "较为简单");
        assert_eq!(JudgementGenerator::assess_complexity(&create_test_plan(5)), "需循序而进");
        assert_eq!(JudgementGenerator::assess_complexity(&create_test_plan(8)), "较为复杂");
    }

    #[test]
    fn test_advice_generation() {
        let advice1 = JudgementGenerator::generate_advice(&create_test_plan(1));
        assert!(advice1.contains("直接"));

        let advice5 = JudgementGenerator::generate_advice(&create_test_plan(5));
        assert!(!advice5.is_empty());
    }

    #[test]
    fn test_timing_assessment() {
        assert_eq!(JudgementGenerator::assess_timing(&1), "当机立断");
        assert_eq!(JudgementGenerator::assess_timing(&3), "时机正好");
        assert_eq!(JudgementGenerator::assess_timing(&10), "宜分步实施");
    }

    #[test]
    fn test_judgement_generation() {
        use super::super::trigram::Trigram;

        let plan = create_test_plan(3);
        let hexagram = super::Hexagram::new(Trigram::Qian, Trigram::Kun);

        let judgement = JudgementGenerator::generate(&hexagram, &plan);

        assert!(judgement.contains("AI 理解"));
        assert!(judgement.contains("任务评估"));
        assert!(judgement.contains("占卜建议"));
    }

    #[test]
    fn test_hexagram_specific_advice() {
        use super::super::trigram::Trigram;

        let hexagram = super::Hexagram::new(Trigram::Qian, Trigram::Qian);
        let advice = JudgementGenerator::generate_hexagram_specific_advice(&hexagram);

        assert!(advice.contains("乾卦"));
        assert!(!advice.is_empty());
    }
}
