//! Intent → Pipeline 转换桥梁
//!
//! **Phase 6.3 Step 1**: 将 Intent DSL 与 Pipeline DSL 连接
//!
//! # 设计哲学
//!
//! "桥"（Bridge）- 连接两个世界：
//! - Intent DSL: 用户意图识别（What）
//! - Pipeline DSL: 命令生成执行（How）
//!
//! # 架构
//!
//! ```text
//! 用户输入 → Intent 匹配 → 实体提取 → [转换桥梁] → ExecutionPlan → Shell命令
//!            ↑________________Intent DSL________________↑
//!                                        ↑________Pipeline DSL________↑
//! ```

use crate::dsl::intent::{EntityType, IntentMatch};
use crate::dsl::pipeline::{BaseOperation, Direction, ExecutionPlan, Field};
use std::collections::HashMap;

/// Intent → Pipeline 转换器
///
/// 将 Intent 匹配结果（含实体）转换为 ExecutionPlan
///
/// # Example
///
/// ```rust
/// use realconsole::dsl::intent::pipeline_bridge::IntentToPipeline;
///
/// let converter = IntentToPipeline::new();
/// let plan = converter.convert(&intent_match, &entities);
/// ```
#[derive(Debug)]
pub struct IntentToPipeline {
    /// 是否启用 Pipeline DSL（配置开关）
    enabled: bool,
}

impl IntentToPipeline {
    /// 创建新的转换器
    pub fn new() -> Self {
        Self { enabled: true }
    }

    /// 创建转换器（可配置是否启用）
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    /// 转换 Intent 匹配结果为 ExecutionPlan
    ///
    /// # Arguments
    ///
    /// * `intent_match` - Intent 匹配结果
    /// * `entities` - 提取的实体参数
    ///
    /// # Returns
    ///
    /// - `Some(ExecutionPlan)` - 成功转换
    /// - `None` - 该 Intent 不支持 Pipeline 或转换失败
    pub fn convert(
        &self,
        intent_match: &IntentMatch,
        entities: &HashMap<String, EntityType>,
    ) -> Option<ExecutionPlan> {
        if !self.enabled {
            return None;
        }

        // 根据 Intent 名称分发到具体的转换函数
        match intent_match.intent.name.as_str() {
            "find_files_by_size" => self.convert_find_files_by_size(entities),
            "find_recent_files" => self.convert_find_recent_files(entities),
            "check_disk_usage" => self.convert_check_disk_usage(entities),
            // 未来扩展其他 Intent
            _ => None,
        }
    }

    /// 转换 find_files_by_size Intent
    ///
    /// **Pipeline 结构**：
    /// 1. FindFiles: 查找文件（按类型过滤）
    /// 2. SortFiles: 按体积排序（升序/降序）
    /// 3. LimitFiles: 限制结果数量
    ///
    /// **哲学体现**：
    /// - 象（不变）：3个基础操作的组合
    /// - 爻（变化）：direction 参数（Ascending ⇄ Descending）
    fn convert_find_files_by_size(
        &self,
        entities: &HashMap<String, EntityType>,
    ) -> Option<ExecutionPlan> {
        // 提取参数
        let path = self.extract_path(entities)?;
        let pattern = self.extract_file_pattern(entities)?;
        let direction = self.extract_sort_direction(entities)?;
        let limit = self.extract_limit(entities)?;

        // 构建 ExecutionPlan
        let plan = ExecutionPlan::new()
            .with_operation(BaseOperation::FindFiles { path, pattern })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Size,
                direction,
            })
            .with_operation(BaseOperation::LimitFiles { count: limit });

        Some(plan)
    }

    /// 转换 find_recent_files Intent (Phase 6.3 Step 2)
    ///
    /// **Pipeline 结构**：
    /// 1. FindFiles: 查找文件（按类型过滤）
    /// 2. SortFiles: 按时间排序（默认降序 - 最新的在前）
    /// 3. LimitFiles: 限制结果数量
    ///
    /// **与 find_files_by_size 的对比**：
    /// - 相同点：操作结构完全相同（象不变）
    /// - 不同点：排序字段从 Size 变为 Time（爻之变）
    ///
    /// **哲学体现**：
    /// - 象（不变）：FindFiles + SortFiles + LimitFiles
    /// - 爻（变化）：field 参数（Size ⇄ Time）
    fn convert_find_recent_files(
        &self,
        entities: &HashMap<String, EntityType>,
    ) -> Option<ExecutionPlan> {
        // 提取参数（复用相同的提取方法）
        let path = self.extract_path(entities)?;
        let pattern = self.extract_file_pattern(entities)?;
        let limit = self.extract_limit(entities)?;

        // 注意：时间排序默认使用降序（最新的在前）
        // 不需要从用户输入提取 sort_order，因为 "最近" 隐含了降序
        let direction = Direction::Descending;

        // 构建 ExecutionPlan
        let plan = ExecutionPlan::new()
            .with_operation(BaseOperation::FindFiles { path, pattern })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Time,  // 关键区别：按时间排序
                direction,
            })
            .with_operation(BaseOperation::LimitFiles { count: limit });

        Some(plan)
    }

    /// 转换 check_disk_usage Intent (Phase 6.3 Step 2)
    ///
    /// **Pipeline 结构**：
    /// 1. DiskUsage: 检查磁盘使用
    /// 2. SortFiles: 按大小排序（默认降序 - 最大的在前）
    /// 3. LimitFiles: 限制结果数量
    ///
    /// **与 find_files_by_size 的对比**：
    /// - 不同点：基础命令从 FindFiles 变为 DiskUsage
    /// - 不同点：排序字段使用 Field::Default（du 输出第一列就是大小）
    /// - 相同点：都是 3 操作结构（象不变）
    ///
    /// **哲学体现**：
    /// - 象（不变）：<基础操作> + SortFiles + LimitFiles
    /// - 爻（变化）：基础操作类型（FindFiles ⇄ DiskUsage）
    fn convert_check_disk_usage(
        &self,
        entities: &HashMap<String, EntityType>,
    ) -> Option<ExecutionPlan> {
        // 提取参数
        let path = self.extract_path(entities)?;
        let limit = self.extract_limit(entities)?;

        // 磁盘使用统计总是按大小降序（最大的在前）
        let direction = Direction::Descending;

        // 构建 ExecutionPlan
        let plan = ExecutionPlan::new()
            .with_operation(BaseOperation::DiskUsage { path })
            .with_operation(BaseOperation::SortFiles {
                field: Field::Default, // du 输出第一列就是大小，不需要指定列
                direction,
            })
            .with_operation(BaseOperation::LimitFiles { count: limit });

        Some(plan)
    }

    // ========== 实体提取辅助方法 ==========

    /// 提取路径参数
    fn extract_path(&self, entities: &HashMap<String, EntityType>) -> Option<String> {
        match entities.get("path") {
            Some(EntityType::Path(path)) => Some(path.clone()),
            _ => Some(".".to_string()), // 默认当前目录
        }
    }

    /// 提取文件模式（类型 → glob pattern）
    fn extract_file_pattern(&self, entities: &HashMap<String, EntityType>) -> Option<String> {
        match entities.get("ext") {
            Some(EntityType::FileType(ext)) => Some(format!("*.{}", ext)),
            _ => Some("*".to_string()), // 默认所有文件
        }
    }

    /// 提取排序方向
    ///
    /// **核心转换**：将 shell 标志（-hr/-h）转换为 Direction 枚举
    fn extract_sort_direction(&self, entities: &HashMap<String, EntityType>) -> Option<Direction> {
        match entities.get("sort_order") {
            Some(EntityType::Custom(type_name, value)) if type_name == "sort" => {
                match value.as_str() {
                    "-hr" => Some(Direction::Descending), // 降序（最大）
                    "-h" => Some(Direction::Ascending),   // 升序（最小）
                    _ => Some(Direction::Descending),     // 默认降序
                }
            }
            _ => Some(Direction::Descending), // 默认降序
        }
    }

    /// 提取限制数量
    fn extract_limit(&self, entities: &HashMap<String, EntityType>) -> Option<usize> {
        match entities.get("limit") {
            Some(EntityType::Number(n)) => Some(*n as usize),
            _ => Some(10), // 默认10个
        }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for IntentToPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::intent::{Intent, IntentDomain};

    fn create_test_intent_match() -> IntentMatch {
        let intent = Intent::new(
            "find_files_by_size",
            IntentDomain::FileOps,
            vec!["查找".to_string()],
            vec![],
            0.7,
        );

        IntentMatch {
            intent,
            confidence: 1.0,
            matched_keywords: vec!["查找".to_string()],
            extracted_entities: HashMap::new(),
        }
    }

    #[test]
    fn test_converter_creation() {
        let converter = IntentToPipeline::new();
        assert!(converter.is_enabled());

        let converter_disabled = IntentToPipeline::with_enabled(false);
        assert!(!converter_disabled.is_enabled());
    }

    #[test]
    fn test_convert_find_files_by_size_descending() {
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match();

        let mut entities = HashMap::new();
        entities.insert("path".to_string(), EntityType::Path(".".to_string()));
        entities.insert("ext".to_string(), EntityType::FileType("rs".to_string()));
        entities.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-hr".to_string()),
        );
        entities.insert("limit".to_string(), EntityType::Number(5.0));

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_some());
        let plan = plan.unwrap();

        // 验证操作数量
        assert_eq!(plan.len(), 3);

        // 验证生成的命令
        let command = plan.to_shell_command();
        assert!(command.contains("find . -name '*.rs'"));
        assert!(command.contains("sort -k5 -hr")); // 降序
        assert!(command.contains("head -n 5"));
    }

    #[test]
    fn test_convert_find_files_by_size_ascending() {
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match();

        let mut entities = HashMap::new();
        entities.insert("path".to_string(), EntityType::Path(".".to_string()));
        entities.insert("ext".to_string(), EntityType::FileType("rs".to_string()));
        entities.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-h".to_string()), // 升序
        );
        entities.insert("limit".to_string(), EntityType::Number(1.0));

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_some());
        let plan = plan.unwrap();

        // 验证生成的命令
        let command = plan.to_shell_command();
        assert!(command.contains("sort -k5 -h")); // 升序
        assert!(!command.contains("-hr"));
        assert!(command.contains("head -n 1"));
    }

    #[test]
    fn test_convert_with_default_values() {
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match();

        // 空实体 - 应该使用默认值
        let entities = HashMap::new();

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_some());
        let plan = plan.unwrap();

        let command = plan.to_shell_command();
        assert!(command.contains("find . -name '*'")); // 默认路径和模式
        assert!(command.contains("sort -k5 -hr")); // 默认降序
        assert!(command.contains("head -n 10")); // 默认10个
    }

    #[test]
    fn test_convert_unsupported_intent() {
        let converter = IntentToPipeline::new();

        let intent = Intent::new(
            "unknown_intent",
            IntentDomain::FileOps,
            vec![],
            vec![],
            0.5,
        );

        let intent_match = IntentMatch {
            intent,
            confidence: 1.0,
            matched_keywords: vec![],
            extracted_entities: HashMap::new(),
        };

        let entities = HashMap::new();
        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_none(), "不支持的 Intent 应该返回 None");
    }

    #[test]
    fn test_converter_disabled() {
        let converter = IntentToPipeline::with_enabled(false);
        let intent_match = create_test_intent_match();

        let mut entities = HashMap::new();
        entities.insert("path".to_string(), EntityType::Path(".".to_string()));

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_none(), "禁用时应该返回 None");
    }

    #[test]
    fn test_extract_path() {
        let converter = IntentToPipeline::new();

        let mut entities = HashMap::new();
        entities.insert(
            "path".to_string(),
            EntityType::Path("./src".to_string()),
        );

        let path = converter.extract_path(&entities);
        assert_eq!(path, Some("./src".to_string()));

        // 无路径时使用默认值
        let empty_entities = HashMap::new();
        let default_path = converter.extract_path(&empty_entities);
        assert_eq!(default_path, Some(".".to_string()));
    }

    #[test]
    fn test_extract_file_pattern() {
        let converter = IntentToPipeline::new();

        let mut entities = HashMap::new();
        entities.insert("ext".to_string(), EntityType::FileType("py".to_string()));

        let pattern = converter.extract_file_pattern(&entities);
        assert_eq!(pattern, Some("*.py".to_string()));

        // 无类型时使用默认值
        let empty_entities = HashMap::new();
        let default_pattern = converter.extract_file_pattern(&empty_entities);
        assert_eq!(default_pattern, Some("*".to_string()));
    }

    #[test]
    fn test_extract_sort_direction() {
        let converter = IntentToPipeline::new();

        // 测试降序
        let mut entities = HashMap::new();
        entities.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-hr".to_string()),
        );
        let dir = converter.extract_sort_direction(&entities);
        assert_eq!(dir, Some(Direction::Descending));

        // 测试升序
        entities.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-h".to_string()),
        );
        let dir = converter.extract_sort_direction(&entities);
        assert_eq!(dir, Some(Direction::Ascending));

        // 默认值
        let empty_entities = HashMap::new();
        let default_dir = converter.extract_sort_direction(&empty_entities);
        assert_eq!(default_dir, Some(Direction::Descending));
    }

    #[test]
    fn test_extract_limit() {
        let converter = IntentToPipeline::new();

        let mut entities = HashMap::new();
        entities.insert("limit".to_string(), EntityType::Number(20.0));

        let limit = converter.extract_limit(&entities);
        assert_eq!(limit, Some(20));

        // 默认值
        let empty_entities = HashMap::new();
        let default_limit = converter.extract_limit(&empty_entities);
        assert_eq!(default_limit, Some(10));
    }

    #[test]
    fn test_philosophy_demonstration() {
        // 哲学验证：象不变，爻可变
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match();

        // 最大文件（降序）
        let mut entities_largest = HashMap::new();
        entities_largest.insert("ext".to_string(), EntityType::FileType("rs".to_string()));
        entities_largest.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-hr".to_string()),
        );

        // 最小文件（升序）
        let mut entities_smallest = HashMap::new();
        entities_smallest.insert("ext".to_string(), EntityType::FileType("rs".to_string()));
        entities_smallest.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-h".to_string()),
        );

        let plan_largest = converter.convert(&intent_match, &entities_largest).unwrap();
        let plan_smallest = converter
            .convert(&intent_match, &entities_smallest)
            .unwrap();

        // 验证：结构相同（都是3个操作）
        assert_eq!(plan_largest.len(), plan_smallest.len());
        assert_eq!(plan_largest.len(), 3);

        // 验证：命令不同（只有排序方向不同）
        let cmd_largest = plan_largest.to_shell_command();
        let cmd_smallest = plan_smallest.to_shell_command();

        assert!(cmd_largest.contains("-hr"));
        assert!(cmd_smallest.contains("-h"));
        assert!(!cmd_smallest.contains("-hr"));

        // 哲学体现：
        // - 象（不变）：ExecutionPlan 的3个操作
        // - 爻（变化）：Direction::Descending ⇄ Direction::Ascending
        // - 结果：只有一个参数的差异！
    }

    // ========== Phase 6.3 Step 2: find_recent_files 测试 ==========

    fn create_test_intent_match_recent() -> IntentMatch {
        let intent = Intent::new(
            "find_recent_files",
            IntentDomain::FileOps,
            vec!["查找".to_string(), "最近".to_string()],
            vec![],
            0.65,
        );

        IntentMatch {
            intent,
            confidence: 1.0,
            matched_keywords: vec!["最近".to_string()],
            extracted_entities: HashMap::new(),
        }
    }

    #[test]
    fn test_convert_find_recent_files() {
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match_recent();

        let mut entities = HashMap::new();
        entities.insert("path".to_string(), EntityType::Path(".".to_string()));
        entities.insert("ext".to_string(), EntityType::FileType("md".to_string()));
        entities.insert("limit".to_string(), EntityType::Number(5.0));

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_some());
        let plan = plan.unwrap();

        // 验证操作数量
        assert_eq!(plan.len(), 3);

        // 验证生成的命令
        let command = plan.to_shell_command();
        assert!(command.contains("find . -name '*.md'"));
        assert!(command.contains("sort -k6 -hr")); // 按时间降序
        assert!(command.contains("head -n 5"));
    }

    #[test]
    fn test_convert_find_recent_files_default_values() {
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match_recent();

        // 空实体 - 应该使用默认值
        let entities = HashMap::new();

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_some());
        let plan = plan.unwrap();

        let command = plan.to_shell_command();
        assert!(command.contains("find . -name '*'")); // 默认路径和模式
        assert!(command.contains("sort -k6 -hr")); // 默认降序
        assert!(command.contains("head -n 10")); // 默认10个
    }

    #[test]
    fn test_philosophy_size_vs_time() {
        // 哲学验证：相同的操作结构，不同的排序字段
        let converter = IntentToPipeline::new();

        // 按大小排序
        let intent_size = create_test_intent_match();
        let mut entities_size = HashMap::new();
        entities_size.insert("ext".to_string(), EntityType::FileType("rs".to_string()));
        entities_size.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-hr".to_string()),
        );

        // 按时间排序
        let intent_time = create_test_intent_match_recent();
        let mut entities_time = HashMap::new();
        entities_time.insert("ext".to_string(), EntityType::FileType("rs".to_string()));

        let plan_size = converter.convert(&intent_size, &entities_size).unwrap();
        let plan_time = converter.convert(&intent_time, &entities_time).unwrap();

        // 验证：结构相同（都是3个操作）
        assert_eq!(plan_size.len(), plan_time.len());
        assert_eq!(plan_size.len(), 3);

        // 验证：排序字段不同
        let cmd_size = plan_size.to_shell_command();
        let cmd_time = plan_time.to_shell_command();

        assert!(cmd_size.contains("sort -k5")); // Size: column 5
        assert!(cmd_time.contains("sort -k6")); // Time: column 6

        // 哲学体现：
        // - 象（不变）：3个操作的组合结构
        // - 爻（变化）：field 参数（Size ⇄ Time）
        // - 结果：相同的结构，不同的语义！
    }

    // ========== Phase 6.3 Step 2: check_disk_usage 测试 ==========

    fn create_test_intent_match_disk_usage() -> IntentMatch {
        let intent = Intent::new(
            "check_disk_usage",
            IntentDomain::DiagnosticOps,
            vec!["检查".to_string(), "磁盘".to_string()],
            vec![],
            0.7,
        );

        IntentMatch {
            intent,
            confidence: 1.0,
            matched_keywords: vec!["磁盘".to_string()],
            extracted_entities: HashMap::new(),
        }
    }

    #[test]
    fn test_convert_check_disk_usage() {
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match_disk_usage();

        let mut entities = HashMap::new();
        entities.insert("path".to_string(), EntityType::Path("/var/log".to_string()));
        entities.insert("limit".to_string(), EntityType::Number(10.0));

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_some());
        let plan = plan.unwrap();

        // 验证操作数量
        assert_eq!(plan.len(), 3);

        // 验证生成的命令
        let command = plan.to_shell_command();
        assert!(command.contains("du -sh /var/log/*"));
        assert!(command.contains("sort -hr")); // Default field, 降序
        assert!(command.contains("head -n 10"));
    }

    #[test]
    fn test_convert_check_disk_usage_default_values() {
        let converter = IntentToPipeline::new();
        let intent_match = create_test_intent_match_disk_usage();

        // 空实体 - 应该使用默认值
        let entities = HashMap::new();

        let plan = converter.convert(&intent_match, &entities);

        assert!(plan.is_some());
        let plan = plan.unwrap();

        let command = plan.to_shell_command();
        assert!(command.contains("du -sh ./*")); // 默认路径
        assert!(command.contains("sort -hr")); // 默认降序
        assert!(command.contains("head -n 10")); // 默认10个
    }

    #[test]
    fn test_philosophy_find_vs_du() {
        // 哲学验证：相同的操作模式，不同的基础命令
        let converter = IntentToPipeline::new();

        // FindFiles + SortFiles + LimitFiles
        let intent_find = create_test_intent_match();
        let mut entities_find = HashMap::new();
        entities_find.insert("path".to_string(), EntityType::Path(".".to_string()));
        entities_find.insert("ext".to_string(), EntityType::FileType("rs".to_string()));
        entities_find.insert(
            "sort_order".to_string(),
            EntityType::Custom("sort".to_string(), "-hr".to_string()),
        );
        entities_find.insert("limit".to_string(), EntityType::Number(5.0));

        // DiskUsage + SortFiles + LimitFiles
        let intent_du = create_test_intent_match_disk_usage();
        let mut entities_du = HashMap::new();
        entities_du.insert("path".to_string(), EntityType::Path(".".to_string()));
        entities_du.insert("limit".to_string(), EntityType::Number(5.0));

        let plan_find = converter.convert(&intent_find, &entities_find).unwrap();
        let plan_du = converter.convert(&intent_du, &entities_du).unwrap();

        // 验证：结构相同（都是3个操作）
        assert_eq!(plan_find.len(), plan_du.len());
        assert_eq!(plan_find.len(), 3);

        // 验证：基础命令不同
        let cmd_find = plan_find.to_shell_command();
        let cmd_du = plan_du.to_shell_command();

        assert!(cmd_find.contains("find"));
        assert!(cmd_du.contains("du -sh"));

        // 验证：排序列不同
        assert!(cmd_find.contains("sort -k5")); // FindFiles: 指定列
        assert!(cmd_du.contains("sort -hr"));    // DiskUsage: 默认第一列
        assert!(!cmd_du.contains("sort -k"));

        // 哲学体现：
        // - 象（不变）：<基础操作> + SortFiles + LimitFiles 结构
        // - 爻（变化）：基础操作（FindFiles ⇄ DiskUsage）
        // - 爻（变化）：排序字段（Field::Size ⇄ Field::Default）
        // - 结果：相同的模式，不同的实现！
    }
}
