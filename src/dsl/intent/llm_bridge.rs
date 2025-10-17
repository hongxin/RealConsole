//! LLM → Pipeline 桥梁（Phase 7）
//!
//! **核心思想**：让 LLM 参与意图理解，生成结构化的执行计划
//!
//! # 架构
//!
//! ```text
//! 用户输入
//!   ↓
//! LLM 理解（System Prompt）
//!   ↓
//! 结构化 JSON
//!   ↓
//! ExecutionPlan（Pipeline DSL）
//!   ↓
//! Shell 命令
//! ```
//!
//! # 设计哲学
//!
//! - **LLM 是核心理解器**，而非事后检查器
//! - **结构化输出 + 安全验证** = 可控的智能生成
//! - **Fallback 机制**：LLM 失败时降级到规则匹配

use crate::dsl::pipeline::{BaseOperation, Direction, ExecutionPlan, Field};
use crate::llm::{LlmClient, Message, MessageRole};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// LLM 驱动的 Pipeline 生成器
pub struct LlmToPipeline {
    llm_client: Arc<dyn LlmClient>,
    system_prompt: String,
}

impl LlmToPipeline {
    /// 创建新的 LLM 桥接器
    pub fn new(llm_client: Arc<dyn LlmClient>) -> Self {
        Self {
            llm_client,
            system_prompt: SYSTEM_PROMPT.to_string(),
        }
    }

    /// 使用 LLM 理解用户输入，生成 ExecutionPlan
    ///
    /// # 流程
    ///
    /// 1. 调用 LLM（System Prompt + 用户输入）
    /// 2. 解析 JSON 响应
    /// 3. 检查是否适用（applicable）
    /// 4. 转换为 ExecutionPlan
    /// 5. 安全验证
    pub async fn understand_and_generate(
        &self,
        user_input: &str,
    ) -> Result<ExecutionPlan, String> {
        // 1. 调用 LLM
        let llm_response = self.call_llm(user_input).await?;

        // 2. 解析 JSON
        let llm_intent: LlmIntent = self.parse_json(&llm_response)?;

        // 3. 检查是否适用
        if !llm_intent.applicable {
            return Err(format!(
                "LLM 判定不适用于文件操作: {}",
                llm_intent.explanation
            ));
        }

        // 4. 转换为 ExecutionPlan
        let plan = self.to_execution_plan(llm_intent)?;

        // 5. 安全验证
        plan.validate_safety()?;

        Ok(plan)
    }

    /// 调用 LLM
    async fn call_llm(&self, user_input: &str) -> Result<String, String> {
        let messages = vec![
            Message {
                role: MessageRole::System,
                content: Some(self.system_prompt.clone()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: MessageRole::User,
                content: Some(user_input.to_string()),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        match self.llm_client.chat(messages).await {
            Ok(response) => Ok(response),
            Err(e) => Err(format!("LLM 调用失败: {}", e)),
        }
    }

    /// 解析 JSON 响应
    fn parse_json(&self, response: &str) -> Result<LlmIntent, String> {
        // 尝试从响应中提取 JSON（可能包含其他文本）
        let json_str = extract_json(response)?;

        serde_json::from_str(&json_str)
            .map_err(|e| format!("JSON 解析失败: {}", e))
    }

    /// 转换为 ExecutionPlan
    fn to_execution_plan(&self, intent: LlmIntent) -> Result<ExecutionPlan, String> {
        let mut plan = ExecutionPlan::new();

        // 获取基础操作（必须存在）
        let base_op = intent.base_operation.ok_or("缺少 base_operation")?;

        // 添加基础操作
        plan = match base_op.op_type.as_str() {
            "find_files" => {
                let path = base_op
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");
                let pattern = base_op
                    .parameters
                    .get("pattern")
                    .and_then(|v| v.as_str())
                    .unwrap_or("*");

                plan.with_operation(BaseOperation::FindFiles {
                    path: path.to_string(),
                    pattern: pattern.to_string(),
                })
            }
            "disk_usage" => {
                let path = base_op
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");

                plan.with_operation(BaseOperation::DiskUsage {
                    path: path.to_string(),
                })
            }
            "list_files" => {
                let path = base_op
                    .parameters
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");

                plan.with_operation(BaseOperation::ListFiles {
                    path: path.to_string(),
                })
            }
            _ => {
                return Err(format!(
                    "不支持的基础操作: {}",
                    base_op.op_type
                ))
            }
        };

        // 添加修饰操作
        for modifier in intent.modifiers {
            plan = match modifier.op_type.as_str() {
                "sort" => {
                    let field = modifier
                        .parameters
                        .get("field")
                        .and_then(|v| v.as_str())
                        .map(parse_field)
                        .unwrap_or(Field::Default);

                    let direction = modifier
                        .parameters
                        .get("direction")
                        .and_then(|v| v.as_str())
                        .map(parse_direction)
                        .unwrap_or(Direction::Descending);

                    plan.with_operation(BaseOperation::SortFiles { field, direction })
                }
                "limit" => {
                    let count = modifier
                        .parameters
                        .get("count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as usize;

                    plan.with_operation(BaseOperation::LimitFiles { count })
                }
                "filter" => {
                    let condition = modifier
                        .parameters
                        .get("condition")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    plan.with_operation(BaseOperation::FilterFiles {
                        condition: condition.to_string(),
                    })
                }
                _ => plan, // 忽略不支持的修饰操作
            };
        }

        Ok(plan)
    }
}

// ========== 数据结构 ==========

/// LLM 输出的结构化意图
#[derive(Debug, Deserialize, Serialize)]
pub struct LlmIntent {
    /// 是否适用于 Pipeline 生成（true = 适用，false = 不适用）
    pub applicable: bool,

    /// 意图类型（当 applicable=false 时可为空）
    #[serde(default)]
    pub intent_type: String,

    /// 基础操作（当 applicable=false 时可选）
    #[serde(default)]
    pub base_operation: Option<BaseOpJson>,

    /// 修饰操作列表（当 applicable=false 时为空）
    #[serde(default)]
    pub modifiers: Vec<ModifierJson>,

    /// 解释说明
    pub explanation: String,
}

/// 基础操作（JSON 格式）
#[derive(Debug, Deserialize, Serialize)]
pub struct BaseOpJson {
    #[serde(rename = "type")]
    pub op_type: String,
    pub parameters: HashMap<String, Value>,
}

/// 修饰操作（JSON 格式）
#[derive(Debug, Deserialize, Serialize)]
pub struct ModifierJson {
    #[serde(rename = "type")]
    pub op_type: String,
    #[serde(flatten)]
    pub parameters: HashMap<String, Value>,
}

// ========== 辅助函数 ==========

/// 解析 Field 枚举
fn parse_field(s: &str) -> Field {
    match s.to_lowercase().as_str() {
        "size" => Field::Size,
        "time" => Field::Time,
        "name" => Field::Name,
        "default" => Field::Default,
        _ => Field::Default,
    }
}

/// 解析 Direction 枚举
fn parse_direction(s: &str) -> Direction {
    match s.to_lowercase().as_str() {
        "ascending" | "asc" => Direction::Ascending,
        "descending" | "desc" => Direction::Descending,
        _ => Direction::Descending,
    }
}

/// 从响应中提取 JSON
///
/// LLM 可能返回 ```json ... ``` 格式，需要提取纯 JSON
fn extract_json(response: &str) -> Result<String, String> {
    let response = response.trim();

    // 尝试直接解析
    if response.starts_with('{') {
        return Ok(response.to_string());
    }

    // 尝试提取 ```json ... ```
    if let Some(start) = response.find("```json") {
        let json_start = start + 7; // "```json".len()
        if let Some(end_offset) = response[json_start..].find("```") {
            let json_end = json_start + end_offset;
            return Ok(response[json_start..json_end].trim().to_string());
        }
    }

    // 尝试提取 ``` ... ```
    if let Some(start) = response.find("```") {
        let after_first = start + 3;
        if let Some(end_offset) = response[after_first..].find("```") {
            let json_end = after_first + end_offset;
            return Ok(response[after_first..json_end].trim().to_string());
        }
    }

    // 尝试找到第一个 { 和最后一个 }
    if let Some(start) = response.find('{') {
        if let Some(end) = response.rfind('}') {
            if end > start {
                return Ok(response[start..=end].to_string());
            }
        }
    }

    Err("无法提取 JSON".to_string())
}

// ========== 安全验证 ==========

impl ExecutionPlan {
    /// 安全验证
    ///
    /// 检查：
    /// 1. 路径安全性（不能包含 ..，不能是根目录）
    /// 2. 命令长度限制
    /// 3. 黑名单检查
    pub fn validate_safety(&self) -> Result<(), String> {
        // 验证每个操作
        for op in &self.operations {
            match op {
                BaseOperation::FindFiles { path, .. } => {
                    validate_path(&path)?;
                }
                BaseOperation::DiskUsage { path } => {
                    validate_path(&path)?;
                }
                BaseOperation::ListFiles { path } => {
                    validate_path(&path)?;
                }
                _ => {}
            }
        }

        // 验证生成的命令
        let command = self.to_shell_command();

        if command.len() > 1000 {
            return Err("生成的命令过长".to_string());
        }

        // 黑名单检查
        let dangerous_patterns = [
            "rm -rf /",
            ":(){ :|:& };:",
            "> /dev/sda",
            "mkfs",
            "dd if=",
        ];

        for pattern in &dangerous_patterns {
            if command.contains(pattern) {
                return Err(format!("命令包含危险模式: {}", pattern));
            }
        }

        Ok(())
    }
}

/// 验证路径安全性
fn validate_path(path: &str) -> Result<(), String> {
    if path.contains("..") {
        return Err("路径包含非法字符 ..".to_string());
    }

    if path == "/" {
        return Err("不允许操作根目录".to_string());
    }

    // 检查是否包含 shell 特殊字符（除了 ., /, -, _）
    let invalid_chars = ['$', '`', ';', '|', '&', '>', '<', '\n', '\r'];
    for ch in invalid_chars {
        if path.contains(ch) {
            return Err(format!("路径包含非法字符: {}", ch));
        }
    }

    Ok(())
}

// ========== System Prompt ==========

const SYSTEM_PROMPT: &str = r#"你是 RealConsole 的意图理解助手。你的任务是判断用户输入是否适合文件操作，如果适合则转换为结构化的执行计划。

## 重要规则

**你必须判断用户输入是否适合文件操作！**

- 如果用户询问的是：时间、天气、计算、对话、知识问答等非文件操作，请设置 `applicable: false`
- 只有当用户明确要查找、列出、排序、统计文件/目录时，才设置 `applicable: true`

## 可用的基础操作

### 1. find_files - 查找文件
参数：
- path (string): 搜索路径，默认 "."
- pattern (string): 文件名模式，如 "*.rs", "*.py", "*"

### 2. disk_usage - 检查磁盘使用
参数：
- path (string): 目录路径，默认 "."

### 3. list_files - 列出文件
参数：
- path (string): 目录路径，默认 "."

## 可用的修饰操作

### 1. sort - 排序
参数：
- field (string): "size" | "time" | "name" | "default"
- direction (string): "ascending" (升序/最小/最旧) | "descending" (降序/最大/最新)

### 2. limit - 限制数量
参数：
- count (number): 显示前N个结果

### 3. filter - 过滤
参数：
- condition (string): 过滤条件

## 输出格式

必须输出有效的 JSON，格式如下：

**当适用时** (applicable: true):
{
  "applicable": true,
  "intent_type": "file_operations",
  "base_operation": {
    "type": "基础操作类型",
    "parameters": { 参数字典 }
  },
  "modifiers": [
    {
      "type": "修饰操作类型",
      参数...
    }
  ],
  "explanation": "简短的中文解释"
}

**当不适用时** (applicable: false):
{
  "applicable": false,
  "explanation": "这是一个关于XX的问题，不适合文件操作"
}

## 关键映射规则

1. "最大" / "最多" / "大于" → direction: "descending"
2. "最小" / "最少" / "小于" → direction: "ascending"
3. "最近" / "最新" → field: "time", direction: "descending"
4. "最旧" → field: "time", direction: "ascending"
5. 没有指定方向时，默认 "descending"
6. 文件类型映射：
   - "rs文件" / "rust文件" → pattern: "*.rs"
   - "py文件" / "python文件" → pattern: "*.py"
   - "md文件" / "markdown文件" → pattern: "*.md"
   - 默认 → pattern: "*"

## 示例

### 示例 0 - 不适用的场景
用户输入: "现在几点了"
输出:
{
  "applicable": false,
  "explanation": "这是一个时间查询，不是文件操作"
}

用户输入: "今天天气怎么样"
输出:
{
  "applicable": false,
  "explanation": "这是一个天气查询，不是文件操作"
}

用户输入: "1+1等于几"
输出:
{
  "applicable": false,
  "explanation": "这是一个数学计算，不是文件操作"
}

### 示例 1 - 文件操作
用户输入: "显示当前目录下体积最小的rs文件"
输出:
{
  "applicable": true,
  "intent_type": "file_operations",
  "base_operation": {
    "type": "find_files",
    "parameters": {
      "path": ".",
      "pattern": "*.rs"
    }
  },
  "modifiers": [
    {
      "type": "sort",
      "field": "size",
      "direction": "ascending"
    },
    {
      "type": "limit",
      "count": 1
    }
  ],
  "explanation": "查找.rs文件，按大小升序，取第1个（最小）"
}

### 示例 2 - 文件操作
用户输入: "查找最近修改的md文件"
输出:
{
  "applicable": true,
  "intent_type": "file_operations",
  "base_operation": {
    "type": "find_files",
    "parameters": {
      "path": ".",
      "pattern": "*.md"
    }
  },
  "modifiers": [
    {
      "type": "sort",
      "field": "time",
      "direction": "descending"
    },
    {
      "type": "limit",
      "count": 10
    }
  ],
  "explanation": "查找.md文件，按修改时间降序，显示前10个"
}

### 示例 3 - 文件操作
用户输入: "检查src目录磁盘使用"
输出:
{
  "applicable": true,
  "intent_type": "file_operations",
  "base_operation": {
    "type": "disk_usage",
    "parameters": {
      "path": "src"
    }
  },
  "modifiers": [
    {
      "type": "sort",
      "field": "default",
      "direction": "descending"
    },
    {
      "type": "limit",
      "count": 10
    }
  ],
  "explanation": "检查src目录磁盘使用，按大小降序，显示前10个"
}

现在请处理用户输入。只输出 JSON，不要其他内容。"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_direct() {
        let response = r#"{"intent_type": "test"}"#;
        let result = extract_json(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"intent_type": "test"}"#);
    }

    #[test]
    fn test_extract_json_with_markdown() {
        let response = r#"```json
{"intent_type": "test"}
```"#;
        let result = extract_json(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"intent_type": "test"}"#);
    }

    #[test]
    fn test_extract_json_with_text() {
        let response = r#"Here is the result: {"intent_type": "test"} done"#;
        let result = extract_json(response);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"intent_type": "test"}"#);
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(parse_field("size"), Field::Size);
        assert_eq!(parse_field("time"), Field::Time);
        assert_eq!(parse_field("name"), Field::Name);
        assert_eq!(parse_field("default"), Field::Default);
        assert_eq!(parse_field("unknown"), Field::Default);
    }

    #[test]
    fn test_parse_direction() {
        assert_eq!(parse_direction("ascending"), Direction::Ascending);
        assert_eq!(parse_direction("descending"), Direction::Descending);
        assert_eq!(parse_direction("asc"), Direction::Ascending);
        assert_eq!(parse_direction("desc"), Direction::Descending);
        assert_eq!(parse_direction("unknown"), Direction::Descending);
    }

    #[test]
    fn test_validate_path() {
        assert!(validate_path(".").is_ok());
        assert!(validate_path("./src").is_ok());
        assert!(validate_path("/usr/local").is_ok());

        assert!(validate_path("../..").is_err());
        assert!(validate_path("/").is_err());
        assert!(validate_path("test;rm -rf").is_err());
        assert!(validate_path("$(whoami)").is_err());
    }

    #[test]
    fn test_validate_safety() {
        let mut plan = ExecutionPlan::new();
        plan = plan.with_operation(BaseOperation::FindFiles {
            path: ".".to_string(),
            pattern: "*.rs".to_string(),
        });

        assert!(plan.validate_safety().is_ok());

        // 危险路径
        let mut bad_plan = ExecutionPlan::new();
        bad_plan = bad_plan.with_operation(BaseOperation::FindFiles {
            path: "../..".to_string(),
            pattern: "*".to_string(),
        });

        assert!(bad_plan.validate_safety().is_err());
    }
}
