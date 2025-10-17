//! 工具系统核心
//!
//! 提供工具定义、注册、执行等功能
//!
//! 架构：
//! - Tool: 工具定义（名称、描述、参数 Schema、执行函数）
//! - ToolRegistry: 工具注册表
//! - ToolResult: 工具执行结果

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::sync::Arc;

/// 工具参数类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

/// 工具参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    /// 参数名称
    pub name: String,

    /// 参数类型
    #[serde(rename = "type")]
    pub param_type: ParameterType,

    /// 参数描述
    pub description: String,

    /// 是否必需
    pub required: bool,

    /// 默认值
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<JsonValue>,
}

/// 工具定义
pub struct Tool {
    /// 工具名称
    pub name: String,

    /// 工具描述
    pub description: String,

    /// 参数列表
    pub parameters: Vec<Parameter>,

    /// 执行函数
    pub handler: Arc<dyn Fn(JsonValue) -> Result<String, String> + Send + Sync>,
}

impl Tool {
    /// 创建新工具
    pub fn new<F>(name: &str, description: &str, parameters: Vec<Parameter>, handler: F) -> Self
    where
        F: Fn(JsonValue) -> Result<String, String> + Send + Sync + 'static,
    {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            parameters,
            handler: Arc::new(handler),
        }
    }

    /// 转换为 OpenAI Function Schema
    pub fn to_function_schema(&self) -> JsonValue {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        for param in &self.parameters {
            let mut prop = serde_json::Map::new();
            prop.insert("type".to_string(), json!(param.param_type));
            prop.insert("description".to_string(), json!(param.description));

            if let Some(ref default) = param.default {
                prop.insert("default".to_string(), default.clone());
            }

            properties.insert(param.name.clone(), JsonValue::Object(prop));

            if param.required {
                required.push(param.name.clone());
            }
        }

        json!({
            "type": "function",
            "function": {
                "name": self.name,
                "description": self.description,
                "parameters": {
                    "type": "object",
                    "properties": properties,
                    "required": required,
                }
            }
        })
    }

    /// 执行工具
    pub fn execute(&self, args: JsonValue) -> Result<String, String> {
        // 验证必需参数
        for param in &self.parameters {
            if param.required {
                if let Some(obj) = args.as_object() {
                    if !obj.contains_key(&param.name) {
                        return Err(format!("缺少必需参数: {}", param.name));
                    }
                } else {
                    return Err("参数格式错误，应为 JSON 对象".to_string());
                }
            }
        }

        // 调用执行函数
        (self.handler)(args)
    }
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("parameters", &self.parameters)
            .finish()
    }
}

/// 工具注册表
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register(&mut self, tool: Tool) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// 获取所有工具名称
    pub fn list_tools(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    /// 获取所有工具的 Function Schema（用于 LLM）
    pub fn get_function_schemas(&self) -> Vec<JsonValue> {
        self.tools.values()
            .map(|tool| tool.to_function_schema())
            .collect()
    }

    /// 执行工具
    pub fn execute(&self, name: &str, args: JsonValue) -> Result<String, String> {
        match self.get(name) {
            Some(tool) => tool.execute(args),
            None => Err(format!("未找到工具: {}", name)),
        }
    }

    /// 工具数量
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具调用请求（来自 LLM）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// 工具名称
    pub name: String,

    /// 工具参数（JSON）
    pub arguments: JsonValue,
}

/// 工具执行结果
#[derive(Debug, Clone)]
pub struct ToolResult {
    /// 工具名称
    pub tool_name: String,

    /// 是否成功
    pub success: bool,

    /// 结果内容
    pub content: String,
}

impl ToolResult {
    /// 创建成功结果
    pub fn success(tool_name: String, content: String) -> Self {
        Self {
            tool_name,
            success: true,
            content,
        }
    }

    /// 创建失败结果
    pub fn error(tool_name: String, error: String) -> Self {
        Self {
            tool_name,
            success: false,
            content: format!("错误: {}", error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_calculator_handler(args: JsonValue) -> Result<String, String> {
        let a = args["a"].as_f64().ok_or("参数 a 必须是数字")?;
        let b = args["b"].as_f64().ok_or("参数 b 必须是数字")?;
        let op = args["op"].as_str().ok_or("参数 op 必须是字符串")?;

        let result = match op {
            "add" => a + b,
            "sub" => a - b,
            "mul" => a * b,
            "div" => {
                if b == 0.0 {
                    return Err("除数不能为零".to_string());
                }
                a / b
            }
            _ => return Err(format!("不支持的操作: {}", op)),
        };

        Ok(format!("{} {} {} = {}", a, op, b, result))
    }

    #[test]
    fn test_tool_creation() {
        let tool = Tool::new(
            "calculator",
            "简单计算器",
            vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: ParameterType::Number,
                    description: "第一个数".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: ParameterType::Number,
                    description: "第二个数".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "op".to_string(),
                    param_type: ParameterType::String,
                    description: "操作符 (add/sub/mul/div)".to_string(),
                    required: true,
                    default: None,
                },
            ],
            test_calculator_handler,
        );

        assert_eq!(tool.name, "calculator");
        assert_eq!(tool.parameters.len(), 3);
    }

    #[test]
    fn test_tool_function_schema() {
        let tool = Tool::new(
            "calculator",
            "简单计算器",
            vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: ParameterType::Number,
                    description: "第一个数".to_string(),
                    required: true,
                    default: None,
                },
            ],
            test_calculator_handler,
        );

        let schema = tool.to_function_schema();
        assert!(schema["function"]["name"].as_str().unwrap() == "calculator");
        assert!(schema["function"]["parameters"]["properties"]["a"].is_object());
    }

    #[test]
    fn test_tool_execute() {
        let tool = Tool::new(
            "calculator",
            "简单计算器",
            vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: ParameterType::Number,
                    description: "第一个数".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: ParameterType::Number,
                    description: "第二个数".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "op".to_string(),
                    param_type: ParameterType::String,
                    description: "操作符".to_string(),
                    required: true,
                    default: None,
                },
            ],
            test_calculator_handler,
        );

        // 成功执行
        let result = tool.execute(json!({"a": 10, "b": 5, "op": "add"}));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("15"));

        // 缺少参数
        let result = tool.execute(json!({"a": 10}));
        assert!(result.is_err());

        // 除以零
        let result = tool.execute(json!({"a": 10, "b": 0, "op": "div"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_registry() {
        let mut registry = ToolRegistry::new();

        let tool = Tool::new(
            "calculator",
            "简单计算器",
            vec![],
            test_calculator_handler,
        );

        registry.register(tool);

        assert_eq!(registry.len(), 1);
        assert!(registry.get("calculator").is_some());
        assert!(registry.get("unknown").is_none());

        let tools = registry.list_tools();
        assert_eq!(tools.len(), 1);
        assert!(tools.contains(&"calculator"));
    }

    #[test]
    fn test_registry_execute() {
        let mut registry = ToolRegistry::new();

        let tool = Tool::new(
            "calculator",
            "简单计算器",
            vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: ParameterType::Number,
                    description: "第一个数".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: ParameterType::Number,
                    description: "第二个数".to_string(),
                    required: true,
                    default: None,
                },
                Parameter {
                    name: "op".to_string(),
                    param_type: ParameterType::String,
                    description: "操作符".to_string(),
                    required: true,
                    default: None,
                },
            ],
            test_calculator_handler,
        );

        registry.register(tool);

        // 成功执行
        let result = registry.execute("calculator", json!({"a": 20, "b": 10, "op": "sub"}));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("10"));

        // 工具不存在
        let result = registry.execute("unknown", json!({}));
        assert!(result.is_err());
    }
}
