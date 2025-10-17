//! 高级工具实现 (Phase 5)
//!
//! 提供更多实用工具：
//! - HTTP 工具组（http_get, http_post）
//! - JSON 工具组（json_parse, json_query）
//! - 文本处理工具组（text_search, text_replace, text_split）
//! - 系统信息工具组（get_env, get_system_info）

use crate::tool::{Parameter, ParameterType, Tool, ToolRegistry};
use serde_json::Value as JsonValue;
use std::time::Duration;

/// 注册所有高级工具到注册表
pub fn register_advanced_tools(registry: &mut ToolRegistry) {
    // HTTP 工具
    registry.register(create_http_get_tool());
    registry.register(create_http_post_tool());

    // JSON 工具
    registry.register(create_json_parse_tool());
    registry.register(create_json_query_tool());

    // 文本处理工具
    registry.register(create_text_search_tool());
    registry.register(create_text_replace_tool());
    registry.register(create_text_split_tool());

    // 系统信息工具
    registry.register(create_get_env_tool());
    registry.register(create_system_info_tool());
}

// ============================================================================
// HTTP 工具组
// ============================================================================

/// HTTP GET 请求工具
fn create_http_get_tool() -> Tool {
    Tool::new(
        "http_get",
        "发送 HTTP GET 请求获取数据",
        vec![
            Parameter {
                name: "url".to_string(),
                param_type: ParameterType::String,
                description: "目标 URL（http 或 https）".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "timeout".to_string(),
                param_type: ParameterType::Number,
                description: "超时时间（秒），默认 30，最大 60".to_string(),
                required: false,
                default: Some(JsonValue::Number(30.into())),
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let url = args["url"]
                .as_str()
                .ok_or("缺少参数 'url'")?;

            // 安全检查：只允许 http/https
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("URL 必须以 http:// 或 https:// 开头".to_string());
            }

            // 获取超时时间
            let timeout = args["timeout"]
                .as_f64()
                .unwrap_or(30.0)
                .clamp(1.0, 60.0); // 限制在 1-60 秒

            // 执行异步 HTTP 请求
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    // 创建 HTTP 客户端
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(timeout as u64))
                        .build()
                        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

                    // 发送请求
                    let response = client
                        .get(url)
                        .send()
                        .await
                        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

                    // 检查状态码
                    let status = response.status();
                    if !status.is_success() {
                        return Err(format!("HTTP 错误: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));
                    }

                    // 读取响应体（限制 10MB）
                    let bytes = response
                        .bytes()
                        .await
                        .map_err(|e| format!("读取响应失败: {}", e))?;

                    if bytes.len() > 10 * 1024 * 1024 {
                        return Err("响应内容超过 10MB 限制".to_string());
                    }

                    // 转换为字符串
                    let text = String::from_utf8_lossy(&bytes).to_string();
                    Ok(text)
                })
            })
        },
    )
}

/// HTTP POST 请求工具
fn create_http_post_tool() -> Tool {
    Tool::new(
        "http_post",
        "发送 HTTP POST 请求提交数据",
        vec![
            Parameter {
                name: "url".to_string(),
                param_type: ParameterType::String,
                description: "目标 URL（http 或 https）".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "body".to_string(),
                param_type: ParameterType::String,
                description: "请求体内容".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "content_type".to_string(),
                param_type: ParameterType::String,
                description: "Content-Type 头，默认 application/json".to_string(),
                required: false,
                default: Some(JsonValue::String("application/json".to_string())),
            },
            Parameter {
                name: "timeout".to_string(),
                param_type: ParameterType::Number,
                description: "超时时间（秒），默认 30，最大 60".to_string(),
                required: false,
                default: Some(JsonValue::Number(30.into())),
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let url = args["url"]
                .as_str()
                .ok_or("缺少参数 'url'")?;

            let body = args["body"]
                .as_str()
                .ok_or("缺少参数 'body'")?;

            // 安全检查
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("URL 必须以 http:// 或 https:// 开头".to_string());
            }

            let content_type = args["content_type"]
                .as_str()
                .unwrap_or("application/json");

            let timeout = args["timeout"]
                .as_f64()
                .unwrap_or(30.0)
                .clamp(1.0, 60.0);

            // 执行异步 HTTP 请求
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let client = reqwest::Client::builder()
                        .timeout(Duration::from_secs(timeout as u64))
                        .build()
                        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

                    let response = client
                        .post(url)
                        .header("Content-Type", content_type)
                        .body(body.to_string())
                        .send()
                        .await
                        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

                    let status = response.status();
                    if !status.is_success() {
                        return Err(format!("HTTP 错误: {} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")));
                    }

                    let bytes = response
                        .bytes()
                        .await
                        .map_err(|e| format!("读取响应失败: {}", e))?;

                    if bytes.len() > 10 * 1024 * 1024 {
                        return Err("响应内容超过 10MB 限制".to_string());
                    }

                    let text = String::from_utf8_lossy(&bytes).to_string();
                    Ok(text)
                })
            })
        },
    )
}

// ============================================================================
// JSON 工具组
// ============================================================================

/// JSON 解析工具
fn create_json_parse_tool() -> Tool {
    Tool::new(
        "json_parse",
        "解析和美化 JSON 字符串",
        vec![
            Parameter {
                name: "json_str".to_string(),
                param_type: ParameterType::String,
                description: "要解析的 JSON 字符串".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "pretty".to_string(),
                param_type: ParameterType::Boolean,
                description: "是否美化输出，默认 true".to_string(),
                required: false,
                default: Some(JsonValue::Bool(true)),
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let json_str = args["json_str"]
                .as_str()
                .ok_or("缺少参数 'json_str'")?;

            // 大小限制：1MB
            if json_str.len() > 1024 * 1024 {
                return Err("JSON 字符串超过 1MB 限制".to_string());
            }

            let pretty = args["pretty"]
                .as_bool()
                .unwrap_or(true);

            // 解析 JSON
            let parsed: JsonValue = serde_json::from_str(json_str)
                .map_err(|e| format!("JSON 解析失败: {}", e))?;

            // 格式化输出
            if pretty {
                serde_json::to_string_pretty(&parsed)
                    .map_err(|e| format!("JSON 格式化失败: {}", e))
            } else {
                serde_json::to_string(&parsed)
                    .map_err(|e| format!("JSON 序列化失败: {}", e))
            }
        },
    )
}

/// JSON 查询工具（简化版，使用路径访问）
fn create_json_query_tool() -> Tool {
    Tool::new(
        "json_query",
        "从 JSON 中提取指定字段",
        vec![
            Parameter {
                name: "json_str".to_string(),
                param_type: ParameterType::String,
                description: "JSON 字符串".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "path".to_string(),
                param_type: ParameterType::String,
                description: "字段路径，如 'user.name' 或 'items[0].id'".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let json_str = args["json_str"]
                .as_str()
                .ok_or("缺少参数 'json_str'")?;

            let path = args["path"]
                .as_str()
                .ok_or("缺少参数 'path'")?;

            // 大小限制
            if json_str.len() > 1024 * 1024 {
                return Err("JSON 字符串超过 1MB 限制".to_string());
            }

            // 解析 JSON
            let parsed: JsonValue = serde_json::from_str(json_str)
                .map_err(|e| format!("JSON 解析失败: {}", e))?;

            // 简单路径解析（支持 . 和 [])
            let result = query_json_path(&parsed, path)?;

            // 格式化结果
            serde_json::to_string_pretty(&result)
                .map_err(|e| format!("结果序列化失败: {}", e))
        },
    )
}

/// 简单的 JSON 路径查询（不使用外部库）
fn query_json_path(value: &JsonValue, path: &str) -> Result<JsonValue, String> {
    let mut current = value;

    // 分割路径（支持 . 和 []）
    let parts: Vec<&str> = path.split('.').collect();

    for part in parts {
        // 处理数组索引 items[0]
        if part.contains('[') && part.contains(']') {
            let key_end = part.find('[').unwrap();
            let key = &part[..key_end];

            // 先访问对象
            if !key.is_empty() {
                current = current.get(key)
                    .ok_or(format!("字段 '{}' 不存在", key))?;
            }

            // 提取索引
            let idx_start = part.find('[').unwrap() + 1;
            let idx_end = part.find(']').unwrap();
            let index_str = &part[idx_start..idx_end];
            let index: usize = index_str.parse()
                .map_err(|_| format!("无效的数组索引: {}", index_str))?;

            // 访问数组
            current = current.get(index)
                .ok_or(format!("数组索引 {} 越界", index))?;
        } else {
            // 普通字段访问
            current = current.get(part)
                .ok_or(format!("字段 '{}' 不存在", part))?;
        }
    }

    Ok(current.clone())
}

// ============================================================================
// 文本处理工具组
// ============================================================================

/// 文本搜索工具
fn create_text_search_tool() -> Tool {
    Tool::new(
        "text_search",
        "在文本中搜索匹配的行",
        vec![
            Parameter {
                name: "text".to_string(),
                param_type: ParameterType::String,
                description: "要搜索的文本".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "pattern".to_string(),
                param_type: ParameterType::String,
                description: "搜索模式（支持正则表达式）".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "case_sensitive".to_string(),
                param_type: ParameterType::Boolean,
                description: "是否区分大小写，默认 false".to_string(),
                required: false,
                default: Some(JsonValue::Bool(false)),
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let text = args["text"]
                .as_str()
                .ok_or("缺少参数 'text'")?;

            let pattern = args["pattern"]
                .as_str()
                .ok_or("缺少参数 'pattern'")?;

            let case_sensitive = args["case_sensitive"]
                .as_bool()
                .unwrap_or(false);

            // 构建正则表达式
            let regex_pattern = if case_sensitive {
                format!("(?-i){}", pattern)
            } else {
                format!("(?i){}", pattern)
            };

            let re = regex::Regex::new(&regex_pattern)
                .map_err(|e| format!("正则表达式错误: {}", e))?;

            // 搜索匹配的行
            let mut matches = Vec::new();
            for (line_num, line) in text.lines().enumerate() {
                if re.is_match(line) {
                    matches.push(format!("{}> {}", line_num + 1, line));
                }
            }

            if matches.is_empty() {
                Ok("未找到匹配的行".to_string())
            } else {
                Ok(format!("找到 {} 个匹配:\n{}", matches.len(), matches.join("\n")))
            }
        },
    )
}

/// 文本替换工具
fn create_text_replace_tool() -> Tool {
    Tool::new(
        "text_replace",
        "替换文本中的内容",
        vec![
            Parameter {
                name: "text".to_string(),
                param_type: ParameterType::String,
                description: "原始文本".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "pattern".to_string(),
                param_type: ParameterType::String,
                description: "要替换的模式".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "replacement".to_string(),
                param_type: ParameterType::String,
                description: "替换内容".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "all".to_string(),
                param_type: ParameterType::Boolean,
                description: "是否替换所有匹配，默认 true".to_string(),
                required: false,
                default: Some(JsonValue::Bool(true)),
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let text = args["text"]
                .as_str()
                .ok_or("缺少参数 'text'")?;

            let pattern = args["pattern"]
                .as_str()
                .ok_or("缺少参数 'pattern'")?;

            let replacement = args["replacement"]
                .as_str()
                .ok_or("缺少参数 'replacement'")?;

            let replace_all = args["all"]
                .as_bool()
                .unwrap_or(true);

            // 构建正则表达式
            let re = regex::Regex::new(pattern)
                .map_err(|e| format!("正则表达式错误: {}", e))?;

            // 执行替换
            let result = if replace_all {
                re.replace_all(text, replacement).to_string()
            } else {
                re.replace(text, replacement).to_string()
            };

            Ok(result)
        },
    )
}

/// 文本分割工具
fn create_text_split_tool() -> Tool {
    Tool::new(
        "text_split",
        "按分隔符分割文本",
        vec![
            Parameter {
                name: "text".to_string(),
                param_type: ParameterType::String,
                description: "要分割的文本".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "delimiter".to_string(),
                param_type: ParameterType::String,
                description: "分隔符".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "max_split".to_string(),
                param_type: ParameterType::Number,
                description: "最大分割次数，0 表示无限，默认 0".to_string(),
                required: false,
                default: Some(JsonValue::Number(0.into())),
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let text = args["text"]
                .as_str()
                .ok_or("缺少参数 'text'")?;

            let delimiter = args["delimiter"]
                .as_str()
                .ok_or("缺少参数 'delimiter'")?;

            let max_split = args["max_split"]
                .as_f64()
                .unwrap_or(0.0) as usize;

            // 执行分割
            let parts: Vec<&str> = if max_split == 0 {
                text.split(delimiter).collect()
            } else {
                text.splitn(max_split + 1, delimiter).collect()
            };

            // 格式化输出
            let mut result = format!("分割成 {} 部分:\n", parts.len());
            for (i, part) in parts.iter().enumerate() {
                result.push_str(&format!("[{}] {}\n", i, part));
            }

            Ok(result)
        },
    )
}

// ============================================================================
// 系统信息工具组
// ============================================================================

/// 获取环境变量工具
fn create_get_env_tool() -> Tool {
    Tool::new(
        "get_env",
        "获取环境变量值",
        vec![
            Parameter {
                name: "name".to_string(),
                param_type: ParameterType::String,
                description: "环境变量名".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "default".to_string(),
                param_type: ParameterType::String,
                description: "默认值（如果环境变量不存在）".to_string(),
                required: false,
                default: None,
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let name = args["name"]
                .as_str()
                .ok_or("缺少参数 'name'")?;

            // 安全检查：禁止读取敏感环境变量
            let sensitive_patterns = [
                "PASSWORD", "SECRET", "TOKEN", "API_KEY", "PRIVATE_KEY",
                "CREDENTIAL", "AUTH", "PASS"
            ];

            let name_upper = name.to_uppercase();
            for pattern in &sensitive_patterns {
                if name_upper.contains(pattern) {
                    return Err(format!("禁止读取敏感环境变量: {}", name));
                }
            }

            // 读取环境变量
            match std::env::var(name) {
                Ok(value) => Ok(format!("{} = {}", name, value)),
                Err(_) => {
                    if let Some(default) = args["default"].as_str() {
                        Ok(format!("{} = {} (默认值)", name, default))
                    } else {
                        Err(format!("环境变量 '{}' 不存在", name))
                    }
                }
            }
        },
    )
}

/// 获取系统信息工具
fn create_system_info_tool() -> Tool {
    Tool::new(
        "get_system_info",
        "获取系统信息",
        vec![
            Parameter {
                name: "info_type".to_string(),
                param_type: ParameterType::String,
                description: "信息类型: os, arch, hostname, user, home_dir".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let info_type = args["info_type"]
                .as_str()
                .ok_or("缺少参数 'info_type'")?;

            match info_type.to_lowercase().as_str() {
                "os" => Ok(format!("操作系统: {}", std::env::consts::OS)),
                "arch" => Ok(format!("架构: {}", std::env::consts::ARCH)),
                "hostname" => {
                    match hostname::get() {
                        Ok(name) => Ok(format!("主机名: {}", name.to_string_lossy())),
                        Err(e) => Err(format!("获取主机名失败: {}", e))
                    }
                }
                "user" => {
                    match std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
                        Ok(user) => Ok(format!("当前用户: {}", user)),
                        Err(_) => Err("无法获取当前用户名".to_string())
                    }
                }
                "home_dir" => {
                    match std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
                        Ok(home) => Ok(format!("Home 目录: {}", home)),
                        Err(_) => Err("无法获取 home 目录".to_string())
                    }
                }
                _ => Err(format!(
                    "未知的信息类型: {}. 支持: os, arch, hostname, user, home_dir",
                    info_type
                ))
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_parse_valid() {
        let tool = create_json_parse_tool();
        let args = serde_json::json!({
            "json_str": r#"{"name": "test", "value": 123}"#,
            "pretty": true
        });

        let result = (tool.handler)(args);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("name"));
        assert!(output.contains("test"));
    }

    #[test]
    fn test_json_parse_invalid() {
        let tool = create_json_parse_tool();
        let args = serde_json::json!({
            "json_str": r#"{"invalid json"#,
            "pretty": true
        });

        let result = (tool.handler)(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("解析失败"));
    }

    #[test]
    fn test_json_query_simple_path() {
        let tool = create_json_query_tool();
        let args = serde_json::json!({
            "json_str": r#"{"user": {"name": "Alice", "age": 30}}"#,
            "path": "user.name"
        });

        let result = (tool.handler)(args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Alice"));
    }

    #[test]
    fn test_json_query_array_index() {
        let tool = create_json_query_tool();
        let args = serde_json::json!({
            "json_str": r#"{"items": [{"id": 1}, {"id": 2}]}"#,
            "path": "items[0].id"
        });

        let result = (tool.handler)(args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("1"));
    }

    #[test]
    fn test_text_search() {
        let tool = create_text_search_tool();
        let args = serde_json::json!({
            "text": "Hello World\nRust is great\nHello Rust",
            "pattern": "Hello",
            "case_sensitive": false
        });

        let result = (tool.handler)(args);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("2 个匹配"));
    }

    #[test]
    fn test_text_replace() {
        let tool = create_text_replace_tool();
        let args = serde_json::json!({
            "text": "Hello World, Hello Rust",
            "pattern": "Hello",
            "replacement": "Hi",
            "all": true
        });

        let result = (tool.handler)(args);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Hi World"));
        assert!(output.contains("Hi Rust"));
    }

    #[test]
    fn test_text_split() {
        let tool = create_text_split_tool();
        let args = serde_json::json!({
            "text": "apple,banana,cherry",
            "delimiter": ",",
            "max_split": 0
        });

        let result = (tool.handler)(args);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("3 部分"));
        assert!(output.contains("apple"));
    }

    #[test]
    fn test_get_env_safe() {
        let tool = create_get_env_tool();
        let args = serde_json::json!({
            "name": "PATH",
            "default": "not_found"
        });

        let result = (tool.handler)(args);
        // PATH 应该存在
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_env_sensitive() {
        let tool = create_get_env_tool();
        let args = serde_json::json!({
            "name": "API_KEY",
            "default": "default"
        });

        let result = (tool.handler)(args);
        // 应该被阻止
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("敏感"));
    }

    #[test]
    fn test_system_info_os() {
        let tool = create_system_info_tool();
        let args = serde_json::json!({
            "info_type": "os"
        });

        let result = (tool.handler)(args);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("操作系统"));
    }
}
