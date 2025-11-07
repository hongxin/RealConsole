//! Intent 路由器 - v1.31.0
//!
//! 在 LLM 拆解之前尝试 Intent 预识别，提升简单任务的响应速度

use super::types::{ExecutionPlan, ExecutionStep};
use crate::dsl::intent::{BuiltinIntents, IntentMatcher, TemplateEngine};
use serde_json::json;

/// Intent 路由器
///
/// 将简单的自然语言意图通过 Intent DSL 快速识别并转换为执行计划
pub struct IntentRouter {
    matcher: IntentMatcher,
    engine: TemplateEngine,
    confidence_threshold: f64,
}

impl IntentRouter {
    /// 创建新的 Intent 路由器
    ///
    /// 使用内置的 24 个 Intent 和默认置信度阈值 0.7
    pub fn new() -> Self {
        let builtin = BuiltinIntents::new();
        Self {
            matcher: builtin.create_matcher(),
            engine: builtin.create_engine(),
            confidence_threshold: 0.7,
        }
    }

    /// 设置置信度阈值
    ///
    /// # 参数
    ///
    /// - `threshold`: 置信度阈值（0.0-1.0）
    ///
    /// # 建议值
    ///
    /// - 0.8-1.0: 高准确性，低覆盖率
    /// - 0.6-0.8: 平衡（推荐）
    /// - <0.6: 高覆盖率，可能误判
    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// 尝试通过 Intent 匹配生成执行计划
    ///
    /// # 参数
    ///
    /// - `input`: 用户输入的自然语言
    ///
    /// # 返回
    ///
    /// - `Some(ExecutionPlan)`: 匹配成功，返回单步执行计划
    /// - `None`: 匹配失败或置信度不足，应回退到 LLM 拆解
    ///
    /// # 示例
    ///
    /// ```no_run
    /// let router = IntentRouter::new();
    ///
    /// // 成功匹配
    /// if let Some(plan) = router.try_match("查看当前目录") {
    ///     println!("Intent 识别成功: {}", plan.understanding);
    ///     // plan.steps[0].tool == "shell_execute"
    ///     // plan.steps[0].params == {"command": "ls -la"}
    /// }
    ///
    /// // 失败回退
    /// if let None = router.try_match("帮我优化这段代码") {
    ///     // 复杂任务，需要 LLM 拆解
    /// }
    /// ```
    pub fn try_match(&self, input: &str) -> Option<ExecutionPlan> {
        // 1. Intent 匹配
        let matches = self.matcher.match_intent(input);
        let best_match = matches.first()?;

        // 2. 检查置信度
        if best_match.confidence < self.confidence_threshold {
            eprintln!(
                "🔍 [Intent] 置信度不足: {} (需要 >= {})",
                best_match.confidence, self.confidence_threshold
            );
            return None;
        }

        // 3. 生成 shell 命令
        let template_plan = self.engine.generate_from_intent(best_match).ok()?;

        eprintln!(
            "✨ [Intent] 匹配成功: {} (置信度: {:.2})",
            best_match.intent.name, best_match.confidence
        );
        eprintln!("   命令: {}", template_plan.command);

        // 4. 转换为 ExecutionPlan
        let understanding = format!(
            "通过 Intent DSL 快速识别：{} (置信度: {:.2})",
            best_match.intent.name, best_match.confidence
        );

        // v1.31.0: 使用 shell_execute 工具执行 Intent 生成的命令
        let step = ExecutionStep::new(
            format!("执行命令: {}", template_plan.command),
            "shell_execute".to_string(),
            0.5, // shell 命令预计 0.5 秒
        )
        .with_params(json!({
            "command": template_plan.command
        }));

        Some(ExecutionPlan::new(understanding, vec![step]))
    }

    /// 获取当前置信度阈值
    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold
    }
}

impl Default for IntentRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_router_creation() {
        let router = IntentRouter::new();
        assert_eq!(router.confidence_threshold(), 0.7);
    }

    #[test]
    fn test_set_threshold() {
        let router = IntentRouter::new().with_threshold(0.8);
        assert_eq!(router.confidence_threshold(), 0.8);

        // 测试边界值
        let router = IntentRouter::new().with_threshold(1.5);
        assert_eq!(router.confidence_threshold(), 1.0);

        let router = IntentRouter::new().with_threshold(-0.1);
        assert_eq!(router.confidence_threshold(), 0.0);
    }

    #[test]
    fn test_simple_intent_match() {
        let router = IntentRouter::new();

        // 应该匹配的简单命令
        let plan = router.try_match("查看当前目录");
        assert!(plan.is_some(), "应该匹配 list_directory");

        if let Some(plan) = plan {
            assert_eq!(plan.step_count(), 1);
            assert_eq!(plan.steps[0].tool, "shell_execute");
            assert!(plan.understanding.contains("Intent DSL"));
        }
    }

    #[test]
    fn test_complex_input_no_match() {
        let router = IntentRouter::new();

        // 复杂任务不应该匹配
        let plan = router.try_match("帮我分析这段代码的性能瓶颈并给出优化建议");
        assert!(plan.is_none(), "复杂任务应该返回 None 以回退到 LLM");
    }

    #[test]
    fn test_low_confidence_no_match() {
        let router = IntentRouter::new().with_threshold(0.95);

        // 高阈值下，一般的匹配应该失败
        let plan = router.try_match("目录");
        assert!(plan.is_none(), "置信度不足应该返回 None");
    }
}
