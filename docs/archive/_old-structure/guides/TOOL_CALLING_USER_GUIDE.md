# 工具调用用户指南

## 📖 概述

RealConsole 的工具调用（Tool Calling）功能允许 LLM 自动调用预定义的工具来完成任务，如计算、文件操作、获取时间等。这使得 AI 助手能够执行实际操作，而不仅仅是提供文本建议。

### 特性

- **自动工具调用** - LLM 根据用户意图自动选择和调用工具
- **迭代执行** - 支持多轮工具调用（最多 5 轮）
- **安全可控** - 内置工具有严格的安全检查
- **标准兼容** - 使用 OpenAI Function Calling 格式
- **平滑降级** - 不支持工具调用的 LLM 自动回退到普通对话

---

## 🚀 快速开始

### 1. 启用工具调用

编辑配置文件 `smartconsole.yaml`：

```yaml
features:
  tool_calling_enabled: true
  max_tool_iterations: 5        # 最大迭代轮数（默认 5）
  max_tools_per_round: 3        # 每轮最多工具数（默认 3）
```

### 2. 启动 RealConsole

```bash
cargo run --release
```

### 3. 使用自然语言交互

工具调用是**完全自动的**，您只需用自然语言提出需求：

```
> 帮我计算 (10 + 5) * 2 的结果

[LLM 自动调用 calculator 工具]
根据计算结果，(10 + 5) * 2 = 30
```

---

## 🛠️ 内置工具

RealConsole 提供 5 个内置工具：

### 1. calculator - 数学计算

**功能**: 安全地计算数学表达式

**参数**:
- `expression` (必需) - 数学表达式，如 "10+5", "sqrt(16)", "sin(3.14/2)"

**支持的运算**:
- 基本运算: `+`, `-`, `*`, `/`, `%`
- 幂运算: `^` (例如 `2^3` = 8)
- 函数: `sqrt()`, `sin()`, `cos()`, `tan()`, `abs()`, `floor()`, `ceil()`, `round()`
- 常量: `pi`, `e`

**示例**:
```
> 帮我计算 2^10

[LLM 调用 calculator 工具]
2^10 = 1024
```

**安全性**: 使用 `meval` 库进行安全求值，防止代码注入

---

### 2. read_file - 读取文件

**功能**: 读取指定文件的内容

**参数**:
- `path` (必需) - 文件路径

**示例**:
```
> 读取 config.yaml 的内容

[LLM 调用 read_file 工具]
文件内容如下：
features:
  tool_calling_enabled: true
  ...
```

**安全限制**:
- ❌ 阻止读取 `/etc/shadow`
- ❌ 阻止读取私钥文件 (`.pem`, `.key`)
- ❌ 阻止读取密码文件 (`password`, `secret`)
- ✅ 允许读取普通文本文件、配置文件等

---

### 3. write_file - 写入文件

**功能**: 将内容写入指定文件

**参数**:
- `path` (必需) - 文件路径
- `content` (必需) - 文件内容

**示例**:
```
> 创建一个 hello.txt 文件，内容是 "Hello, World!"

[LLM 调用 write_file 工具]
文件已创建：hello.txt
```

**安全限制**:
- ❌ 阻止写入系统目录 (`/etc`, `/bin`, `/usr`, `/var`)
- ❌ 阻止写入根目录 `/`
- ✅ 允许写入用户目录和当前工作目录

---

### 4. list_dir - 列出目录

**功能**: 列出指定目录的内容

**参数**:
- `path` (可选) - 目录路径，默认为当前目录 "."

**示例**:
```
> 列出当前目录的文件

[LLM 调用 list_dir 工具]
当前目录包含以下文件：
- Cargo.toml
- src/
- tests/
- README.md
```

---

### 5. get_datetime - 获取日期时间

**功能**: 获取当前日期和时间

**参数**:
- `format` (可选) - 格式类型：`"datetime"` (默认), `"date"`, `"time"`, `"timestamp"`

**示例**:
```
> 现在几点了？

[LLM 调用 get_datetime 工具]
当前时间是：2025-01-15 14:30:45
```

**格式说明**:
- `datetime`: 完整日期时间 (2025-01-15 14:30:45)
- `date`: 仅日期 (2025-01-15)
- `time`: 仅时间 (14:30:45)
- `timestamp`: Unix 时间戳 (1736951445)

---

## 📋 工具管理命令

除了自动工具调用，您还可以手动管理和调用工具：

### 列出所有工具

```bash
> /tools
可用工具 5 个工具:
  • calculator - 执行数学计算表达式
  • read_file - 读取文件内容
  • write_file - 写入文件内容
  • list_dir - 列出目录内容
  • get_datetime - 获取当前日期时间
```

**快捷命令**: `/tools list` 或 `/tools l`

---

### 查看工具详情

```bash
> /tools info calculator
工具名称: calculator
描述: 执行数学计算表达式

参数:
  • expression [必需] - 数学表达式
    类型: String

Function Schema:
{
  "type": "function",
  "function": {
    "name": "calculator",
    "description": "执行数学计算表达式",
    "parameters": {
      "type": "object",
      "properties": {
        "expression": {
          "type": "string",
          "description": "数学表达式"
        }
      },
      "required": ["expression"]
    }
  }
}
```

**快捷命令**: `/tools info <tool>` 或 `/tools i <tool>`

---

### 手动调用工具

```bash
> /tools call calculator {"expression": "10+5"}
✓ calculator
15.0
```

**用法**: `/tools call <tool_name> <json_args>`

**快捷命令**: `/tools c <tool> <json>`

**参数格式**: 必须是有效的 JSON 对象

**示例**:
```bash
# 计算器
> /tools call calculator {"expression": "2^10"}
1024.0

# 读取文件
> /tools call read_file {"path": "README.md"}
# RealConsole
...

# 写入文件
> /tools call write_file {"path": "test.txt", "content": "Hello"}
文件已成功写入: test.txt

# 列出目录
> /tools call list_dir {"path": "."}
Cargo.toml
src/
...

# 获取时间
> /tools call get_datetime {"format": "date"}
2025-01-15
```

---

## 💡 使用场景

### 场景 1: 数据分析

```
> 我有一个数据集，平均值是 45.6，标准差是 12.3，请帮我计算 95% 置信区间

[LLM 自动调用 calculator 工具 2 次]
95% 置信区间的计算如下：
下限 = 45.6 - 1.96 * 12.3 = 21.492
上限 = 45.6 + 1.96 * 12.3 = 69.708

因此，95% 置信区间为 [21.49, 69.71]
```

---

### 场景 2: 文件批处理

```
> 读取 data.csv 文件，统计其中的数字，然后将结果写入 result.txt

[LLM 执行以下操作]
1. 调用 read_file 读取 data.csv
2. 调用 calculator 进行统计计算
3. 调用 write_file 写入 result.txt

操作完成！结果已保存到 result.txt
```

---

### 场景 3: 项目管理

```
> 列出 src/ 目录下的所有文件，统计文件数量

[LLM 调用 list_dir 工具]
src/ 目录包含以下文件：
- main.rs
- lib.rs
- agent.rs
- config.rs
... 共 15 个文件
```

---

## ⚙️ 配置选项

### 完整配置示例

```yaml
features:
  # 是否启用工具调用（默认 false）
  tool_calling_enabled: true

  # 最大迭代轮数（默认 5）
  # 防止无限循环，LLM 最多进行 5 轮工具调用
  max_tool_iterations: 5

  # 每轮最多工具数（默认 3）
  # 每次 LLM 响应最多调用 3 个工具
  max_tools_per_round: 3
```

### 配置说明

#### `tool_calling_enabled`
- **类型**: boolean
- **默认值**: `false`
- **说明**: 是否启用工具调用功能
- **建议**: 设置为 `true` 以获得完整的智能助手体验

#### `max_tool_iterations`
- **类型**: 整数
- **默认值**: `5`
- **范围**: 1-20
- **说明**: LLM 可以进行的最大工具调用轮数
- **场景**:
  - `3-5`: 适合简单任务
  - `5-10`: 适合中等复杂任务
  - `10+`: 适合复杂的多步骤任务

#### `max_tools_per_round`
- **类型**: 整数
- **默认值**: `3`
- **范围**: 1-10
- **说明**: 每轮 LLM 响应最多可以调用的工具数量
- **建议**: 保持默认值 3，避免单次调用过多工具

---

## 🔍 故障排查

### 问题 1: 工具调用未生效

**症状**: 启用了 `tool_calling_enabled: true`，但 LLM 仍然只是文本回复，不调用工具

**可能原因**:
1. 使用的 LLM 不支持 Function Calling
2. LLM 客户端未实现 `chat_with_tools()` 方法

**解决方案**:
```bash
# 检查 LLM 配置
> /llm
Primary LLM: Deepseek (deepseek-chat)  ✓ 支持工具调用
Fallback LLM: Ollama (qwen3:4b)       ✗ 不支持工具调用

# 如果使用 Ollama，需要切换到支持工具调用的 LLM
# 例如 Deepseek、OpenAI
```

**支持工具调用的 LLM**:
- ✅ Deepseek (deepseek-chat)
- ✅ OpenAI (gpt-3.5-turbo, gpt-4)
- ❌ Ollama (大部分模型不支持，会自动降级)

---

### 问题 2: 工具调用超出最大轮数

**症状**: 错误信息 "达到最大迭代次数"

**原因**: LLM 在 5 轮内未能返回最终答案，持续调用工具

**解决方案**:
```yaml
# 增加最大轮数
features:
  max_tool_iterations: 10
```

---

### 问题 3: 工具调用权限错误

**症状**: "文件操作被拒绝" 或 "路径不允许"

**原因**: 尝试访问受保护的系统文件或目录

**解决方案**:
- 确保操作的是用户目录下的文件
- 不要尝试读取 `/etc/shadow`、私钥等敏感文件
- 不要尝试写入系统目录 (`/etc`, `/bin`, `/usr`)

---

### 问题 4: JSON 参数解析错误

**症状**: 手动调用工具时报 "JSON 解析失败"

**原因**: JSON 格式不正确

**解决方案**:
```bash
# ❌ 错误示例（缺少引号）
> /tools call calculator {expression: "10+5"}

# ✅ 正确示例
> /tools call calculator {"expression": "10+5"}

# ✅ 多个参数
> /tools call write_file {"path": "test.txt", "content": "Hello World"}
```

---

## 📚 进阶使用

### 调试工具调用

如果想查看 LLM 的工具调用过程，可以查看日志：

```bash
> /log recent 10

执行日志（最近 10 条）:
2025-01-15 14:30:45 | Text | ✓ | 1.2s | 帮我计算...
  → LLM 调用 calculator 工具
  → 返回结果: 1024.0
2025-01-15 14:31:12 | Text | ✓ | 2.5s | 读取文件...
  → LLM 调用 read_file 工具
  → 返回结果: 文件内容...
```

### 统计工具调用

```bash
> /stats llm

LLM 统计:
总调用: 150 次
成功: 145 次
失败: 5 次
重试: 12 次
工具调用: 78 次
平均延迟: 1.8s
```

---

## 🎯 最佳实践

### 1. 明确描述任务

**推荐**:
```
> 计算 2 的 10 次方
```

**不推荐**:
```
> 算一下那个什么...就是 2 的很多次方
```

---

### 2. 拆解复杂任务

**推荐**:
```
> 第一步，读取 data.csv 文件
> 好的，现在计算平均值
> 最后，将结果写入 result.txt
```

**不推荐**:
```
> 读取文件然后计算各种统计量并生成 10 个不同的报告同时...
```

---

### 3. 验证工具结果

对于重要操作，可以手动验证：

```bash
# LLM 写入文件后
> /tools call read_file {"path": "output.txt"}

# 验证内容是否正确
```

---

## 🔐 安全建议

1. **不要盲目信任 LLM** - 对于重要的文件操作，先查看 LLM 的计划，再确认执行
2. **设置合理的权限** - 使用普通用户运行 RealConsole，不要使用 root
3. **定期备份** - 工具调用可能修改文件，建议定期备份重要数据
4. **审查敏感操作** - 对于涉及敏感数据的操作，使用手动 `/tools call` 而非自动调用

---

## 📞 获取帮助

- 查看所有命令: `/help`
- 查看工具列表: `/tools`
- 查看工具详情: `/tools info <tool_name>`
- 查看项目文档: [开发者指南](TOOL_CALLING_DEVELOPER_GUIDE.md)
- 报告问题: [GitHub Issues](https://github.com/your-repo/realconsole/issues)

---

**祝您使用愉快！** 🎉
