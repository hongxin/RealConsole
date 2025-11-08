//! 态势分析模块
//!
//! 基于易经哲学的科学态势分析系统，而非占卜算命。
//!
//! # 核心理念
//!
//! 本模块的本质是**态势测算分析系统**（Situation Analysis System），
//! 借助易经的智慧计算模式，对执行计划进行科学、理性的分析：
//!
//! - **复杂度分析**：步骤数量、阴阳平衡、步骤多样性
//! - **风险评估**：步骤顺序合理性、缺失步骤、潜在问题
//! - **时机分析**：计划是否适合当前执行
//! - **优化建议**：基于分析结果的实用建议
//!
//! # 设计哲学
//!
//! "易有三义"融入态势分析：
//! - **简易**：用简单的指标度量复杂的计划
//! - **变易**：识别计划可能的演化路径
//! - **不易**：遵循执行计划的基本规律

use crate::agent::decomposition::types::ExecutionPlan;
use super::step_analyzer::{StepAnalyzer, StepNature, YinYang};
use super::sequence_validator::{SequenceValidator, SequenceValidation};
use serde::{Deserialize, Serialize};

/// 态势分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SituationAnalysis {
    /// 复杂度等级
    pub complexity: ComplexityLevel,

    /// 风险等级
    pub risk: RiskLevel,

    /// 序列验证结果
    pub sequence_validation: SequenceValidation,

    /// 阴阳平衡情况
    pub yin_yang_balance: YinYangBalance,

    /// 步骤性质分布
    pub nature_distribution: NatureDistribution,

    /// 优化建议
    pub suggestions: Vec<String>,

    /// 总体评价
    pub overall_summary: String,

    /// 适合执行的时机建议
    pub timing_advice: String,
}

impl SituationAnalysis {
    /// 是否适合执行
    pub fn is_ready_to_execute(&self) -> bool {
        self.risk != RiskLevel::High && self.sequence_validation.is_valid
    }

    /// 获取复杂度图标
    pub fn complexity_icon(&self) -> &'static str {
        match self.complexity {
            ComplexityLevel::Simple => "🟢",
            ComplexityLevel::Moderate => "🟡",
            ComplexityLevel::Complex => "🟠",
            ComplexityLevel::VeryComplex => "🔴",
        }
    }

    /// 获取风险图标
    pub fn risk_icon(&self) -> &'static str {
        match self.risk {
            RiskLevel::Low => "🟢",
            RiskLevel::Medium => "🟡",
            RiskLevel::High => "🔴",
        }
    }
}

/// 复杂度等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    /// 简单（1-3步）
    Simple,
    /// 适中（4-6步）
    Moderate,
    /// 复杂（7-12步）
    Complex,
    /// 非常复杂（>12步）
    VeryComplex,
}

impl ComplexityLevel {
    pub fn from_step_count(count: usize) -> Self {
        match count {
            0..=3 => ComplexityLevel::Simple,
            4..=6 => ComplexityLevel::Moderate,
            7..=12 => ComplexityLevel::Complex,
            _ => ComplexityLevel::VeryComplex,
        }
    }

    pub fn chinese_name(&self) -> &'static str {
        match self {
            ComplexityLevel::Simple => "简单",
            ComplexityLevel::Moderate => "适中",
            ComplexityLevel::Complex => "复杂",
            ComplexityLevel::VeryComplex => "非常复杂",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ComplexityLevel::Simple => "步骤少，逻辑清晰，易于执行",
            ComplexityLevel::Moderate => "步骤适中，需要一定规划",
            ComplexityLevel::Complex => "步骤较多，需要仔细协调",
            ComplexityLevel::VeryComplex => "步骤繁多，建议拆分为子任务",
        }
    }
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 低风险：顺序合理，步骤完整
    Low,
    /// 中风险：有警告但不严重
    Medium,
    /// 高风险：有严重问题
    High,
}

impl RiskLevel {
    pub fn chinese_name(&self) -> &'static str {
        match self {
            RiskLevel::Low => "低风险",
            RiskLevel::Medium => "中风险",
            RiskLevel::High => "高风险",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RiskLevel::Low => "计划合理，可以直接执行",
            RiskLevel::Medium => "有可优化之处，建议review",
            RiskLevel::High => "存在严重问题，建议修正后执行",
        }
    }
}

/// 阴阳平衡情况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YinYangBalance {
    /// 阴爻数量（准备、决策步骤）
    pub yin_count: usize,

    /// 阳爻数量（执行、输出步骤）
    pub yang_count: usize,

    /// 阴爻比例
    pub yin_ratio: f64,

    /// 阳爻比例
    pub yang_ratio: f64,

    /// 是否平衡
    pub is_balanced: bool,

    /// 平衡评价
    pub balance_comment: String,
}

impl YinYangBalance {
    pub fn from_counts(yin: usize, yang: usize) -> Self {
        let total = yin + yang;
        let (yin_ratio, yang_ratio) = if total > 0 {
            (yin as f64 / total as f64, yang as f64 / total as f64)
        } else {
            (0.0, 0.0)
        };

        // 阴阳比例在 30%-70% 之间认为是平衡的
        let is_balanced = (0.3..=0.7).contains(&yin_ratio);

        let balance_comment = if total == 0 {
            "无步骤".to_string()
        } else if is_balanced {
            format!("阴阳平衡（阴 {}，阳 {}）", yin, yang)
        } else if yin_ratio > 0.7 {
            format!("阴爻过多（阴 {}，阳 {}），缺少执行动作", yin, yang)
        } else {
            format!("阳爻过多（阴 {}，阳 {}），缺少准备决策", yin, yang)
        };

        Self {
            yin_count: yin,
            yang_count: yang,
            yin_ratio,
            yang_ratio,
            is_balanced,
            balance_comment,
        }
    }
}

/// 步骤性质分布
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatureDistribution {
    /// 准备步骤数
    pub preparation_count: usize,
    /// 执行步骤数
    pub execution_count: usize,
    /// 决策步骤数
    pub decision_count: usize,
    /// 处理步骤数
    pub processing_count: usize,
    /// 输出步骤数
    pub finalization_count: usize,
    /// 清理步骤数
    pub cleanup_count: usize,

    /// 主要性质
    pub dominant_nature: Option<StepNature>,

    /// 分布描述
    pub distribution_summary: String,
}

impl NatureDistribution {
    pub fn from_plan(plan: &ExecutionPlan) -> Self {
        let mut counts = [0; 6];

        for step in &plan.steps {
            let nature = StepAnalyzer::analyze_nature(step);
            let index = match nature {
                StepNature::Preparation => 0,
                StepNature::Execution => 1,
                StepNature::Decision => 2,
                StepNature::Processing => 3,
                StepNature::Finalization => 4,
                StepNature::Cleanup => 5,
            };
            counts[index] += 1;
        }

        let preparation_count = counts[0];
        let execution_count = counts[1];
        let decision_count = counts[2];
        let processing_count = counts[3];
        let finalization_count = counts[4];
        let cleanup_count = counts[5];

        // 找出最多的性质
        let max_count = *counts.iter().max().unwrap_or(&0);
        let dominant_nature = if max_count > 0 {
            let index = counts.iter().position(|&c| c == max_count).unwrap();
            Some(match index {
                0 => StepNature::Preparation,
                1 => StepNature::Execution,
                2 => StepNature::Decision,
                3 => StepNature::Processing,
                4 => StepNature::Finalization,
                5 => StepNature::Cleanup,
                _ => unreachable!(),
            })
        } else {
            None
        };

        // 生成分布描述
        let distribution_summary = if plan.steps.is_empty() {
            "无步骤".to_string()
        } else {
            let mut parts = Vec::new();
            if preparation_count > 0 {
                parts.push(format!("准备{}步", preparation_count));
            }
            if execution_count > 0 {
                parts.push(format!("执行{}步", execution_count));
            }
            if decision_count > 0 {
                parts.push(format!("决策{}步", decision_count));
            }
            if processing_count > 0 {
                parts.push(format!("处理{}步", processing_count));
            }
            if finalization_count > 0 {
                parts.push(format!("输出{}步", finalization_count));
            }
            if cleanup_count > 0 {
                parts.push(format!("清理{}步", cleanup_count));
            }
            parts.join("，")
        };

        Self {
            preparation_count,
            execution_count,
            decision_count,
            processing_count,
            finalization_count,
            cleanup_count,
            dominant_nature,
            distribution_summary,
        }
    }
}

/// 态势分析器
pub struct SituationAnalyzer;

impl SituationAnalyzer {
    /// 分析执行计划的态势
    ///
    /// 返回科学的态势分析结果，而非算命占卜
    pub fn analyze(plan: &ExecutionPlan) -> SituationAnalysis {
        // 1. 复杂度分析
        let complexity = ComplexityLevel::from_step_count(plan.steps.len());

        // 2. 序列验证
        let sequence_validation = SequenceValidator::validate(plan);

        // 3. 风险评估
        let risk = Self::assess_risk(&sequence_validation);

        // 4. 阴阳平衡分析
        let (yin, yang) = StepAnalyzer::analyze_yin_yang_balance(&plan.steps);
        let yin_yang_balance = YinYangBalance::from_counts(yin, yang);

        // 5. 步骤性质分布
        let nature_distribution = NatureDistribution::from_plan(plan);

        // 6. 生成优化建议
        let suggestions = Self::generate_suggestions(
            &complexity,
            &risk,
            &sequence_validation,
            &yin_yang_balance,
        );

        // 7. 总体评价
        let overall_summary = Self::generate_summary(
            &complexity,
            &risk,
            &sequence_validation,
            &yin_yang_balance,
        );

        // 8. 时机建议
        let timing_advice = Self::generate_timing_advice(&complexity, &risk);

        SituationAnalysis {
            complexity,
            risk,
            sequence_validation,
            yin_yang_balance,
            nature_distribution,
            suggestions,
            overall_summary,
            timing_advice,
        }
    }

    /// 评估风险等级
    fn assess_risk(validation: &SequenceValidation) -> RiskLevel {
        if validation.has_critical_issues() {
            RiskLevel::High
        } else if validation.warning_count() > 0 {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        }
    }

    /// 生成优化建议
    fn generate_suggestions(
        complexity: &ComplexityLevel,
        risk: &RiskLevel,
        validation: &SequenceValidation,
        yin_yang: &YinYangBalance,
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // 基于复杂度的建议
        match complexity {
            ComplexityLevel::Simple => {
                suggestions.push("✅ 任务简单直接，建议直接执行".to_string());
            }
            ComplexityLevel::Moderate => {
                suggestions.push("📋 任务复杂度适中，建议检查步骤顺序后执行".to_string());
            }
            ComplexityLevel::Complex => {
                suggestions.push("⚠️  任务较为复杂，建议仔细review每个步骤".to_string());
            }
            ComplexityLevel::VeryComplex => {
                suggestions.push("🔴 任务过于复杂，强烈建议拆分为多个子任务".to_string());
            }
        }

        // 基于风险的建议
        if *risk == RiskLevel::High {
            suggestions.push("❌ 发现严重问题，建议修正后再执行".to_string());
        }

        // 基于序列验证的建议
        for suggestion in &validation.suggestions {
            suggestions.push(format!("💡 {}", suggestion.description));
        }

        // 基于阴阳平衡的建议
        if !yin_yang.is_balanced {
            if yin_yang.yin_ratio > 0.7 {
                suggestions.push("⚖️  建议增加执行类步骤，避免过度准备".to_string());
            } else {
                suggestions.push("⚖️  建议增加准备或决策步骤，避免盲目执行".to_string());
            }
        }

        suggestions
    }

    /// 生成总体评价
    fn generate_summary(
        complexity: &ComplexityLevel,
        risk: &RiskLevel,
        validation: &SequenceValidation,
        yin_yang: &YinYangBalance,
    ) -> String {
        let complexity_desc = format!(
            "复杂度：{} - {}",
            complexity.chinese_name(),
            complexity.description()
        );

        let risk_desc = format!("风险：{} - {}", risk.chinese_name(), risk.description());

        let sequence_desc = validation.overall_assessment.clone();

        let balance_desc = yin_yang.balance_comment.clone();

        format!(
            "{}\n{}\n{}\n{}",
            complexity_desc, risk_desc, sequence_desc, balance_desc
        )
    }

    /// 生成时机建议
    fn generate_timing_advice(complexity: &ComplexityLevel, risk: &RiskLevel) -> String {
        match (complexity, risk) {
            (ComplexityLevel::Simple, RiskLevel::Low) => {
                "适合立即执行".to_string()
            }
            (ComplexityLevel::Simple | ComplexityLevel::Moderate, RiskLevel::Medium) => {
                "适合在review后执行".to_string()
            }
            (_, RiskLevel::High) => {
                "不适合当前执行，建议修正问题后再执行".to_string()
            }
            (ComplexityLevel::Complex | ComplexityLevel::VeryComplex, _) => {
                "建议在充分准备、时间充裕时执行".to_string()
            }
            _ => {
                "可以执行，但建议先review".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::decomposition::types::ExecutionStep;

    #[test]
    fn test_simple_plan_analysis() {
        let plan = ExecutionPlan::new(
            "简单任务".to_string(),
            vec![
                ExecutionStep::new("读取文件".to_string(), "read_file".to_string(), 1.0),
                ExecutionStep::new("显示内容".to_string(), "display".to_string(), 0.5),
            ],
        );

        let analysis = SituationAnalyzer::analyze(&plan);

        assert_eq!(analysis.complexity, ComplexityLevel::Simple);
        assert_eq!(analysis.risk, RiskLevel::Low);
        assert!(analysis.is_ready_to_execute());
    }

    #[test]
    fn test_complex_plan_analysis() {
        // 创建一个非常复杂的计划
        let steps: Vec<ExecutionStep> = (0..15)
            .map(|i| ExecutionStep::new(format!("步骤 {}", i), "tool".to_string(), 1.0))
            .collect();

        let plan = ExecutionPlan::new("复杂任务".to_string(), steps);
        let analysis = SituationAnalyzer::analyze(&plan);

        assert_eq!(analysis.complexity, ComplexityLevel::VeryComplex);
    }

    #[test]
    fn test_risky_plan_analysis() {
        // 输出步骤在第一步 - 严重问题
        let plan = ExecutionPlan::new(
            "有风险的任务".to_string(),
            vec![
                ExecutionStep::new("显示结果".to_string(), "display".to_string(), 0.5),
                ExecutionStep::new("创建文件".to_string(), "create".to_string(), 1.0),
            ],
        );

        let analysis = SituationAnalyzer::analyze(&plan);

        assert_eq!(analysis.risk, RiskLevel::High);
        assert!(!analysis.is_ready_to_execute());
    }

    #[test]
    fn test_yin_yang_balance() {
        let plan = ExecutionPlan::new(
            "平衡任务".to_string(),
            vec![
                ExecutionStep::new("读取".to_string(), "read".to_string(), 1.0),    // Yin
                ExecutionStep::new("创建".to_string(), "create".to_string(), 1.0),  // Yang
                ExecutionStep::new("搜索".to_string(), "search".to_string(), 1.0),  // Yin
                ExecutionStep::new("输出".to_string(), "output".to_string(), 1.0),  // Yang
            ],
        );

        let analysis = SituationAnalyzer::analyze(&plan);

        assert_eq!(analysis.yin_yang_balance.yin_count, 2);
        assert_eq!(analysis.yin_yang_balance.yang_count, 2);
        assert!(analysis.yin_yang_balance.is_balanced);
    }

    #[test]
    fn test_nature_distribution() {
        let plan = ExecutionPlan::new(
            "测试分布".to_string(),
            vec![
                ExecutionStep::new("读取".to_string(), "read".to_string(), 1.0),
                ExecutionStep::new("创建".to_string(), "create".to_string(), 1.0),
                ExecutionStep::new("创建".to_string(), "create".to_string(), 1.0),
            ],
        );

        let analysis = SituationAnalyzer::analyze(&plan);

        assert_eq!(analysis.nature_distribution.preparation_count, 1);
        assert_eq!(analysis.nature_distribution.execution_count, 2);
        assert_eq!(
            analysis.nature_distribution.dominant_nature,
            Some(StepNature::Execution)
        );
    }

    #[test]
    fn test_empty_plan_analysis() {
        let plan = ExecutionPlan::new("空任务".to_string(), vec![]);
        let analysis = SituationAnalyzer::analyze(&plan);

        assert!(analysis.is_ready_to_execute());
        assert_eq!(analysis.complexity, ComplexityLevel::Simple);
    }

    #[test]
    fn test_timing_advice() {
        // 简单低风险计划
        let plan = ExecutionPlan::new(
            "简单任务".to_string(),
            vec![
                ExecutionStep::new("读取".to_string(), "read".to_string(), 1.0),
                ExecutionStep::new("输出".to_string(), "output".to_string(), 0.5),
            ],
        );

        let analysis = SituationAnalyzer::analyze(&plan);
        assert_eq!(analysis.timing_advice, "适合立即执行");
    }

    #[test]
    fn test_complexity_levels() {
        assert_eq!(ComplexityLevel::from_step_count(2), ComplexityLevel::Simple);
        assert_eq!(ComplexityLevel::from_step_count(5), ComplexityLevel::Moderate);
        assert_eq!(ComplexityLevel::from_step_count(10), ComplexityLevel::Complex);
        assert_eq!(
            ComplexityLevel::from_step_count(15),
            ComplexityLevel::VeryComplex
        );
    }
}
