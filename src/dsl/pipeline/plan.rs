//! 执行计划 - Pipeline DSL 的"卦"
//!
//! **哲学**：
//! - 执行计划是基础操作（象）的组合
//! - 不同的组合产生不同的"卦象"
//! - 64卦 = 8×8 种组合，这里的组合空间更大

use super::operations::BaseOperation;
use serde::{Deserialize, Serialize};

/// 执行计划（卦）
///
/// **设计思想**：
/// - 一个计划由多个操作组成
/// - 操作按顺序执行，形成管道
/// - 最终生成 Shell 命令
///
/// **易经对应**：
/// - 内卦（前3爻）：基础操作（如 FindFiles）
/// - 外卦（后3爻）：修饰操作（如 SortFiles, LimitFiles）
/// - 完整的6爻：一个完整的执行计划
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// 操作列表（按执行顺序）
    pub operations: Vec<BaseOperation>,
}

impl ExecutionPlan {
    /// 创建新的执行计划
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// 添加操作
    pub fn with_operation(mut self, operation: BaseOperation) -> Self {
        self.operations.push(operation);
        self
    }

    /// 生成 Shell 命令
    ///
    /// **实现思路**：
    /// - 将所有操作的命令片段用管道连接
    /// - 第一个操作不需要管道
    /// - 后续操作用 `|` 连接
    pub fn to_shell_command(&self) -> String {
        if self.operations.is_empty() {
            return String::new();
        }

        let mut command = String::new();

        for (index, operation) in self.operations.iter().enumerate() {
            if index > 0 {
                command.push_str(" | ");
            }
            command.push_str(&operation.to_shell_fragment());
        }

        command
    }

    /// 验证执行计划的有效性
    ///
    /// **规则**：
    /// - 至少包含一个操作
    /// - 第一个操作必须是数据源（FindFiles 或 ListFiles）
    pub fn validate(&self) -> Result<(), String> {
        if self.operations.is_empty() {
            return Err("执行计划不能为空".to_string());
        }

        match &self.operations[0] {
            BaseOperation::FindFiles { .. } | BaseOperation::ListFiles { .. } => Ok(()),
            _ => Err("第一个操作必须是数据源（FindFiles 或 ListFiles）".to_string()),
        }
    }

    /// 获取操作数量
    pub fn len(&self) -> usize {
        self.operations.len()
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::pipeline::{Direction, Field};

    #[test]
    fn test_empty_plan() {
        let plan = ExecutionPlan::new();
        assert_eq!(plan.len(), 0);
        assert!(plan.is_empty());
        assert_eq!(plan.to_shell_command(), "");
    }

    #[test]
    fn test_single_operation_plan() {
        let plan = ExecutionPlan::new().with_operation(BaseOperation::FindFiles {
            path: ".".to_string(),
            pattern: "*.rs".to_string(),
        });

        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan.to_shell_command(),
            "find . -name '*.rs' -type f -exec ls -lh {} +"
        );
    }

    #[test]
    fn test_find_largest_files() {
        // 案例：查找最大的 rs 文件
        let plan = ExecutionPlan::new()
            .with_operation(BaseOperation::FindFiles {
                path: ".".to_string(),
                pattern: "*.rs".to_string(),
            })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Size,
                direction: Direction::Descending,
            })
            .with_operation(BaseOperation::LimitFiles { count: 10 });

        assert_eq!(plan.len(), 3);
        assert_eq!(
            plan.to_shell_command(),
            "find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -hr | head -n 10"
        );
    }

    #[test]
    fn test_find_smallest_files() {
        // 案例：查找最小的 rs 文件
        // **核心验证**：只需改变 Direction 参数，其他完全相同！
        let plan = ExecutionPlan::new()
            .with_operation(BaseOperation::FindFiles {
                path: ".".to_string(),
                pattern: "*.rs".to_string(),
            })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Size,
                direction: Direction::Ascending,  // 唯一的区别！
            })
            .with_operation(BaseOperation::LimitFiles { count: 1 });

        assert_eq!(plan.len(), 3);
        assert_eq!(
            plan.to_shell_command(),
            "find . -name '*.rs' -type f -exec ls -lh {} + | sort -k5 -h | head -n 1"
        );
    }

    #[test]
    fn test_find_newest_files() {
        // 案例：查找最新的文件
        let plan = ExecutionPlan::new()
            .with_operation(BaseOperation::FindFiles {
                path: ".".to_string(),
                pattern: "*".to_string(),
            })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Time,  // 改变字段
                direction: Direction::Descending,
            })
            .with_operation(BaseOperation::LimitFiles { count: 5 });

        assert_eq!(plan.len(), 3);
        assert_eq!(
            plan.to_shell_command(),
            "find . -name '*' -type f -exec ls -lh {} + | sort -k6 -hr | head -n 5"
        );
    }

    #[test]
    fn test_list_directory() {
        // 案例：简单列出目录
        let plan = ExecutionPlan::new().with_operation(BaseOperation::ListFiles {
            path: ".".to_string(),
        });

        assert_eq!(plan.len(), 1);
        assert_eq!(plan.to_shell_command(), "ls -lh .");
    }

    #[test]
    fn test_plan_validation_empty() {
        let plan = ExecutionPlan::new();
        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_plan_validation_valid() {
        let plan = ExecutionPlan::new().with_operation(BaseOperation::FindFiles {
            path: ".".to_string(),
            pattern: "*.rs".to_string(),
        });

        assert!(plan.validate().is_ok());
    }

    #[test]
    fn test_plan_validation_invalid_first_operation() {
        // 第一个操作不是数据源
        let plan = ExecutionPlan::new().with_operation(BaseOperation::SortFiles {
            field: Field::Size,
            direction: Direction::Descending,
        });

        assert!(plan.validate().is_err());
    }

    #[test]
    fn test_philosophy_demonstration() {
        // **哲学演示**：从"最大"到"最小"只是"爻"的变化

        // 最大的3个文件
        let largest = ExecutionPlan::new()
            .with_operation(BaseOperation::FindFiles {
                path: ".".to_string(),
                pattern: "*.rs".to_string(),
            })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Size,
                direction: Direction::Descending,  // 爻1
            })
            .with_operation(BaseOperation::LimitFiles { count: 3 });

        // 最小的3个文件
        let smallest = ExecutionPlan::new()
            .with_operation(BaseOperation::FindFiles {
                path: ".".to_string(),
                pattern: "*.rs".to_string(),
            })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Size,
                direction: Direction::Ascending,  // 爻1的变化
            })
            .with_operation(BaseOperation::LimitFiles { count: 3 });

        // 操作结构完全相同，只有一个参数不同
        assert_eq!(largest.len(), smallest.len());
        assert_eq!(largest.len(), 3);

        // 但生成的命令不同
        assert_ne!(
            largest.to_shell_command(),
            smallest.to_shell_command()
        );

        println!("最大: {}", largest.to_shell_command());
        println!("最小: {}", smallest.to_shell_command());

        // 这就是"变"的本质：
        // - 象（操作）不变
        // - 爻（参数）变化
        // - 生成无穷变体
    }
}
