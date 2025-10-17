# 工具调用开发者指南

## 📖 概述

本指南面向希望扩展 RealConsole 工具调用功能的开发者。您将学习如何：
- 创建自定义工具
- 注册工具到系统
- 实现工具处理逻辑
- 编写工具测试
- 集成到 Agent

---

## 🏗️ 架构概览

```
┌─────────────────────────────────────────────────────────┐
│                         Agent                           │
│  - 接收用户输入                                          │
│  - 根据配置决定是否使用工具调用                          │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│                    ToolExecutor                         │
│  - 迭代执行工具调用（最多 5 轮）                         │
│  - 与 LLM 对话，解析 tool_calls                         │
│  - 执行工具，将结果返回给 LLM                           │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│                   ToolRegistry                          │
│  - 存储所有已注册的工具                                  │
│  - 提供工具查询、执行接口                                │
│  - 生成 OpenAI Function Schema                          │
└────────────────┬────────────────────────────────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│                        Tool                             │
│  - 工具定义（名称、描述、参数）                          │
│  - 处理函数（执行逻辑）                                  │
│  - 参数验证                                              │
└─────────────────────────────────────────────────────────┘
```

---

## 🛠️ 创建自定义工具

### 步骤 1: 定义工具结构

工具由以下部分组成：
- **名称** (name): 唯一标识符，如 "my_tool"
- **描述** (description): 工具功能说明，LLM 会根据此选择工具
- **参数** (parameters): 参数列表，定义类型、描述、是否必需等
- **处理函数** (handler): 执行工具逻辑的闭包

### 步骤 2: 实现工具

创建新文件 `src/custom_tools.rs`:

```rust
use crate::tool::{Parameter, ParameterType, Tool};
use serde_json::Value as JsonValue;

/// 创建一个天气查询工具
pub fn create_weather_tool() -> Tool {
    Tool::new(
        // 工具名称
        "get_weather",

        // 工具描述（LLM 会根据此选择工具）
        "获取指定城市的天气信息",

        // 参数列表
        vec![
            Parameter {
                name: "city".to_string(),
                param_type: ParameterType::String,
                description: "城市名称，如 '北京'、'上海'".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "unit".to_string(),
                param_type: ParameterType::String,
                description: "温度单位：'celsius' 或 'fahrenheit'".to_string(),
                required: false,
                default: Some(JsonValue::String("celsius".to_string())),
            },
        ],

        // 处理函数
        |args: JsonValue| -> Result<String, String> {
            // 1. 提取参数
            let city = args["city"]
                .as_str()
                .ok_or("缺少参数 'city'")?;

            let unit = args["unit"]
                .as_str()
                .unwrap_or("celsius");

            // 2. 执行业务逻辑
            // 这里是示例，实际应调用天气 API
            let temp = if unit == "celsius" { 22 } else { 72 };
            let condition = "晴朗";

            // 3. 返回结果
            Ok(format!(
                "{}的天气：{}，温度 {}°{}",
                city,
                condition,
                temp,
                if unit == "celsius" { "C" } else { "F" }
            ))
        },
    )
}
```

### 步骤 3: 注册工具

在 `src/custom_tools.rs` 中添加注册函数：

```rust
use crate::tool::ToolRegistry;

/// 注册所有自定义工具
pub fn register_custom_tools(registry: &mut ToolRegistry) {
    registry.register(create_weather_tool());
    // 可以注册更多工具
    // registry.register(create_another_tool());
}
```

### 步骤 4: 集成到 Agent

修改 `src/agent.rs`:

```rust
impl Agent {
    pub fn new(config: Config, registry: CommandRegistry) -> Self {
        // ... 现有代码 ...

        // 注册内置工具
        let mut tool_registry = ToolRegistry::new();
        crate::builtin_tools::register_builtin_tools(&mut tool_registry);

        // ✨ 注册自定义工具
        crate::custom_tools::register_custom_tools(&mut tool_registry);

        // ... 剩余代码 ...
    }
}
```

### 步骤 5: 更新模块声明

在 `src/lib.rs` 中添加：

```rust
pub mod custom_tools;
```

---

## 📝 完整示例

### 示例 1: HTTP 请求工具

```rust
use reqwest;

pub fn create_http_get_tool() -> Tool {
    Tool::new(
        "http_get",
        "发送 HTTP GET 请求获取数据",
        vec![
            Parameter {
                name: "url".to_string(),
                param_type: ParameterType::String,
                description: "目标 URL".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let url = args["url"]
                .as_str()
                .ok_or("缺少参数 'url'")?;

            // 使用 tokio 异步运行时执行异步代码
            let result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    // 发送 HTTP 请求
                    let response = reqwest::get(url)
                        .await
                        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

                    // 获取响应文本
                    let text = response
                        .text()
                        .await
                        .map_err(|e| format!("读取响应失败: {}", e))?;

                    Ok(text)
                })
            });

            result
        },
    )
}
```

---

### 示例 2: 数据库查询工具

```rust
pub fn create_db_query_tool() -> Tool {
    Tool::new(
        "db_query",
        "执行 SQL 查询（仅支持 SELECT）",
        vec![
            Parameter {
                name: "query".to_string(),
                param_type: ParameterType::String,
                description: "SQL 查询语句".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let query = args["query"]
                .as_str()
                .ok_or("缺少参数 'query'")?;

            // 安全检查：只允许 SELECT 查询
            if !query.trim().to_uppercase().starts_with("SELECT") {
                return Err("仅支持 SELECT 查询".to_string());
            }

            // 执行查询（示例代码，需要实际的数据库连接）
            // let result = execute_query(query)?;

            Ok(format!("查询结果: [模拟数据]"))
        },
    )
}
```

---

### 示例 3: 系统命令工具

```rust
use std::process::Command;

pub fn create_shell_tool() -> Tool {
    Tool::new(
        "run_command",
        "执行系统命令（谨慎使用）",
        vec![
            Parameter {
                name: "command".to_string(),
                param_type: ParameterType::String,
                description: "要执行的命令".to_string(),
                required: true,
                default: None,
            },
        ],
        |args: JsonValue| -> Result<String, String> {
            let cmd = args["command"]
                .as_str()
                .ok_or("缺少参数 'command'")?;

            // 安全检查：黑名单
            let dangerous_cmds = ["rm -rf", "format", "dd if="];
            for dangerous in &dangerous_cmds {
                if cmd.contains(dangerous) {
                    return Err(format!("危险命令被阻止: {}", dangerous));
                }
            }

            // 执行命令
            let output = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .map_err(|e| format!("命令执行失败: {}", e))?;

            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(String::from_utf8_lossy(&output.stderr).to_string())
            }
        },
    )
}
```

---

## 🧪 测试工具

### 单元测试

创建测试文件 `tests/test_custom_tools.rs`:

```rust
use realconsole::tool::ToolRegistry;
use realconsole::custom_tools::register_custom_tools;
use serde_json::json;

#[test]
fn test_weather_tool() {
    let mut registry = ToolRegistry::new();
    register_custom_tools(&mut registry);

    // 测试工具是否注册成功
    assert!(registry.get("get_weather").is_some());

    // 测试工具执行
    let result = registry.execute(
        "get_weather",
        json!({"city": "北京", "unit": "celsius"})
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("北京"));
    assert!(output.contains("晴朗"));
}

#[test]
fn test_weather_tool_missing_param() {
    let mut registry = ToolRegistry::new();
    register_custom_tools(&mut registry);

    // 测试缺少必需参数的情况
    let result = registry.execute(
        "get_weather",
        json!({})  // 缺少 city 参数
    );

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("city"));
}

#[test]
fn test_weather_tool_default_unit() {
    let mut registry = ToolRegistry::new();
    register_custom_tools(&mut registry);

    // 测试默认参数
    let result = registry.execute(
        "get_weather",
        json!({"city": "上海"})  // 不提供 unit，使用默认值
    );

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("°C"));  // 默认使用 celsius
}
```

### 集成测试

```rust
use realconsole::agent::Agent;
use realconsole::config::Config;
use realconsole::command::CommandRegistry;

#[tokio::test(flavor = "multi_thread")]
async fn test_weather_tool_integration() {
    let mut config = Config::default();
    config.features.tool_calling_enabled = Some(true);

    let registry = CommandRegistry::new();
    let agent = Agent::new(config, registry);

    // 验证工具已注册
    let tool_reg = agent.tool_registry().read().await;
    assert!(tool_reg.get("get_weather").is_some());
}
```

---

## 📊 参数类型

工具支持以下参数类型：

```rust
pub enum ParameterType {
    String,   // 字符串
    Number,   // 数字（i64 或 f64）
    Boolean,  // 布尔值
    Array,    // 数组
    Object,   // JSON 对象
}
```

### 类型使用示例

```rust
vec![
    // 字符串参数
    Parameter {
        name: "name".to_string(),
        param_type: ParameterType::String,
        description: "用户名称".to_string(),
        required: true,
        default: None,
    },

    // 数字参数
    Parameter {
        name: "age".to_string(),
        param_type: ParameterType::Number,
        description: "年龄".to_string(),
        required: false,
        default: Some(JsonValue::Number(18.into())),
    },

    // 布尔参数
    Parameter {
        name: "active".to_string(),
        param_type: ParameterType::Boolean,
        description: "是否激活".to_string(),
        required: false,
        default: Some(JsonValue::Bool(true)),
    },

    // 数组参数
    Parameter {
        name: "tags".to_string(),
        param_type: ParameterType::Array,
        description: "标签列表".to_string(),
        required: false,
        default: Some(json!([])),
    },

    // 对象参数
    Parameter {
        name: "metadata".to_string(),
        param_type: ParameterType::Object,
        description: "元数据".to_string(),
        required: false,
        default: Some(json!({})),
    },
]
```

---

## 🔐 安全最佳实践

### 1. 输入验证

始终验证用户输入：

```rust
|args: JsonValue| -> Result<String, String> {
    // ✅ 验证必需参数
    let path = args["path"]
        .as_str()
        .ok_or("缺少参数 'path'")?;

    // ✅ 验证参数格式
    if !path.starts_with("/home/") {
        return Err("路径必须在 /home/ 目录下".to_string());
    }

    // ✅ 验证参数范围
    let count = args["count"]
        .as_i64()
        .ok_or("参数 'count' 必须是整数")?;

    if count < 1 || count > 100 {
        return Err("count 必须在 1-100 之间".to_string());
    }

    // ... 执行逻辑
}
```

---

### 2. 路径安全

防止路径遍历攻击：

```rust
use std::path::Path;

fn is_safe_path(path: &str) -> bool {
    let path = Path::new(path);

    // ❌ 阻止绝对路径到系统目录
    if path.starts_with("/etc") ||
       path.starts_with("/bin") ||
       path.starts_with("/usr") {
        return false;
    }

    // ❌ 阻止路径遍历
    for component in path.components() {
        if component == std::path::Component::ParentDir {
            return false;
        }
    }

    // ✅ 允许安全路径
    true
}
```

---

### 3. 命令注入防护

```rust
// ❌ 不安全：直接拼接命令
let cmd = format!("rm {}", user_input);  // 危险！

// ✅ 安全：使用参数化
Command::new("rm")
    .arg(user_input)
    .output()
```

---

### 4. 速率限制

对于消耗资源的操作，添加速率限制：

```rust
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct RateLimiter {
    last_call: Mutex<Option<Instant>>,
    min_interval: Duration,
}

impl RateLimiter {
    fn check(&self) -> Result<(), String> {
        let mut last = self.last_call.lock().unwrap();

        if let Some(last_time) = *last {
            let elapsed = last_time.elapsed();
            if elapsed < self.min_interval {
                return Err(format!(
                    "请等待 {} 秒后再试",
                    (self.min_interval - elapsed).as_secs()
                ));
            }
        }

        *last = Some(Instant::now());
        Ok(())
    }
}
```

---

## 🎨 高级技巧

### 1. 带状态的工具

使用 Arc + Mutex 共享状态：

```rust
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub fn create_kv_store_tool() -> Tool {
    // 创建共享存储
    let store = Arc::new(Mutex::new(HashMap::<String, String>::new()));

    // 克隆用于闭包
    let store_clone = Arc::clone(&store);

    Tool::new(
        "kv_set",
        "设置键值对",
        vec![
            Parameter {
                name: "key".to_string(),
                param_type: ParameterType::String,
                description: "键".to_string(),
                required: true,
                default: None,
            },
            Parameter {
                name: "value".to_string(),
                param_type: ParameterType::String,
                description: "值".to_string(),
                required: true,
                default: None,
            },
        ],
        move |args: JsonValue| -> Result<String, String> {
            let key = args["key"].as_str().ok_or("缺少 key")?;
            let value = args["value"].as_str().ok_or("缺少 value")?;

            let mut store = store_clone.lock().unwrap();
            store.insert(key.to_string(), value.to_string());

            Ok(format!("已设置: {} = {}", key, value))
        },
    )
}
```

---

### 2. 异步工具

处理异步操作：

```rust
|args: JsonValue| -> Result<String, String> {
    let url = args["url"].as_str().ok_or("缺少 url")?;

    // 在同步闭包中执行异步代码
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let response = reqwest::get(url)
                .await
                .map_err(|e| e.to_string())?;

            let body = response
                .text()
                .await
                .map_err(|e| e.to_string())?;

            Ok(body)
        })
    })
}
```

---

### 3. 流式输出工具

对于长时间运行的操作，支持进度反馈：

```rust
use std::io::Write;

|args: JsonValue| -> Result<String, String> {
    let count = args["count"].as_i64().unwrap_or(10);

    let mut results = Vec::new();
    for i in 1..=count {
        // 模拟耗时操作
        std::thread::sleep(Duration::from_millis(100));

        // 实时输出进度
        print!(".");
        std::io::stdout().flush().unwrap();

        results.push(format!("Item {}", i));
    }
    println!();

    Ok(results.join(", "))
}
```

---

## 📚 参考资料

### OpenAI Function Schema 格式

```json
{
  "type": "function",
  "function": {
    "name": "tool_name",
    "description": "Tool description for LLM",
    "parameters": {
      "type": "object",
      "properties": {
        "param1": {
          "type": "string",
          "description": "Parameter description"
        },
        "param2": {
          "type": "number",
          "description": "Another parameter"
        }
      },
      "required": ["param1"]
    }
  }
}
```

### Tool 完整 API

```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
    pub handler: Arc<dyn Fn(JsonValue) -> Result<String, String> + Send + Sync>,
}

impl Tool {
    /// 创建新工具
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Vec<Parameter>,
        handler: impl Fn(JsonValue) -> Result<String, String> + Send + Sync + 'static,
    ) -> Self { ... }

    /// 生成 OpenAI Function Schema
    pub fn to_function_schema(&self) -> JsonValue { ... }

    /// 验证参数并执行
    pub fn execute(&self, args: JsonValue) -> Result<String, String> { ... }
}

pub struct Parameter {
    pub name: String,
    pub param_type: ParameterType,
    pub description: String,
    pub required: bool,
    pub default: Option<JsonValue>,
}

pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}
```

### ToolRegistry API

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    /// 创建空注册表
    pub fn new() -> Self { ... }

    /// 注册工具
    pub fn register(&mut self, tool: Tool) { ... }

    /// 获取工具
    pub fn get(&self, name: &str) -> Option<&Tool> { ... }

    /// 列出所有工具名称
    pub fn list_tools(&self) -> Vec<&str> { ... }

    /// 执行工具
    pub fn execute(&self, name: &str, args: JsonValue) -> Result<String, String> { ... }

    /// 获取所有工具的 Function Schema
    pub fn get_function_schemas(&self) -> Vec<JsonValue> { ... }
}
```

---

## 🐛 调试技巧

### 1. 打印调试信息

```rust
|args: JsonValue| -> Result<String, String> {
    // 打印接收到的参数
    eprintln!("DEBUG: 工具参数 = {:?}", args);

    let result = do_something(args)?;

    // 打印结果
    eprintln!("DEBUG: 工具结果 = {:?}", result);

    Ok(result)
}
```

### 2. 查看 LLM 的工具选择

启用详细日志：

```bash
export RUST_LOG=debug
cargo run
```

### 3. 手动测试工具

使用 `/tools call` 命令测试：

```bash
> /tools call my_tool {"param": "value"}
```

---

## 💡 常见问题

### Q: 工具执行失败，但没有错误信息？

**A**: 确保处理函数返回的 `Err(String)` 包含有用的错误信息：

```rust
// ❌ 不好
Err("错误".to_string())

// ✅ 好
Err(format!("参数验证失败: 期望 1-100，实际 {}", value))
```

---

### Q: 如何让 LLM 优先选择我的工具？

**A**: 编写清晰、详细的工具描述：

```rust
// ❌ 不好
"获取数据"

// ✅ 好
"从 API 获取用户数据，支持按 ID、用户名或邮箱查询"
```

---

### Q: 工具参数太多，如何简化？

**A**: 使用对象参数或提供合理的默认值：

```rust
Parameter {
    name: "options".to_string(),
    param_type: ParameterType::Object,
    description: "查询选项：{limit: 10, offset: 0, sort: 'asc'}".to_string(),
    required: false,
    default: Some(json!({"limit": 10, "offset": 0, "sort": "asc"})),
}
```

---

## 📞 获取帮助

- 查看现有工具实现: `src/builtin_tools.rs`
- 查看测试示例: `tests/test_function_calling_e2e.rs`
- 查看用户文档: [Tool Calling User Guide](TOOL_CALLING_USER_GUIDE.md)
- 报告 Bug: [GitHub Issues](https://github.com/your-repo/realconsole/issues)

---

**祝开发顺利！** 🚀
