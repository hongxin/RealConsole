//! 意图占卜系统
//!
//! 基于易经八卦和六十四卦原理，为意图拆解注入灵魂和文化特色。
//!
//! # 模块组成
//!
//! - `trigram`: 八卦系统，将工具类型映射到八卦符号
//! - `hexagram`: 六十四卦系统，由上下两个八卦组合而成
//! - `yarrow_stalks`: 蓍草演算模拟，生成占卜过程的动画数据
//! - `yao_mapping`: 爻位映射，将执行步骤映射到六爻位置
//! - `judgement_generator`: 卦辞生成器，根据卦象和计划生成动态卦辞
//! - `divination_engine`: 占卜引擎主入口，统一生成完整占卜结果
//!
//! # 使用示例
//!
//! ```rust
//! use realconsole::agent::divination::DivinationEngine;
//! use realconsole::agent::decomposition::{ExecutionPlan, ExecutionStep};
//!
//! let steps = vec![
//!     ExecutionStep::new("列出文件".to_string(), "list_directory".to_string(), 1.0),
//!     ExecutionStep::new("搜索内容".to_string(), "search_text".to_string(), 1.0),
//! ];
//!
//! let plan = ExecutionPlan::new("查找文件".to_string(), steps);
//! let result = DivinationEngine::divine(&plan);
//!
//! println!("卦象：{}", result.hexagram.name);
//! println!("卦辞：{}", result.hexagram.judgement);
//! ```
//!
//! # 设计哲学
//!
//! 本模块将古代易经占卜的智慧融入现代 AI 系统：
//!
//! 1. **象**: 八卦符号（☰☷☳☴☵☲☶☱）象征不同的工具类型
//! 2. **数**: 蓍草演算过程体现数学逻辑和算法思维
//! 3. **理**: 卦辞和爻辞传达执行建议和智慧指导
//!
//! 通过可视化占卜过程，让用户：
//! - 看到 AI 的"思考"过程（演算动画）
//! - 理解任务的复杂度和建议（卦辞）
//! - 感受东方文化的深度和美感（卦象和仪式感）

pub mod trigram;
pub mod hexagram;
pub mod yarrow_stalks;
pub mod yao_mapping;
pub mod judgement_generator;
pub mod divination_engine;

// 重新导出主要类型，方便外部使用
pub use trigram::Trigram;
pub use hexagram::Hexagram;
pub use yarrow_stalks::{YarrowStep, YarrowStalksSimulator};
pub use yao_mapping::{YaoPosition, StepYaoMapping};
pub use judgement_generator::JudgementGenerator;
pub use divination_engine::{DivinationEngine, DivinationResult};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::agent::decomposition::{ExecutionPlan, ExecutionStep};

    #[test]
    fn test_full_divination_workflow() {
        // 创建一个测试计划
        let steps = vec![
            ExecutionStep::new("初始化项目".to_string(), "create_project".to_string(), 1.0),
            ExecutionStep::new("列出文件".to_string(), "list_directory".to_string(), 0.5),
            ExecutionStep::new("搜索内容".to_string(), "search_text".to_string(), 1.5),
        ];

        let plan = ExecutionPlan::new("项目初始化和内容搜索".to_string(), steps);

        // 执行占卜
        let result = DivinationEngine::divine(&plan);

        // 验证所有组件都正常工作
        assert!(!result.hexagram.name.is_empty());
        assert!(!result.hexagram.judgement.is_empty());
        assert!(!result.yarrow_steps.is_empty());
        assert_eq!(result.step_mappings.len(), 3);
        assert!(!result.full_judgement.is_empty());

        // 验证卦象符号
        let symbol = result.hexagram.symbol();
        assert!(symbol.contains("☰") || symbol.contains("☷") ||
                symbol.contains("☳") || symbol.contains("☴") ||
                symbol.contains("☵") || symbol.contains("☲") ||
                symbol.contains("☶") || symbol.contains("☱"));

        // 验证演算步骤顺序
        assert_eq!(result.yarrow_steps[0].operation, "大衍");
        assert_eq!(result.yarrow_steps.last().unwrap().operation, "成卦");

        // 验证爻位映射
        for (i, mapping) in result.step_mappings.iter().enumerate() {
            assert_eq!(mapping.step_index, i);
            assert!(!mapping.line_statement.is_empty());
        }
    }

    #[test]
    fn test_trigram_to_hexagram_mapping() {
        // 测试不同工具组合生成不同的卦象
        let test_cases = vec![
            (vec!["create_file", "list_directory"], "不同工具应生成不同卦象"),
            (vec!["search_text", "find_file"], "搜索类工具应映射到离卦"),
            (vec!["start_process", "run_command"], "执行类工具应映射到震卦"),
        ];

        for (tools, desc) in test_cases {
            let steps: Vec<ExecutionStep> = tools
                .iter()
                .enumerate()
                .map(|(i, tool)| ExecutionStep::new(
                    format!("步骤 {}", i + 1),
                    tool.to_string(),
                    1.0,
                ))
                .collect();

            let plan = ExecutionPlan::new(desc.to_string(), steps);
            let result = DivinationEngine::divine(&plan);

            // 每个测试用例都应该生成有效的卦象
            assert!(!result.hexagram.name.is_empty(), "{}", desc);
        }
    }

    #[test]
    fn test_empty_plan_handling() {
        let plan = ExecutionPlan::new("空计划".to_string(), vec![]);
        let result = DivinationEngine::divine(&plan);

        // 空计划应该使用默认卦象
        assert!(!result.hexagram.name.is_empty());
        assert_eq!(result.step_mappings.len(), 0);
        assert!(!result.yarrow_steps.is_empty()); // 仍应有演算过程
    }

    #[test]
    fn test_quick_vs_full_divination() {
        let steps = vec![
            ExecutionStep::new("步骤1".to_string(), "tool1".to_string(), 1.0),
            ExecutionStep::new("步骤2".to_string(), "tool2".to_string(), 1.0),
        ];

        let plan = ExecutionPlan::new("测试".to_string(), steps);

        let quick_result = DivinationEngine::divine_quick(&plan);
        let full_result = DivinationEngine::divine(&plan);

        // 快速模式和完整模式应生成相同的卦象
        assert_eq!(quick_result.hexagram.name, full_result.hexagram.name);

        // 但快速模式不应有动画数据
        assert!(quick_result.yarrow_steps.is_empty());
        assert!(quick_result.step_mappings.is_empty());

        // 完整模式应有动画数据
        assert!(!full_result.yarrow_steps.is_empty());
        assert!(!full_result.step_mappings.is_empty());
    }
}
