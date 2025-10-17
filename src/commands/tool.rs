//! 工具管理命令
//!
//! 提供工具列表、工具调用、工具信息等功能

use crate::command::{Command, CommandRegistry};
use crate::tool::ToolRegistry;
use colored::Colorize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 注册工具管理命令
pub fn register_tool_commands(registry: &mut CommandRegistry, tool_registry: Arc<RwLock<ToolRegistry>>) {
    // /tools 命令
    let tools_cmd = Command::from_fn(
        "tools",
        "工具管理: tools [list|call|info]",
        move |arg: &str| handle_tools(arg, Arc::clone(&tool_registry)),
    )
    .with_group("tool");
    registry.register(tools_cmd);
}

/// 处理 /tools 命令
fn handle_tools(arg: &str, tool_registry: Arc<RwLock<ToolRegistry>>) -> String {
    let parts: Vec<&str> = arg.split_whitespace().collect();

    if parts.is_empty() {
        return handle_tools_list(tool_registry);
    }

    let subcommand = parts[0];
    let rest = parts.get(1..).unwrap_or(&[]);

    match subcommand {
        "list" | "l" => handle_tools_list(tool_registry),
        "call" | "c" => {
            if rest.is_empty() {
                format!("{} /tools call <tool_name> <json_args>", "用法:".yellow())
            } else {
                handle_tools_call(rest[0], &rest[1..].join(" "), tool_registry)
            }
        }
        "info" | "i" => {
            if rest.is_empty() {
                format!("{} /tools info <tool_name>", "用法:".yellow())
            } else {
                handle_tools_info(rest[0], tool_registry)
            }
        }
        "help" | "h" => tools_help(),
        _ => format!(
            "{} 未知子命令: {}\\n使用 /tools help 查看帮助",
            "错误:".red(),
            subcommand
        ),
    }
}

/// 列出所有工具
fn handle_tools_list(tool_registry: Arc<RwLock<ToolRegistry>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let registry = tool_registry.read().await;
            let tools = registry.list_tools();

            if tools.is_empty() {
                return format!("{}", "暂无可用工具".dimmed());
            }

            let mut lines = vec![format!(
                "{} {} 个工具:",
                "可用工具".bold().cyan(),
                tools.len().to_string().green()
            )];

            for tool_name in tools {
                if let Some(tool) = registry.get(tool_name) {
                    lines.push(format!(
                        "  {} {} - {}",
                        "•".dimmed(),
                        tool_name.green(),
                        tool.description.dimmed()
                    ));
                }
            }

            lines.join("\n")
        })
    })
}

/// 调用工具
fn handle_tools_call(tool_name: &str, args_str: &str, tool_registry: Arc<RwLock<ToolRegistry>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // 解析 JSON 参数
            let args = if args_str.is_empty() {
                json!({})
            } else {
                match serde_json::from_str(args_str) {
                    Ok(v) => v,
                    Err(e) => {
                        return format!("{} JSON 解析失败: {}", "错误:".red(), e);
                    }
                }
            };

            // 执行工具
            let registry = tool_registry.read().await;
            match registry.execute(tool_name, args) {
                Ok(result) => format!("{}\n{}", format!("✓ {}", tool_name).green(), result),
                Err(e) => format!("{} {}: {}", "错误:".red(), tool_name, e),
            }
        })
    })
}

/// 查看工具信息
fn handle_tools_info(tool_name: &str, tool_registry: Arc<RwLock<ToolRegistry>>) -> String {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let registry = tool_registry.read().await;

            match registry.get(tool_name) {
                Some(tool) => {
                    let mut lines = vec![
                        format!("{} {}", "工具名称:".bold(), tool.name.cyan()),
                        format!("{} {}", "描述:".bold(), tool.description),
                    ];

                    if !tool.parameters.is_empty() {
                        lines.push(format!("\n{}", "参数:".bold()));
                        for param in &tool.parameters {
                            let required_text = if param.required { "必需" } else { "可选" };
                            let required_colored = if param.required { required_text.red() } else { required_text.dimmed() };
                            lines.push(format!(
                                "  {} {} [{}] - {}",
                                "•".dimmed(),
                                param.name.cyan(),
                                required_colored,
                                param.description.dimmed()
                            ));
                            lines.push(format!(
                                "    {} {:?}",
                                "类型:".dimmed(),
                                param.param_type
                            ));
                            if let Some(ref default) = param.default {
                                lines.push(format!(
                                    "    {} {}",
                                    "默认值:".dimmed(),
                                    default
                                ));
                            }
                        }
                    }

                    // 示例 Schema
                    lines.push(format!("\n{}", "Function Schema:".bold()));
                    let schema = tool.to_function_schema();
                    lines.push(format!("{}", serde_json::to_string_pretty(&schema).unwrap().dimmed()));

                    lines.join("\n")
                }
                None => format!("{} 未找到工具: {}", "错误:".red(), tool_name),
            }
        })
    })
}

/// 工具命令帮助
fn tools_help() -> String {
    format!(
        r#"{title}

{subtitle}
  /tools                - 列出所有可用工具
  /tools list           - 列出所有可用工具
  /tools call <tool> <json_args> - 调用工具
  /tools info <tool>    - 查看工具详细信息

{examples}
  /tools list
  /tools info calculator
  /tools call calculator {{"expression": "add(10, 5)"}}
  /tools call get_datetime {{"format": "date"}}

{shortcuts}
  list → l, call → c, info → i
"#,
        title = "工具管理".bold().cyan(),
        subtitle = "用法:".bold(),
        examples = "示例:".bold(),
        shortcuts = "快捷命令:".dimmed()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::{Parameter, ParameterType, Tool};

    fn create_test_registry() -> Arc<RwLock<ToolRegistry>> {
        let mut registry = ToolRegistry::new();

        let test_tool = Tool::new(
            "test_tool",
            "测试工具",
            vec![Parameter {
                name: "value".to_string(),
                param_type: ParameterType::String,
                description: "测试值".to_string(),
                required: true,
                default: None,
            }],
            |args| {
                let value = args["value"].as_str().ok_or("value 必须是字符串")?;
                Ok(format!("返回: {}", value))
            },
        );

        registry.register(test_tool);
        Arc::new(RwLock::new(registry))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tools_list() {
        let registry = create_test_registry();
        let result = handle_tools_list(registry);
        assert!(result.contains("test_tool"));
        assert!(result.contains("1 个工具"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tools_call() {
        let registry = create_test_registry();
        let result = handle_tools_call(
            "test_tool",
            r#"{"value": "hello"}"#,
            registry,
        );
        assert!(result.contains("返回: hello"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tools_info() {
        let registry = create_test_registry();
        let result = handle_tools_info("test_tool", registry);
        assert!(result.contains("test_tool"));
        assert!(result.contains("测试工具"));
        assert!(result.contains("value"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tools_call_invalid_json() {
        let registry = create_test_registry();
        let result = handle_tools_call("test_tool", "invalid json", registry);
        assert!(result.contains("JSON 解析失败"));
    }
}
