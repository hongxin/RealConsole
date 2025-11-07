//! Intent 路由器 - v1.31.0 / v1.32.0
//!
//! 在 LLM 拆解之前尝试 Intent 预识别，提升简单任务的响应速度
//!
//! v1.31.0: Intent 预识别 + shell_execute
//! v1.32.0: 智能工具路由（Intent → 专用工具）

use super::tool_router::ToolRouter;
use super::types::{ExecutionPlan, ExecutionStep};
use crate::dsl::intent::{BuiltinIntents, IntentMatcher, TemplateEngine};
use serde_json::json;

/// Intent 路由器
///
/// 将简单的自然语言意图通过 Intent DSL 快速识别并转换为执行计划
///
/// # 版本演进
///
/// - v1.31.0: 所有 Intent 映射到 shell_execute
/// - v1.32.0: 部分 Intent 映射到专用工具（list_dir, count_code_lines）
pub struct IntentRouter {
    matcher: IntentMatcher,
    engine: TemplateEngine,
    tool_router: ToolRouter, // v1.32.0: 工具路由器
    confidence_threshold: f64,
}

impl IntentRouter {
    /// 创建新的 Intent 路由器
    ///
    /// 使用内置的 24 个 Intent 和默认置信度阈值 0.7
    ///
    /// v1.32.0: 同时初始化工具路由器
    pub fn new() -> Self {
        let builtin = BuiltinIntents::new();
        Self {
            matcher: builtin.create_matcher(),
            engine: builtin.create_engine(),
            tool_router: ToolRouter::new(), // v1.32.0
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

        eprintln!(
            "✨ [Intent] 匹配成功: {} (置信度: {:.2})",
            best_match.intent.name, best_match.confidence
        );

        // 3. v1.32.0: 尝试工具路由（Intent → 专用工具）
        let (tool, params, description) = if let Some((tool_name, tool_params)) =
            self.tool_router.route(best_match)
        {
            // 使用专用工具
            (
                tool_name.clone(),
                tool_params,
                format!("使用专用工具: {}", tool_name),
            )
        } else {
            // 回退到 shell_execute（v1.31.0 逻辑）
            let template_plan = self.engine.generate_from_intent(best_match).ok()?;
            eprintln!("   命令: {}", template_plan.command);

            (
                "shell_execute".to_string(),
                json!({"command": template_plan.command}),
                format!("执行命令: {}", template_plan.command),
            )
        };

        // 4. 转换为 ExecutionPlan
        let understanding = format!(
            "通过 Intent DSL 快速识别：{} (置信度: {:.2})",
            best_match.intent.name, best_match.confidence
        );

        let step = ExecutionStep::new(
            description,
            tool,
            0.5, // 预计执行时间
        )
        .with_params(params);

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
            // v1.32.0: list_directory 映射到 list_dir 专用工具
            assert_eq!(plan.steps[0].tool, "list_dir");
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
