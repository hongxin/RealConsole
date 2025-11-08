//! 步骤序列验证模块
//!
//! 基于易经六爻哲学，验证执行计划的步骤顺序是否合理。
//!
//! # 核心理念
//!
//! 好的执行计划应该遵循自然的"爻序"逻辑：
//! 1. **初爻先行**：准备工作在前（读取、加载、验证）
//! 2. **上爻善后**：清理工作在后（关闭、释放、保存）
//! 3. **阴阳平衡**：准备型和执行型步骤应该平衡
//! 4. **循序渐进**：从简单到复杂，从准备到执行到清理
//!
//! # 验证策略
//!
//! - **Critical**：严重问题，如输出步骤出现在最前面
//! - **Warning**：可改进，如缺少准备或清理步骤
//! - **Info**：信息提示，如步骤顺序可优化

use crate::agent::decomposition::types::ExecutionPlan;
use super::step_analyzer::{StepAnalyzer, StepNature};
use super::yao_mapping::YaoPosition;
use serde::{Deserialize, Serialize};

/// 序列验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceValidation {
    /// 是否总体合理
    pub is_valid: bool,

    /// 发现的问题
    pub issues: Vec<SequenceIssue>,

    /// 优化建议
    pub suggestions: Vec<SequenceSuggestion>,

    /// 总体评价
    pub overall_assessment: String,
}

impl SequenceValidation {
    /// 检查是否有严重问题
    pub fn has_critical_issues(&self) -> bool {
        self.issues.iter().any(|issue| issue.severity == IssueSeverity::Critical)
    }

    /// 获取严重问题数量
    pub fn critical_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.severity == IssueSeverity::Critical).count()
    }

    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.issues.iter().filter(|issue| issue.severity == IssueSeverity::Warning).count()
    }
}

/// 序列问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceIssue {
    /// 问题严重程度
    pub severity: IssueSeverity,

    /// 问题类型
    pub issue_type: IssueType,

    /// 涉及的步骤索引
    pub step_indices: Vec<usize>,

    /// 问题描述
    pub description: String,

    /// 详细说明
    pub details: Option<String>,
}

/// 问题严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    /// 严重问题：违背基本逻辑
    Critical,
    /// 警告：可能导致问题
    Warning,
    /// 信息：可以优化
    Info,
}

impl IssueSeverity {
    pub fn symbol(&self) -> &'static str {
        match self {
            IssueSeverity::Critical => "🔴",
            IssueSeverity::Warning => "🟡",
            IssueSeverity::Info => "🔵",
        }
    }

    pub fn chinese_name(&self) -> &'static str {
        match self {
            IssueSeverity::Critical => "严重",
            IssueSeverity::Warning => "警告",
            IssueSeverity::Info => "提示",
        }
    }
}

/// 问题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    /// 步骤顺序颠倒（如输出在准备之前）
    InvertedOrder,
    /// 缺少关键步骤类型
    MissingStepType,
    /// 步骤位置不理想
    SuboptimalPosition,
    /// 阴阳严重失衡
    YinYangImbalance,
    /// 复杂度过高
    OverlyComplex,
}

/// 序列优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceSuggestion {
    /// 建议类型
    pub suggestion_type: SuggestionType,

    /// 建议描述
    pub description: String,

    /// 具体操作
    pub action: Option<String>,
}

/// 建议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuggestionType {
    /// 重新排序步骤
    Reorder,
    /// 拆分步骤
    Split,
    /// 合并步骤
    Merge,
    /// 添加步骤
    AddStep,
    /// 简化流程
    Simplify,
}

/// 步骤序列验证器
pub struct SequenceValidator;

impl SequenceValidator {
    /// 验证执行计划的步骤序列
    ///
    /// 分析步骤顺序是否符合易经六爻逻辑
    pub fn validate(plan: &ExecutionPlan) -> SequenceValidation {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // 如果计划为空，直接返回
        if plan.steps.is_empty() {
            return SequenceValidation {
                is_valid: true,
                issues: vec![],
                suggestions: vec![],
                overall_assessment: "计划为空，无需验证".to_string(),
            };
        }

        // 分析每个步骤的性质
        let step_natures: Vec<(usize, StepNature)> = plan
            .steps
            .iter()
            .enumerate()
            .map(|(i, step)| (i, StepAnalyzer::analyze_nature(step)))
            .collect();

        // 检查1: 准备步骤应该在前面
        Self::check_preparation_position(&step_natures, &mut issues);

        // 检查2: 清理步骤应该在后面
        Self::check_cleanup_position(&step_natures, &mut issues);

        // 检查3: 输出步骤不应该在最前面
        Self::check_finalization_position(&step_natures, &mut issues);

        // 检查4: 缺少关键步骤类型
        Self::check_missing_step_types(&step_natures, &mut issues);

        // 检查5: 阴阳平衡
        Self::check_yin_yang_balance(&plan.steps, &mut issues);

        // 检查6: 复杂度
        Self::check_complexity(&plan.steps, &mut issues);

        // 生成优化建议
        Self::generate_suggestions(&step_natures, &issues, &mut suggestions);

        // 总体评价
        let overall_assessment = Self::generate_overall_assessment(&issues, plan.steps.len());

        SequenceValidation {
            is_valid: !issues.iter().any(|i| i.severity == IssueSeverity::Critical),
            issues,
            suggestions,
            overall_assessment,
        }
    }

    /// 检查准备步骤位置
    fn check_preparation_position(
        step_natures: &[(usize, StepNature)],
        issues: &mut Vec<SequenceIssue>,
    ) {
        // 找到所有准备步骤
        let prep_steps: Vec<usize> = step_natures
            .iter()
            .filter(|(_, nature)| *nature == StepNature::Preparation)
            .map(|(i, _)| *i)
            .collect();

        if prep_steps.is_empty() {
            return;
        }

        // 如果准备步骤出现在后半段，发出警告
        let total = step_natures.len();
        for &step_idx in &prep_steps {
            if step_idx > total / 2 {
                issues.push(SequenceIssue {
                    severity: IssueSeverity::Warning,
                    issue_type: IssueType::SuboptimalPosition,
                    step_indices: vec![step_idx],
                    description: format!("准备步骤 #{} 位置偏后", step_idx + 1),
                    details: Some("准备工作（读取、加载、初始化）通常应该在流程前半段完成".to_string()),
                });
            }
        }
    }

    /// 检查清理步骤位置
    fn check_cleanup_position(
        step_natures: &[(usize, StepNature)],
        issues: &mut Vec<SequenceIssue>,
    ) {
        // 找到所有清理步骤
        let cleanup_steps: Vec<usize> = step_natures
            .iter()
            .filter(|(_, nature)| *nature == StepNature::Cleanup)
            .map(|(i, _)| *i)
            .collect();

        if cleanup_steps.is_empty() {
            return;
        }

        // 如果清理步骤出现在前半段，发出警告
        let total = step_natures.len();
        for &step_idx in &cleanup_steps {
            if step_idx < total / 2 {
                issues.push(SequenceIssue {
                    severity: IssueSeverity::Warning,
                    issue_type: IssueType::SuboptimalPosition,
                    step_indices: vec![step_idx],
                    description: format!("清理步骤 #{} 位置偏前", step_idx + 1),
                    details: Some("清理工作（关闭、释放、保存）通常应该在流程后半段完成".to_string()),
                });
            }
        }
    }

    /// 检查输出步骤位置
    fn check_finalization_position(
        step_natures: &[(usize, StepNature)],
        issues: &mut Vec<SequenceIssue>,
    ) {
        // 找到所有输出步骤
        let finalization_steps: Vec<usize> = step_natures
            .iter()
            .filter(|(_, nature)| *nature == StepNature::Finalization)
            .map(|(i, _)| *i)
            .collect();

        if finalization_steps.is_empty() {
            return;
        }

        // 如果输出步骤是第一步，严重问题
        if finalization_steps.contains(&0) {
            issues.push(SequenceIssue {
                severity: IssueSeverity::Critical,
                issue_type: IssueType::InvertedOrder,
                step_indices: vec![0],
                description: "输出步骤不应该是第一步".to_string(),
                details: Some("在输出结果之前，应该先有准备、执行或处理步骤".to_string()),
            });
        }
    }

    /// 检查缺少的步骤类型
    fn check_missing_step_types(
        step_natures: &[(usize, StepNature)],
        issues: &mut Vec<SequenceIssue>,
    ) {
        let natures: Vec<StepNature> = step_natures.iter().map(|(_, n)| *n).collect();

        // 如果步骤数 >= 3，检查是否缺少准备步骤
        if step_natures.len() >= 3 && !natures.contains(&StepNature::Preparation) {
            issues.push(SequenceIssue {
                severity: IssueSeverity::Info,
                issue_type: IssueType::MissingStepType,
                step_indices: vec![],
                description: "缺少准备步骤（初爻）".to_string(),
                details: Some("建议添加读取配置、验证输入等准备工作".to_string()),
            });
        }

        // 如果有资源创建，但没有清理，发出提示
        if natures.contains(&StepNature::Execution) && !natures.contains(&StepNature::Cleanup) {
            issues.push(SequenceIssue {
                severity: IssueSeverity::Info,
                issue_type: IssueType::MissingStepType,
                step_indices: vec![],
                description: "缺少清理步骤（上爻）".to_string(),
                details: Some("建议添加资源释放、连接关闭等清理工作".to_string()),
            });
        }
    }

    /// 检查阴阳平衡
    fn check_yin_yang_balance(
        steps: &[crate::agent::decomposition::types::ExecutionStep],
        issues: &mut Vec<SequenceIssue>,
    ) {
        let (yin, yang) = StepAnalyzer::analyze_yin_yang_balance(steps);
        let total = yin + yang;

        if total == 0 {
            return;
        }

        // 如果阴阳比例严重失衡（如 90% 都是阳或阴），发出警告
        let yin_ratio = yin as f64 / total as f64;
        let yang_ratio = yang as f64 / total as f64;

        if yin_ratio > 0.8 {
            issues.push(SequenceIssue {
                severity: IssueSeverity::Warning,
                issue_type: IssueType::YinYangImbalance,
                step_indices: vec![],
                description: "阴爻过多（准备、决策步骤占比过高）".to_string(),
                details: Some(format!("阴爻 {} 步，阳爻 {} 步。建议增加执行、输出类步骤", yin, yang)),
            });
        } else if yang_ratio > 0.8 {
            issues.push(SequenceIssue {
                severity: IssueSeverity::Warning,
                issue_type: IssueType::YinYangImbalance,
                step_indices: vec![],
                description: "阳爻过多（执行、输出步骤占比过高）".to_string(),
                details: Some(format!("阴爻 {} 步，阳爻 {} 步。建议增加准备、决策类步骤", yin, yang)),
            });
        }
    }

    /// 检查复杂度
    fn check_complexity(
        steps: &[crate::agent::decomposition::types::ExecutionStep],
        issues: &mut Vec<SequenceIssue>,
    ) {
        let count = steps.len();

        // 如果步骤数超过 12 步（2个六爻循环），建议拆分
        if count > 12 {
            issues.push(SequenceIssue {
                severity: IssueSeverity::Warning,
                issue_type: IssueType::OverlyComplex,
                step_indices: vec![],
                description: format!("步骤数量过多（{} 步）", count),
                details: Some("建议将复杂任务拆分为多个子任务，每个子任务控制在 6 步以内".to_string()),
            });
        }
    }

    /// 生成优化建议
    fn generate_suggestions(
        step_natures: &[(usize, StepNature)],
        issues: &[SequenceIssue],
        suggestions: &mut Vec<SequenceSuggestion>,
    ) {
        // 如果有顺序问题，建议重排
        if issues.iter().any(|i| i.issue_type == IssueType::InvertedOrder) {
            suggestions.push(SequenceSuggestion {
                suggestion_type: SuggestionType::Reorder,
                description: "调整步骤顺序".to_string(),
                action: Some("将准备步骤移到前面，输出步骤移到后面，清理步骤放在最后".to_string()),
            });
        }

        // 如果缺少步骤，建议添加
        if issues.iter().any(|i| i.issue_type == IssueType::MissingStepType) {
            suggestions.push(SequenceSuggestion {
                suggestion_type: SuggestionType::AddStep,
                description: "补充缺少的步骤类型".to_string(),
                action: Some("添加必要的准备或清理步骤".to_string()),
            });
        }

        // 如果过于复杂，建议拆分
        if issues.iter().any(|i| i.issue_type == IssueType::OverlyComplex) {
            suggestions.push(SequenceSuggestion {
                suggestion_type: SuggestionType::Split,
                description: "拆分为多个子任务".to_string(),
                action: Some("将任务按照逻辑阶段拆分，每个阶段独立执行".to_string()),
            });
        }

        // 如果没有问题，给出肯定建议
        if issues.is_empty() && step_natures.len() >= 3 {
            suggestions.push(SequenceSuggestion {
                suggestion_type: SuggestionType::Simplify,
                description: "当前步骤顺序合理".to_string(),
                action: Some("可以直接执行，无需调整".to_string()),
            });
        }
    }

    /// 生成总体评价
    fn generate_overall_assessment(issues: &[SequenceIssue], step_count: usize) -> String {
        if issues.is_empty() {
            return format!("✅ 步骤序列合理（共 {} 步，符合爻序逻辑）", step_count);
        }

        let critical = issues.iter().filter(|i| i.severity == IssueSeverity::Critical).count();
        let warning = issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count();
        let info = issues.iter().filter(|i| i.severity == IssueSeverity::Info).count();

        if critical > 0 {
            format!(
                "❌ 发现 {} 个严重问题，{} 个警告，{} 个提示",
                critical, warning, info
            )
        } else if warning > 0 {
            format!("⚠️  发现 {} 个警告，{} 个提示（可优化）", warning, info)
        } else {
            format!("💡 发现 {} 个优化建议", info)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::decomposition::types::ExecutionStep;

    #[test]
    fn test_ideal_sequence() {
        // 理想序列：准备 -> 执行 -> 输出 -> 清理
        let plan = ExecutionPlan::new(
            "理想任务".to_string(),
            vec![
                ExecutionStep::new("读取配置".to_string(), "read_config".to_string(), 1.0),
                ExecutionStep::new("创建文件".to_string(), "create_file".to_string(), 1.0),
                ExecutionStep::new("写入数据".to_string(), "write_data".to_string(), 1.0),
                ExecutionStep::new("输出结果".to_string(), "display_result".to_string(), 0.5),
                ExecutionStep::new("关闭文件".to_string(), "close_file".to_string(), 0.3),
            ],
        );

        let validation = SequenceValidator::validate(&plan);

        assert!(validation.is_valid);
        assert_eq!(validation.critical_count(), 0);
    }

    #[test]
    fn test_inverted_order() {
        // 严重问题：输出在第一步
        let plan = ExecutionPlan::new(
            "颠倒任务".to_string(),
            vec![
                ExecutionStep::new("输出结果".to_string(), "display_result".to_string(), 0.5),
                ExecutionStep::new("创建文件".to_string(), "create_file".to_string(), 1.0),
            ],
        );

        let validation = SequenceValidator::validate(&plan);

        assert!(!validation.is_valid);
        assert!(validation.has_critical_issues());
        assert!(validation.issues.iter().any(|i| i.issue_type == IssueType::InvertedOrder));
    }

    #[test]
    fn test_missing_preparation() {
        // 缺少准备步骤
        let plan = ExecutionPlan::new(
            "缺少准备".to_string(),
            vec![
                ExecutionStep::new("创建文件".to_string(), "create_file".to_string(), 1.0),
                ExecutionStep::new("写入数据".to_string(), "write_data".to_string(), 1.0),
                ExecutionStep::new("输出结果".to_string(), "display_result".to_string(), 0.5),
            ],
        );

        let validation = SequenceValidator::validate(&plan);

        // 应该有提示，但不算严重问题
        assert!(validation.is_valid);
        assert!(validation.issues.iter().any(|i| i.issue_type == IssueType::MissingStepType));
    }

    #[test]
    fn test_overly_complex() {
        // 步骤过多
        let steps: Vec<ExecutionStep> = (0..15)
            .map(|i| ExecutionStep::new(format!("步骤 {}", i), "tool".to_string(), 1.0))
            .collect();

        let plan = ExecutionPlan::new("复杂任务".to_string(), steps);
        let validation = SequenceValidator::validate(&plan);

        assert!(validation.issues.iter().any(|i| i.issue_type == IssueType::OverlyComplex));
        assert!(validation.suggestions.iter().any(|s| s.suggestion_type == SuggestionType::Split));
    }

    #[test]
    fn test_empty_plan() {
        let plan = ExecutionPlan::new("空计划".to_string(), vec![]);
        let validation = SequenceValidator::validate(&plan);

        assert!(validation.is_valid);
        assert_eq!(validation.issues.len(), 0);
    }

    #[test]
    fn test_yin_yang_balance() {
        // 全部是阳爻（执行类）
        let plan = ExecutionPlan::new(
            "阳爻过多".to_string(),
            vec![
                ExecutionStep::new("创建1".to_string(), "create".to_string(), 1.0),
                ExecutionStep::new("创建2".to_string(), "create".to_string(), 1.0),
                ExecutionStep::new("创建3".to_string(), "create".to_string(), 1.0),
                ExecutionStep::new("创建4".to_string(), "create".to_string(), 1.0),
                ExecutionStep::new("创建5".to_string(), "create".to_string(), 1.0),
            ],
        );

        let validation = SequenceValidator::validate(&plan);

        assert!(validation.issues.iter().any(|i| i.issue_type == IssueType::YinYangImbalance));
    }

    #[test]
    fn test_cleanup_position() {
        // 清理步骤在前面
        let plan = ExecutionPlan::new(
            "清理步骤位置不当".to_string(),
            vec![
                ExecutionStep::new("关闭连接".to_string(), "close".to_string(), 0.3),
                ExecutionStep::new("创建文件".to_string(), "create".to_string(), 1.0),
                ExecutionStep::new("写入数据".to_string(), "write".to_string(), 1.0),
                ExecutionStep::new("输出结果".to_string(), "output".to_string(), 0.5),
            ],
        );

        let validation = SequenceValidator::validate(&plan);

        assert!(validation.issues.iter().any(|i| i.issue_type == IssueType::SuboptimalPosition));
    }
}
