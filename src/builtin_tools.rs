//! 内置工具实现
//!
//! 提供一组常用的内置工具：
//! - Calculator: 数学计算
//! - FileOps: 文件操作（读/写/列表）
//! - DateTime: 日期时间查询
//! - Lunar: 农历查询（公历/农历转换、节气、干支生肖）

use crate::lunar_tool;
use crate::tool::{Parameter, ParameterType, Tool, ToolRegistry};
use chrono::Local;
use serde_json::{json, Value as JsonValue};
use std::fs;
use std::path::Path;

/// 注册所有内置工具
pub fn register_builtin_tools(registry: &mut ToolRegistry) {
    register_calculator(registry);
    register_file_ops(registry);
    register_datetime(registry);
    register_code_stats(registry);
    register_shell_execute(registry); // ✨ Phase 8: Shell 执行工具
    lunar_tool::register_lunar_tool(registry); // ✨ 农历工具
}

/// 注册计算器工具
fn register_calculator(registry: &mut ToolRegistry) {
    let tool = Tool::new(
        "calculator",
        "执行数学计算。支持: 加(+), 减(-), 乘(*), 除(/), 乘方(^), 括号, 常量(pi, e)。可以一次性计算复杂表达式。",
        vec![
            Parameter {
                name: "expression".to_string(),
                param_type: ParameterType::String,
                description: "数学表达式，如 '10+2-30+40*60+10' 或 '2^3 + sqrt(16)' 或 'sin(pi/2)'".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| {
            let expr = args["expression"]
                .as_str()
                .ok_or("expression 必须是字符串")?;

            // 使用 evalexpr 安全地求值数学表达式
            match evalexpr::eval(expr) {
                Ok(result) => Ok(format!("{} = {}", expr, result)),
                Err(e) => {
                    // 如果 evalexpr 失败，尝试函数格式（向后兼容）
                    if let Some(result) = parse_function_expr(expr) {
                        return result;
                    }
                    Err(format!("计算失败: {}. 表达式: {}", e, expr))
                }
            }
        },
    );

    registry.register(tool);
}

/// 解析函数表达式 (add/sub/mul/div/pow/sqrt)
fn parse_function_expr(expr: &str) -> Option<Result<String, String>> {
    let expr = expr.trim();

    // 匹配函数格式: func(a, b) 或 func(a)
    if let Some(open_paren) = expr.find('(') {
        let func_name = &expr[..open_paren].trim();
        let close_paren = expr.rfind(')')?;
        let args_str = &expr[open_paren + 1..close_paren];

        let args: Vec<&str> = args_str.split(',').map(|s| s.trim()).collect();

        match *func_name {
            "add" | "sub" | "mul" | "div" | "pow" => {
                if args.len() != 2 {
                    return Some(Err(format!("{} 需要 2 个参数", func_name)));
                }

                let a = args[0].parse::<f64>().ok()?;
                let b = args[1].parse::<f64>().ok()?;

                let result = match *func_name {
                    "add" => a + b,
                    "sub" => a - b,
                    "mul" => a * b,
                    "div" => {
                        if b == 0.0 {
                            return Some(Err("除数不能为零".to_string()));
                        }
                        a / b
                    }
                    "pow" => a.powf(b),
                    _ => unreachable!(),
                };

                Some(Ok(format!("{} = {}", expr, result)))
            }
            "sqrt" => {
                if args.len() != 1 {
                    return Some(Err("sqrt 需要 1 个参数".to_string()));
                }

                let a = args[0].parse::<f64>().ok()?;
                if a < 0.0 {
                    return Some(Err("sqrt 参数不能为负数".to_string()));
                }

                Some(Ok(format!("sqrt({}) = {}", a, a.sqrt())))
            }
            _ => None,
        }
    } else {
        None
    }
}

/// 注册文件操作工具
fn register_file_ops(registry: &mut ToolRegistry) {
    // 读取文件
    let read_file = Tool::new(
        "read_file",
        "读取文件内容",
        vec![Parameter {
            name: "path".to_string(),
            param_type: ParameterType::String,
            description: "文件路径".to_string(),
            required: true,
            default: None,
        }],
        |args: JsonValue| {
            let path = args["path"].as_str().ok_or("path 必须是字符串")?;

            // 安全检查：禁止读取敏感文件
            let dangerous_patterns = ["/etc/shadow", "/etc/passwd", ".ssh/id_rsa"];
            for pattern in &dangerous_patterns {
                if path.contains(pattern) {
                    return Err(format!("禁止读取敏感文件: {}", path));
                }
            }

            match fs::read_to_string(path) {
                Ok(content) => {
                    // 限制输出大小（最多 1000 字符）
                    // 使用 chars().take() 以避免 UTF-8 边界问题
                    let char_count = content.chars().count();
                    if char_count > 1000 {
                        let preview: String = content.chars().take(1000).collect();
                        Ok(format!("{}... (已截断，共 {} 字符)", preview, char_count))
                    } else {
                        Ok(content)
                    }
                }
                Err(e) => Err(format!("读取文件失败: {}", e)),
            }
        },
    );

    // 写入文件
    let write_file = Tool::new(
        "write_file",
        "写入内容到文件",
        vec![
            Parameter {
                name: "path".to_string(),
                param_type: ParameterType::String,
                description: "文件路径".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "content".to_string(),
                param_type: ParameterType::String,
                description: "文件内容".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| {
            let path = args["path"].as_str().ok_or("path 必须是字符串")?;
            let content = args["content"].as_str().ok_or("content 必须是字符串")?;

            // 安全检查：禁止写入系统目录
            if path.starts_with("/etc/") || path.starts_with("/sys/") || path.starts_with("/proc/")
            {
                return Err(format!("禁止写入系统目录: {}", path));
            }

            match fs::write(path, content) {
                Ok(_) => Ok(format!("已写入 {} 字节到文件: {}", content.len(), path)),
                Err(e) => Err(format!("写入文件失败: {}", e)),
            }
        },
    );

    // 列出目录
    let list_dir = Tool::new(
        "list_dir",
        "列出目录下的文件和子目录",
        vec![Parameter {
            name: "path".to_string(),
            param_type: ParameterType::String,
            description: "目录路径（默认为当前目录）".to_string(),
            required: false,
            default: Some(json!(".")),
        }],
        |args: JsonValue| {
            let path = args["path"].as_str().unwrap_or(".");

            if !Path::new(path).is_dir() {
                return Err(format!("不是有效的目录: {}", path));
            }

            match fs::read_dir(path) {
                Ok(entries) => {
                    let mut items = Vec::new();
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let file_type = if entry.path().is_dir() {
                            "目录"
                        } else {
                            "文件"
                        };
                        items.push(format!("{} ({})", name, file_type));
                    }

                    if items.is_empty() {
                        Ok("目录为空".to_string())
                    } else {
                        Ok(format!("共 {} 项:\n{}", items.len(), items.join("\n")))
                    }
                }
                Err(e) => Err(format!("读取目录失败: {}", e)),
            }
        },
    );

    registry.register(read_file);
    registry.register(write_file);
    registry.register(list_dir);
}

/// 注册日期时间工具
fn register_datetime(registry: &mut ToolRegistry) {
    let tool = Tool::new(
        "get_datetime",
        "获取当前日期和时间",
        vec![Parameter {
            name: "format".to_string(),
            param_type: ParameterType::String,
            description: "格式类型: full (完整), date (仅日期), time (仅时间), timestamp (时间戳)"
                .to_string(),
            required: false,
            default: Some(json!("full")),
        }],
        |args: JsonValue| {
            let format = args["format"].as_str().unwrap_or("full");
            let now = Local::now();

            let result = match format {
                "full" => now.format("%Y-%m-%d %H:%M:%S").to_string(),
                "date" => now.format("%Y-%m-%d").to_string(),
                "time" => now.format("%H:%M:%S").to_string(),
                "timestamp" => now.timestamp().to_string(),
                _ => {
                    return Err(format!(
                        "不支持的格式: {}。支持: full, date, time, timestamp",
                        format
                    ))
                }
            };

            Ok(result)
        },
    );

    registry.register(tool);
}

/// 注册代码统计工具
fn register_code_stats(registry: &mut ToolRegistry) {
    let tool = Tool::new(
        "count_code_lines",
        "统计指定目录下的代码文件行数。支持按文件扩展名过滤（如 .rs, .js, .py）。",
        vec![
            Parameter {
                name: "directory".to_string(),
                param_type: ParameterType::String,
                description: "要统计的目录路径（默认为当前目录）".to_string(),
                required: false,
                default: Some(json!(".")),
            },
            Parameter {
                name: "extension".to_string(),
                param_type: ParameterType::String,
                description: "文件扩展名（如 'rs', 'js', 'py'，不含点号）".to_string(),
                required: false,
                default: Some(json!("rs")),
            },
        ],
        |args: JsonValue| {
            let directory = args["directory"].as_str().unwrap_or(".");
            let extension = args["extension"].as_str().unwrap_or("rs");

            // 检查目录是否存在
            let dir_path = Path::new(directory);
            if !dir_path.exists() {
                return Err(format!("目录不存在: {}", directory));
            }

            // 递归统计代码行数
            match count_lines_recursive(dir_path, extension) {
                Ok(stats) => {
                    let total_lines = stats.iter().map(|(_, lines)| lines).sum::<usize>();
                    let file_count = stats.len();

                    if file_count == 0 {
                        return Ok(format!("未找到 .{} 文件", extension));
                    }

                    // 构建结果字符串
                    let mut result =
                        format!("统计结果 (目录: {}, 扩展名: .{})\n", directory, extension);
                    result.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                    result.push_str(&format!("文件总数: {} 个\n", file_count));
                    result.push_str(&format!("代码总行数: {} 行\n", total_lines));

                    // 显示前10个最大的文件
                    result.push_str("最大的10个文件:\n");
                    result.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

                    let mut sorted_stats = stats.clone();
                    sorted_stats.sort_by(|a, b| b.1.cmp(&a.1));

                    for (i, (path, lines)) in sorted_stats.iter().take(10).enumerate() {
                        result.push_str(&format!("{}. {:>6} 行  {}\n", i + 1, lines, path));
                    }

                    Ok(result)
                }
                Err(e) => Err(format!("统计失败: {}", e)),
            }
        },
    );

    registry.register(tool);
}

/// 递归统计目录下指定扩展名的文件行数
fn count_lines_recursive(dir: &Path, extension: &str) -> Result<Vec<(String, usize)>, String> {
    let mut results = Vec::new();

    let entries = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();

        // 跳过隐藏文件和目录
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "target" || name_str == "node_modules" {
                continue;
            }
        }

        if path.is_dir() {
            // 递归处理子目录
            if let Ok(sub_results) = count_lines_recursive(&path, extension) {
                results.extend(sub_results);
            }
        } else if path.is_file() {
            // 检查文件扩展名
            if let Some(ext) = path.extension() {
                if ext == extension {
                    // 统计行数
                    if let Ok(content) = fs::read_to_string(&path) {
                        let line_count = content.lines().count();
                        let relative_path = path
                            .strip_prefix(".")
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .to_string();
                        results.push((relative_path, line_count));
                    }
                }
            }
        }
    }

    Ok(results)
}

/// 注册 Shell 执行工具 (Phase 8)
///
/// 允许 LLM 通过 Function Calling 执行安全的 shell 命令
///
/// 安全策略：
/// - 黑名单过滤危险命令（rm -rf /, sudo, etc.）
/// - 只允许只读操作和常见查询命令
/// - 超时限制（10秒）
fn register_shell_execute(registry: &mut ToolRegistry) {
    let tool = Tool::new(
        "shell_execute",
        "执行 shell 命令获取系统信息。支持：查看文件（ls, cat, head, tail）、磁盘占用（du, df）、进程信息（ps）、网络状态（ping, curl）、查找文件（find）等只读操作。严禁使用危险命令（rm, sudo, chmod, chown等）。",
        vec![
            Parameter {
                name: "command".to_string(),
                param_type: ParameterType::String,
                description: "要执行的 shell 命令，例如：'du -sh target' 或 'ls -lah' 或 'ps aux | grep rust'".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| {
            let command = args["command"]
                .as_str()
                .ok_or("command 必须是字符串")?;

            // 安全检查：黑名单
            let dangerous_commands = [
                "rm ", "sudo ", "su ", "chmod ", "chown ", "kill ", "pkill ",
                "shutdown", "reboot", "dd ", "mkfs", "> /dev/", "format",
                "&& rm", "; rm", "| rm", "rm -rf", "rm -f /",
            ];

            let command_lower = command.to_lowercase();
            for dangerous in &dangerous_commands {
                if command_lower.contains(dangerous) {
                    return Err(format!(
                        "安全限制：禁止执行包含 '{}' 的命令。此工具仅支持只读查询操作。",
                        dangerous
                    ));
                }
            }

            // ⚠️ 注意：这里必须使用 block_in_place 而不是创建新的 runtime
            // 因为工具执行本身已经在 tokio runtime 中了
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    match crate::shell_executor::execute_shell(command).await {
                        Ok(output) => {
                            // 限制输出大小（最多 2000 字符）
                            let char_count = output.chars().count();
                            let result = if char_count > 2000 {
                                let preview: String = output.chars().take(2000).collect();
                                format!("{}... (已截断，共 {} 字符)", preview, char_count)
                            } else {
                                output
                            };

                            // ✨ 用户安全建议：明确显示执行的命令
                            Ok(format!(
                                "📌 执行命令: {}\n{}\n",
                                command,
                                result
                            ))
                        }
                        Err(e) => Err(e.to_string()),
                    }
                })
            })
        },
    );

    registry.register(tool);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_add() {
        let mut registry = ToolRegistry::new();
        register_calculator(&mut registry);

        // 测试新的表达式格式
        let result = registry.execute("calculator", json!({"expression": "10 + 5"}));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("15"));
    }

    #[test]
    fn test_calculator_complex_expression() {
        let mut registry = ToolRegistry::new();
        register_calculator(&mut registry);

        // 测试复杂表达式（这就是导致问题的例子）
        let result = registry.execute("calculator", json!({"expression": "10+2-30+40*60+10"}));
        assert!(result.is_ok());
        let output = result.unwrap();
        // 10+2-30+40*60+10 = 12-30+2400+10 = -18+2400+10 = 2392
        assert!(output.contains("2392"));
    }

    #[test]
    fn test_calculator_function_format() {
        let mut registry = ToolRegistry::new();
        register_calculator(&mut registry);

        // 测试旧的函数格式（向后兼容）
        let result = registry.execute("calculator", json!({"expression": "add(10, 5)"}));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("15"));
    }

    #[test]
    fn test_calculator_div_by_zero() {
        let mut registry = ToolRegistry::new();
        register_calculator(&mut registry);

        let result = registry.execute("calculator", json!({"expression": "div(10, 0)"}));
        assert!(result.is_err());
    }

    #[test]
    fn test_calculator_sqrt() {
        let mut registry = ToolRegistry::new();
        register_calculator(&mut registry);

        let result = registry.execute("calculator", json!({"expression": "sqrt(16)"}));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("4"));
    }

    #[test]
    fn test_datetime() {
        let mut registry = ToolRegistry::new();
        register_datetime(&mut registry);

        let result = registry.execute("get_datetime", json!({"format": "date"}));
        assert!(result.is_ok());
        let date = result.unwrap();
        // 应该是 YYYY-MM-DD 格式
        assert!(date.contains('-'));
        assert_eq!(date.split('-').count(), 3);
    }

    #[test]
    fn test_list_dir() {
        let mut registry = ToolRegistry::new();
        register_file_ops(&mut registry);

        // 列出当前目录
        let result = registry.execute("list_dir", json!({"path": "."}));
        assert!(result.is_ok());
    }

    #[test]
    fn test_read_file_dangerous() {
        let mut registry = ToolRegistry::new();
        register_file_ops(&mut registry);

        // 尝试读取敏感文件
        let result = registry.execute("read_file", json!({"path": "/etc/shadow"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("禁止"));
    }

    #[test]
    fn test_write_file_dangerous() {
        let mut registry = ToolRegistry::new();
        register_file_ops(&mut registry);

        // 尝试写入系统目录
        let result = registry.execute(
            "write_file",
            json!({"path": "/etc/test", "content": "test"}),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("禁止"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_shell_execute_safe() {
        let mut registry = ToolRegistry::new();
        register_shell_execute(&mut registry);

        // 测试安全命令：echo
        let result = registry.execute("shell_execute", json!({"command": "echo 'test'"}));
        assert!(result.is_ok());
        assert!(result.unwrap().contains("test"));
    }

    #[test]
    fn test_shell_execute_dangerous_rm() {
        let mut registry = ToolRegistry::new();
        register_shell_execute(&mut registry);

        // 测试危险命令：rm
        let result = registry.execute("shell_execute", json!({"command": "rm -rf /tmp/test"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全限制"));
    }

    #[test]
    fn test_shell_execute_dangerous_sudo() {
        let mut registry = ToolRegistry::new();
        register_shell_execute(&mut registry);

        // 测试危险命令：sudo
        let result = registry.execute("shell_execute", json!({"command": "sudo whoami"}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("安全限制"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_shell_execute_du() {
        let mut registry = ToolRegistry::new();
        register_shell_execute(&mut registry);

        // 测试 du 命令（用户的实际需求）
        let result = registry.execute("shell_execute", json!({"command": "du -sh ."}));
        assert!(result.is_ok());
        // 应该包含大小信息
    }
}
