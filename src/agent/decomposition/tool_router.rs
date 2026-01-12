//! Tool Router - 智能工具路由器
//!
//! 将 Intent 映射到专用工具，提供比 shell_execute 更精准的参数验证和执行语义。
//!
//! # 设计哲学
//!
//! - **保守映射**: 只映射最常用且映射最直接的 Intent
//! - **回退机制**: 未映射的 Intent 返回 None，由 IntentRouter 回退到 shell_execute
//! - **渐进增强**: 后续版本可逐步扩展映射表
//!
//! # 示例
//!
//! ```ignore
//! use realconsole::agent::decomposition::tool_router::ToolRouter;
//!
//! let router = ToolRouter::new();
//!
//! // 尝试路由 Intent
//! if let Some((tool_name, params)) = router.route(&intent_match) {
//!     println!("映射到专用工具: {}", tool_name);
//! } else {
//!     println!("回退到 shell_execute");
//! }
//! ```

use crate::dsl::intent::{EntityType, IntentMatch};
use serde_json::{json, Value as JsonValue};

/// Intent → Tool 映射关系
///
/// 定义了如何将特定的 Intent 映射到专用工具，包括参数提取逻辑。
pub struct ToolMapping {
    /// Intent 名称（如 "list_directory"）
    pub intent_name: String,

    /// 目标工具名称（如 "list_dir"）
    pub tool_name: String,

    /// 参数提取器：从 IntentMatch 中提取工具参数
    ///
    /// # 返回
    /// - Ok(JsonValue) - 成功提取的参数
    /// - Err(String) - 提取失败（回退到 shell_execute）
    pub param_extractor: fn(&IntentMatch) -> Result<JsonValue, String>,
}

impl std::fmt::Debug for ToolMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolMapping")
            .field("intent_name", &self.intent_name)
            .field("tool_name", &self.tool_name)
            .field("param_extractor", &"<function>")
            .finish()
    }
}

/// 工具路由器
///
/// 维护 Intent 到工具的映射表，并提供路由功能。
///
/// # v1.32.0 映射表
///
/// | Intent | 工具 | 参数 |
/// |--------|------|------|
/// | list_directory | list_dir | path (from entity or default ".") |
/// | count_python_lines | count_code_lines | directory=".", extension="py" |
///
/// # 示例
///
/// ```ignore
/// use realconsole::agent::decomposition::tool_router::ToolRouter;
///
/// let router = ToolRouter::new();
/// let intent_match = /* ... */;
///
/// match router.route(&intent_match) {
///     Some((tool, params)) => {
///         println!("🎯 映射到专用工具: {} {:?}", tool, params);
///     }
///     None => {
///         println!("🐚 回退到 shell_execute");
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ToolRouter {
    mappings: Vec<ToolMapping>,
}

impl ToolRouter {
    /// 创建新的工具路由器（预加载映射表）
    pub fn new() -> Self {
        Self {
            mappings: Self::build_mappings(),
        }
    }

    /// 路由 Intent 到工具调用
    ///
    /// # 参数
    /// - intent_match: Intent 匹配结果
    ///
    /// # 返回
    /// - Some((tool_name, params)) - 成功映射到专用工具
    /// - None - 未找到映射（应回退到 shell_execute）
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let router = ToolRouter::new();
    ///
    /// if let Some((tool, params)) = router.route(&intent_match) {
    ///     println!("使用专用工具: {}", tool);
    /// }
    /// ```
    pub fn route(&self, intent_match: &IntentMatch) -> Option<(String, JsonValue)> {
        // 遍历映射表
        for mapping in &self.mappings {
            if mapping.intent_name == intent_match.intent.name {
                // 尝试提取参数
                match (mapping.param_extractor)(intent_match) {
                    Ok(params) => {
                        eprintln!(
                            "🎯 [ToolRouter] {} → {} {:?}",
                            intent_match.intent.name, mapping.tool_name, params
                        );
                        return Some((mapping.tool_name.clone(), params));
                    }
                    Err(e) => {
                        eprintln!("⚠️ [ToolRouter] 参数提取失败: {}", e);
                        return None;
                    }
                }
            }
        }

        // 未找到映射
        None
    }

    /// 构建映射表（v1.35.0 - 渐进增强：5 个映射）
    ///
    /// # 设计原则
    ///
    /// 1. **优先级**: 只映射最常用且映射最直接的 Intent
    /// 2. **简单性**: 参数提取逻辑简单明确
    /// 3. **可扩展**: 后续版本可逐步添加更多映射
    ///
    /// # 当前映射（v1.35.0）
    ///
    /// - list_directory → list_dir (文件操作，极高频)
    /// - count_python_lines → count_code_lines (代码统计，高频)
    /// - find_files_by_name → find_file (文件查找，高频)
    /// - grep_pattern → search_text (文本搜索，高频)
    /// - count_files → count_files_tool (文件统计，中-高频)
    fn build_mappings() -> Vec<ToolMapping> {
        vec![
            // ===== v1.32.0 映射 =====
            // 映射 1: list_directory → list_dir
            ToolMapping {
                intent_name: "list_directory".to_string(),
                tool_name: "list_dir".to_string(),
                param_extractor: extract_list_directory_params,
            },
            // 映射 2: count_python_lines → count_code_lines
            ToolMapping {
                intent_name: "count_python_lines".to_string(),
                tool_name: "count_code_lines".to_string(),
                param_extractor: extract_count_python_lines_params,
            },
            // ===== v1.33.0 新增映射 =====
            // 映射 3: find_files_by_name → find_file
            ToolMapping {
                intent_name: "find_files_by_name".to_string(),
                tool_name: "find_file".to_string(),
                param_extractor: extract_find_files_by_name_params,
            },
            // ===== v1.34.0 新增映射 =====
            // 映射 4: grep_pattern → search_text
            ToolMapping {
                intent_name: "grep_pattern".to_string(),
                tool_name: "search_text".to_string(),
                param_extractor: extract_grep_pattern_params,
            },
            // ===== v1.35.0 新增映射 =====
            // 映射 5: count_files → count_files_tool
            ToolMapping {
                intent_name: "count_files".to_string(),
                tool_name: "count_files_tool".to_string(),
                param_extractor: extract_count_files_params,
            },
        ]
    }
}

impl Default for ToolRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ===== 参数提取器实现 =====

/// 提取 list_directory Intent 的参数
///
/// # 策略
/// 1. 从 extracted_entities 中查找 Path 实体
/// 2. 如果找到，使用实体中的路径
/// 3. 如果未找到，使用默认值 "."（当前目录）
fn extract_list_directory_params(intent_match: &IntentMatch) -> Result<JsonValue, String> {
    // 尝试从实体中提取路径
    let path = intent_match
        .extracted_entities
        .values()
        .find_map(|entity| {
            if let EntityType::Path(p) = entity {
                Some(p.as_str())
            } else {
                None
            }
        })
        .unwrap_or("."); // 默认为当前目录

    Ok(json!({ "path": path }))
}

/// 提取 count_python_lines Intent 的参数
///
/// # 策略
/// 1. directory 固定为 "."（当前目录）
/// 2. extension 固定为 "py"（Python 文件）
///
/// # 扩展
/// 未来可以从实体中提取 directory 和 extension
fn extract_count_python_lines_params(_intent_match: &IntentMatch) -> Result<JsonValue, String> {
    Ok(json!({
        "directory": ".",
        "extension": "py"
    }))
}

/// 提取 find_files_by_name Intent 的参数 (v1.33.0)
///
/// # 策略
/// 1. 从实体中提取文件类型（FileType），转换为通配符模式 (*.ext)
/// 2. 如果实体中有 Custom("pattern", value)，直接使用该模式
/// 3. directory 默认为 "."（当前目录）
/// 4. max_depth 默认为 10
/// 5. max_results 默认为 100
fn extract_find_files_by_name_params(intent_match: &IntentMatch) -> Result<JsonValue, String> {
    // 尝试从实体中提取文件模式
    let pattern = intent_match
        .extracted_entities
        .values()
        .find_map(|entity| {
            match entity {
                EntityType::FileType(ext) => {
                    // 文件类型实体，如 "py" → "*.py"
                    Some(format!("*.{}", ext))
                }
                EntityType::Custom(name, value) if name == "pattern" => {
                    // 自定义模式实体
                    Some(value.clone())
                }
                _ => None,
            }
        })
        .ok_or("未能从 Intent 中提取文件模式")?;

    Ok(json!({
        "directory": ".",
        "pattern": pattern,
        "max_depth": 10,
        "max_results": 100
    }))
}

/// 提取 grep_pattern Intent 的参数 (v1.34.0)
///
/// # 策略
/// 1. 从实体中提取搜索模式（Custom("pattern", value)）
/// 2. directory 默认为 "."（当前目录）
/// 3. file_pattern 默认为 "*"（所有文件），如果有 FileType 实体则使用 "*.ext"
/// 4. case_insensitive 默认为 false
/// 5. max_results 默认为 100
///
/// # 示例
/// - 输入: "搜索 TODO 注释"
/// - 提取: Custom("pattern", "TODO")
/// - 输出: {"pattern": "TODO", "directory": ".", "file_pattern": "*", ...}
fn extract_grep_pattern_params(intent_match: &IntentMatch) -> Result<JsonValue, String> {
    // 尝试从实体中提取搜索模式
    let pattern = intent_match
        .extracted_entities
        .values()
        .find_map(|entity| {
            if let EntityType::Custom(name, value) = entity {
                if name == "pattern" {
                    Some(value.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or("未能从 Intent 中提取搜索模式")?;

    // 提取路径（可选）
    let directory = intent_match
        .extracted_entities
        .values()
        .find_map(|entity| {
            if let EntityType::Path(path) = entity {
                Some(path.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| ".".to_string());

    // 提取文件类型（可选）
    let file_pattern = intent_match
        .extracted_entities
        .values()
        .find_map(|entity| {
            if let EntityType::FileType(ext) = entity {
                Some(format!("*.{}", ext))
            } else {
                None
            }
        })
        .unwrap_or_else(|| "*".to_string());

    Ok(json!({
        "pattern": pattern,
        "directory": directory,
        "file_pattern": file_pattern,
        "case_insensitive": false,
        "max_results": 100
    }))
}

/// 提取 count_files Intent 的参数 (v1.35.0)
///
/// # 策略
/// 1. 从实体中提取路径（Path），默认为 "."
/// 2. 从实体中提取文件类型（FileType），转换为 "*.ext"，默认为 "*"
/// 3. max_depth 默认为 10
/// 4. show_breakdown 默认为 false
///
/// # 示例
/// - 输入: "统计 Python 文件数量"
/// - 提取: FileType("py")
/// - 输出: {"directory": ".", "file_pattern": "*.py", ...}
fn extract_count_files_params(intent_match: &IntentMatch) -> Result<JsonValue, String> {
    // 提取路径（可选）
    let directory = intent_match
        .extracted_entities
        .values()
        .find_map(|entity| {
            if let EntityType::Path(path) = entity {
                Some(path.clone())
            } else {
                None
            }
        })
        .unwrap_or_else(|| ".".to_string());

    // 提取文件类型（可选）
    let file_pattern = intent_match
        .extracted_entities
        .values()
        .find_map(|entity| {
            if let EntityType::FileType(ext) = entity {
                Some(format!("*.{}", ext))
            } else {
                None
            }
        })
        .unwrap_or_else(|| "*".to_string());

    Ok(json!({
        "directory": directory,
        "file_pattern": file_pattern,
        "max_depth": 10,
        "show_breakdown": false
    }))
}

// ===== 单元测试 =====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::intent::{Intent, IntentDomain};
    use std::collections::HashMap;

    /// 辅助函数：创建测试用的 IntentMatch
    fn create_test_match(intent_name: &str, entities: HashMap<String, EntityType>) -> IntentMatch {
        IntentMatch {
            intent: Intent {
                name: intent_name.to_string(),
                domain: IntentDomain::FileOps,
                keywords: vec![],
                patterns: vec![],
                entities: HashMap::new(),
                confidence_threshold: 0.5,
            },
            confidence: 0.85,
            matched_keywords: vec![],
            extracted_entities: entities,
        }
    }

    #[test]
    fn test_router_creation() {
        let router = ToolRouter::new();
        assert_eq!(router.mappings.len(), 5); // v1.35.0: 5 个映射
    }

    #[test]
    fn test_route_list_directory_default_path() {
        let router = ToolRouter::new();

        // 不带路径实体的 list_directory
        let intent_match = create_test_match("list_directory", HashMap::new());

        let result = router.route(&intent_match);
        assert!(result.is_some());

        let (tool, params) = result.unwrap();
        assert_eq!(tool, "list_dir");
        assert_eq!(params["path"], ".");
    }

    #[test]
    fn test_route_list_directory_with_path() {
        let router = ToolRouter::new();

        // 带路径实体的 list_directory
        let mut entities = HashMap::new();
        entities.insert("path".to_string(), EntityType::Path("/tmp".to_string()));

        let intent_match = create_test_match("list_directory", entities);

        let result = router.route(&intent_match);
        assert!(result.is_some());

        let (tool, params) = result.unwrap();
        assert_eq!(tool, "list_dir");
        assert_eq!(params["path"], "/tmp");
    }

    #[test]
    fn test_route_count_python_lines() {
        let router = ToolRouter::new();

        let intent_match = create_test_match("count_python_lines", HashMap::new());

        let result = router.route(&intent_match);
        assert!(result.is_some());

        let (tool, params) = result.unwrap();
        assert_eq!(tool, "count_code_lines");
        assert_eq!(params["directory"], ".");
        assert_eq!(params["extension"], "py");
    }

    #[test]
    fn test_route_unmapped_intent() {
        let router = ToolRouter::new();

        // 未映射的 Intent（如 check_memory_usage）
        let intent_match = create_test_match("check_memory_usage", HashMap::new());

        let result = router.route(&intent_match);
        assert!(result.is_none()); // 应该返回 None（回退）
    }

    #[test]
    fn test_param_extractor_list_directory() {
        // 测试无实体情况
        let intent_match = create_test_match("list_directory", HashMap::new());
        let result = extract_list_directory_params(&intent_match);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["path"], ".");

        // 测试有实体情况
        let mut entities = HashMap::new();
        entities.insert("path".to_string(), EntityType::Path("/tmp".to_string()));
        let intent_match = create_test_match("list_directory", entities);
        let result = extract_list_directory_params(&intent_match);
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["path"], "/tmp");
    }

    #[test]
    fn test_param_extractor_count_python_lines() {
        let intent_match = create_test_match("count_python_lines", HashMap::new());
        let result = extract_count_python_lines_params(&intent_match);
        assert!(result.is_ok());

        let params = result.unwrap();
        assert_eq!(params["directory"], ".");
        assert_eq!(params["extension"], "py");
    }
}
