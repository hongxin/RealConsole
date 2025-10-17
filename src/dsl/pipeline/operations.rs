//! 基础操作定义 - Pipeline DSL 的"象"
//!
//! **哲学**：
//! - 这些操作是"不变"的（象）
//! - 但它们的参数可以"变化"（爻）
//! - 组合产生无穷变化（卦）

use serde::{Deserialize, Serialize};

/// 排序字段（爻之一）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Field {
    /// 文件大小
    Size,
    /// 修改时间
    Time,
    /// 文件名
    Name,
    /// 默认字段（不指定列，sort 使用第一列）
    /// 用于 du 等输出格式简单的命令
    Default,
}

impl Field {
    /// 转换为 sort 命令的列索引
    ///
    /// ls -lh 输出格式：
    /// ```text
    /// -rw-r--r--  1 user  group   47K Oct 15 21:41 file.rs
    /// [  权限  ] [硬链接] [用户] [组] [大小] [  时间  ] [文件名]
    ///   1          2       3     4      5     6 7 8      9
    /// ```
    ///
    /// du 输出格式：
    /// ```text
    /// 47K    ./src/file.rs
    /// [大小]  [路径]
    ///   1      2
    /// ```
    pub fn to_sort_key(&self) -> Option<&'static str> {
        match self {
            Field::Size => Some("5"),    // 第5列：大小
            Field::Time => Some("6"),    // 第6列：时间（月份）
            Field::Name => Some("9"),    // 第9列：文件名
            Field::Default => None,      // 不指定列，默认第一列
        }
    }
}

/// 排序方向（爻之二）
///
/// **核心洞察**：
/// - 最大 vs 最小 不是两个独立的操作
/// - 而是同一个操作在不同方向上的变化
/// - 这就是"爻"的本质 - 变化的维度
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// 升序（从小到大）- 用于"最小"
    Ascending,
    /// 降序（从大到小）- 用于"最大"
    Descending,
}

impl Direction {
    /// 转换为 sort 命令的参数
    pub fn to_sort_flag(&self) -> &'static str {
        match self {
            Direction::Ascending => "-h",      // 升序：按人类可读格式排序
            Direction::Descending => "-hr",    // 降序：-r = reverse
        }
    }
}

/// 基础操作（象）
///
/// **设计原则**：
/// - 每个操作代表一个不可再分的基础功能
/// - 操作之间可以组合，形成管道
/// - 操作的参数可以变化，体现"爻"的思想
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BaseOperation {
    /// 查找文件
    ///
    /// **参数**：
    /// - `path`: 搜索路径
    /// - `pattern`: 文件名模式（支持通配符）
    FindFiles {
        path: String,
        pattern: String,
    },

    /// 列出文件（无过滤）
    ///
    /// **参数**：
    /// - `path`: 目录路径
    ListFiles {
        path: String,
    },

    /// 检查磁盘使用（Phase 6.3 Step 2）
    ///
    /// **参数**：
    /// - `path`: 目录路径
    ///
    /// **输出格式**: `<size>\t<path>` (第一列是大小)
    DiskUsage {
        path: String,
    },

    /// 排序文件
    ///
    /// **参数**：
    /// - `field`: 排序字段（大小/时间/名称/默认）
    /// - `direction`: 排序方向（升序/降序）
    ///
    /// **哲学体现**：
    /// - field 和 direction 是两个独立的"爻"
    /// - 它们的组合产生不同的"卦象"
    SortFiles {
        field: Field,
        direction: Direction,
    },

    /// 限制输出数量
    ///
    /// **参数**：
    /// - `count`: 显示前 N 条结果
    LimitFiles {
        count: usize,
    },

    /// 过滤文件（按条件）
    ///
    /// **参数**：
    /// - `condition`: 过滤条件（未来扩展：大小、时间范围等）
    FilterFiles {
        condition: String,
    },
}

impl BaseOperation {
    /// 生成该操作对应的 Shell 命令片段
    ///
    /// **设计**：
    /// - 每个操作独立生成命令片段
    /// - 片段之间通过管道连接
    /// - 体现 Unix 哲学：组合小工具
    pub fn to_shell_fragment(&self) -> String {
        match self {
            BaseOperation::FindFiles { path, pattern } => {
                format!("find {} -name '{}' -type f -exec ls -lh {{}} +", path, pattern)
            }

            BaseOperation::ListFiles { path } => {
                format!("ls -lh {}", path)
            }

            BaseOperation::DiskUsage { path } => {
                format!("du -sh {}/*", path)
            }

            BaseOperation::SortFiles { field, direction } => {
                // Field::Default 不指定列（用于 du 等简单输出）
                if let Some(key) = field.to_sort_key() {
                    format!("sort -k{} {}", key, direction.to_sort_flag())
                } else {
                    format!("sort {}", direction.to_sort_flag())
                }
            }

            BaseOperation::LimitFiles { count } => {
                format!("head -n {}", count)
            }

            BaseOperation::FilterFiles { condition } => {
                format!("grep '{}'", condition)
            }
        }
    }

    /// 判断该操作是否需要管道连接
    pub fn needs_pipe(&self) -> bool {
        !matches!(self, BaseOperation::ListFiles { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_sort_key() {
        assert_eq!(Field::Size.to_sort_key(), Some("5"));
        assert_eq!(Field::Time.to_sort_key(), Some("6"));
        assert_eq!(Field::Name.to_sort_key(), Some("9"));
        assert_eq!(Field::Default.to_sort_key(), None);
    }

    #[test]
    fn test_direction_sort_flag() {
        assert_eq!(Direction::Ascending.to_sort_flag(), "-h");
        assert_eq!(Direction::Descending.to_sort_flag(), "-hr");
    }

    #[test]
    fn test_find_files_fragment() {
        let op = BaseOperation::FindFiles {
            path: ".".to_string(),
            pattern: "*.rs".to_string(),
        };

        assert_eq!(
            op.to_shell_fragment(),
            "find . -name '*.rs' -type f -exec ls -lh {} +"
        );
    }

    #[test]
    fn test_sort_files_fragment_ascending() {
        let op = BaseOperation::SortFiles {
            field: Field::Size,
            direction: Direction::Ascending,
        };

        assert_eq!(op.to_shell_fragment(), "sort -k5 -h");
    }

    #[test]
    fn test_sort_files_fragment_descending() {
        let op = BaseOperation::SortFiles {
            field: Field::Size,
            direction: Direction::Descending,
        };

        assert_eq!(op.to_shell_fragment(), "sort -k5 -hr");
    }

    #[test]
    fn test_limit_files_fragment() {
        let op = BaseOperation::LimitFiles { count: 10 };
        assert_eq!(op.to_shell_fragment(), "head -n 10");
    }

    #[test]
    fn test_disk_usage_fragment() {
        let op = BaseOperation::DiskUsage {
            path: "/var/log".to_string(),
        };

        assert_eq!(op.to_shell_fragment(), "du -sh /var/log/*");
    }

    #[test]
    fn test_sort_files_with_default_field() {
        // Field::Default 用于不指定列的排序（如 du 输出）
        let op = BaseOperation::SortFiles {
            field: Field::Default,
            direction: Direction::Descending,
        };

        assert_eq!(op.to_shell_fragment(), "sort -hr");
    }

    #[test]
    fn test_sort_files_with_default_field_ascending() {
        let op = BaseOperation::SortFiles {
            field: Field::Default,
            direction: Direction::Ascending,
        };

        assert_eq!(op.to_shell_fragment(), "sort -h");
    }

    #[test]
    fn test_operations_are_combinable() {
        // 验证操作可以独立创建和组合
        let find = BaseOperation::FindFiles {
            path: ".".to_string(),
            pattern: "*.rs".to_string(),
        };

        let sort = BaseOperation::SortFiles {
            field: Field::Size,
            direction: Direction::Descending,
        };

        let limit = BaseOperation::LimitFiles { count: 5 };

        // 所有操作都可以独立存在
        assert!(find.needs_pipe());
        assert!(sort.needs_pipe());
        assert!(limit.needs_pipe());
    }

    #[test]
    fn test_disk_usage_pipeline_composition() {
        // Phase 6.3 Step 2: 验证 DiskUsage + SortFiles(Default) + LimitFiles
        let du = BaseOperation::DiskUsage {
            path: ".".to_string(),
        };
        let sort = BaseOperation::SortFiles {
            field: Field::Default,
            direction: Direction::Descending,
        };
        let limit = BaseOperation::LimitFiles { count: 10 };

        assert_eq!(du.to_shell_fragment(), "du -sh ./*");
        assert_eq!(sort.to_shell_fragment(), "sort -hr");
        assert_eq!(limit.to_shell_fragment(), "head -n 10");

        // 组合应该生成: du -sh ./* | sort -hr | head -n 10
    }
}
