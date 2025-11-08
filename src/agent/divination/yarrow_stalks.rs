//! 蓍草演算模拟
//!
//! 模拟古代占卜的蓍草演算过程（用于动画数据生成）

use serde::{Deserialize, Serialize};

/// 演算步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YarrowStep {
    /// 操作名称
    pub operation: String,
    /// 当前数值
    pub value: usize,
    /// 操作描述
    pub description: String,
}

/// 蓍草演算器
pub struct YarrowStalksSimulator;

impl YarrowStalksSimulator {
    /// 模拟演算过程（生成动画数据）
    ///
    /// # 参数
    /// - `target_steps`: 目标步骤数（ExecutionPlan 的步骤数）
    ///
    /// # 返回
    /// 演算步骤列表（用于前端动画）
    pub fn simulate(target_steps: usize) -> Vec<YarrowStep> {
        let mut steps = Vec::new();
        let mut current = 49;  // 大衍之数五十，其用四十有九

        // 第一步：大衍
        steps.push(YarrowStep {
            operation: "大衍".to_string(),
            value: current,
            description: "大衍之数五十，其用四十有九".to_string(),
        });

        // 简化演算：逐步减少到目标步骤数
        let operations = ["分二", "挂一", "揲四", "归奇"];
        let decrement = if current > target_steps {
            (current - target_steps) / operations.len()
        } else {
            0
        };

        for operation in operations {
            current = current.saturating_sub(decrement);
            steps.push(YarrowStep {
                operation: operation.to_string(),
                value: current,
                description: Self::get_description(operation),
            });
        }

        // 最后一步：成卦
        steps.push(YarrowStep {
            operation: "成卦".to_string(),
            value: target_steps,
            description: format!("得 {} 个步骤", target_steps),
        });

        steps
    }

    fn get_description(operation: &str) -> String {
        match operation {
            "分二" => "分而为二，以象两仪".to_string(),
            "挂一" => "挂一以象三才".to_string(),
            "揲四" => "揲之以四，以象四时".to_string(),
            "归奇" => "归奇于扐，以象闰余".to_string(),
            _ => String::new(),
        }
    }

    /// 生成更真实的演算过程（三变成爻）
    ///
    /// 这是更接近传统占卜的方式，每个爻需要三次变化
    #[allow(dead_code)]
    pub fn simulate_traditional(yao_count: usize) -> Vec<YarrowStep> {
        let mut steps = Vec::new();

        // 大衍之数
        steps.push(YarrowStep {
            operation: "大衍".to_string(),
            value: 49,
            description: "大衍之数五十，其用四十有九".to_string(),
        });

        // 为每个爻进行三变
        for yao_idx in 0..yao_count {
            for change_idx in 0..3 {
                let mut current = 49;

                // 分二
                current = current / 2;
                steps.push(YarrowStep {
                    operation: format!("第{}爻-第{}变-分二", yao_idx + 1, change_idx + 1),
                    value: current,
                    description: "分而为二，以象两仪".to_string(),
                });

                // 挂一
                current -= 1;
                steps.push(YarrowStep {
                    operation: format!("第{}爻-第{}变-挂一", yao_idx + 1, change_idx + 1),
                    value: current,
                    description: "挂一以象三才".to_string(),
                });

                // 揲四
                let remainder = current % 4;
                current -= remainder;
                steps.push(YarrowStep {
                    operation: format!("第{}爻-第{}变-揲四", yao_idx + 1, change_idx + 1),
                    value: current,
                    description: format!("揲之以四，得 {}", remainder),
                });
            }

            // 成爻
            steps.push(YarrowStep {
                operation: format!("第{}爻成", yao_idx + 1),
                value: yao_idx + 1,
                description: format!("第 {} 爻已成", yao_idx + 1),
            });
        }

        // 最终成卦
        steps.push(YarrowStep {
            operation: "成卦".to_string(),
            value: yao_count,
            description: format!("六爻已成，共 {} 爻", yao_count),
        });

        steps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yarrow_simulation() {
        let steps = YarrowStalksSimulator::simulate(3);

        // 应该有：大衍 + 4个操作 + 成卦 = 6步
        assert_eq!(steps.len(), 6);

        // 第一步应该是大衍
        assert_eq!(steps[0].operation, "大衍");
        assert_eq!(steps[0].value, 49);

        // 最后一步应该是成卦
        assert_eq!(steps.last().unwrap().operation, "成卦");
        assert_eq!(steps.last().unwrap().value, 3);
    }

    #[test]
    fn test_yarrow_operations() {
        let steps = YarrowStalksSimulator::simulate(5);

        // 检查是否包含核心操作
        let operations: Vec<String> = steps.iter().map(|s| s.operation.clone()).collect();
        assert!(operations.contains(&"大衍".to_string()));
        assert!(operations.contains(&"分二".to_string()));
        assert!(operations.contains(&"成卦".to_string()));
    }

    #[test]
    fn test_traditional_simulation() {
        let steps = YarrowStalksSimulator::simulate_traditional(6);

        // 应该有：大衍 + 6爻*3变*3步 + 6个成爻 + 最终成卦
        // = 1 + 54 + 6 + 1 = 62步
        assert!(steps.len() > 50);

        // 第一步是大衍
        assert_eq!(steps[0].operation, "大衍");

        // 最后一步是成卦
        assert_eq!(steps.last().unwrap().operation, "成卦");
    }

    #[test]
    fn test_step_descriptions() {
        let steps = YarrowStalksSimulator::simulate(4);

        // 检查描述不为空
        for step in &steps {
            assert!(!step.description.is_empty());
        }
    }
}
