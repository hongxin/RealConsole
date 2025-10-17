# 多轮对话使用指南

本指南介绍 RealConsole 的智能多轮对话功能，帮助用户通过自然语言交互完成复杂任务。

## 快速开始

### 基本概念

**多轮对话**是一种交互模式，系统会：
1. 识别你的意图
2. 智能收集缺失的参数
3. 确认后执行操作

### 触发对话

使用特定关键词即可启动多轮对话：

```bash
# 日志分析
> 分析日志

# 文件操作
> 删除文件
> 移动文件
> 复制文件
```

## 支持的场景

### 1. 日志分析

**触发词**：`分析日志`、`查看日志`

**需要的参数**：
- `file_path` (必需) - 日志文件路径
- `keyword` (必需) - 搜索关键词
- `time_range` (可选) - 时间范围

**示例对话**：
```
用户> 分析日志

▶ 启动多轮对话
输入 'cancel' 或 'exit' 可以随时取消对话

❓ 你想分析哪个日志文件呢？
  💡 支持绝对路径或相对路径
  📝 例如: /var/log/app.log

用户> /var/log/system.log

✓ 已记录

❓ 你想搜索什么关键词？
  💡 支持正则表达式
  📝 例如: ERROR|WARN

用户> ERROR

✓ 已记录

📋 参数摘要:
  file_path = "/var/log/system.log"
  keyword = "ERROR"

确认执行？[y/N]:

用户> y

✓ 执行成功

[日志分析结果...]
```

### 2. 文件操作

**触发词**：`删除文件`、`移动文件`、`复制文件`

**需要的参数**：
- `operation` (必需) - 操作类型 (delete/move/copy)
- `source` (必需) - 源文件路径
- `destination` (可选) - 目标路径（移动/复制时需要）

**示例对话**：
```
用户> 删除文件

▶ 启动多轮对话

❓ 你想执行什么操作呢？
  💡 delete, move, copy
  📝 例如: delete

用户> delete

✓ 已记录

❓ 请提供要操作的文件路径
  📝 例如: /path/to/file.txt

用户> /tmp/test.txt

✓ 已记录

📋 参数摘要:
  operation = "delete"
  source = "/tmp/test.txt"

确认执行？[y/N]:
```

## 智能功能

### 自动参数提取

系统会尝试从你的输入中自动提取参数：

```
用户> 分析 /var/log/app.log 的错误

✓ 系统自动识别：
  - file_path = /var/log/app.log
  - keyword = 错误

❓ 还需要指定时间范围吗？（可选）
```

### 智能提问

系统使用 LLM 生成自然、友好的提问：

```
❌ 生硬的提问：
  请输入 time_range 参数（格式：YYYY-MM-DD）

✅ 智能提问：
  你想查看哪个时间段的日志呢？比如今天的或者最近一周的？
```

### 参数验证

系统会自动验证参数的合法性：

```
用户> 分析日志

❓ 请提供日志文件路径

用户> /nonexistent/file.log

✗ 参数收集失败: 文件不存在：/nonexistent/file.log

❓ 请提供一个有效的文件路径
```

## 对话控制

### 取消对话

任何时候输入以下命令可以取消对话：
- `cancel`
- `exit`
- `quit`

```
用户> cancel

✓ 对话已取消
```

### 确认/拒绝

参数收集完成后，系统会请求确认：
- 输入 `y` 或 `yes` 确认执行
- 输入 `n` 或 `no` 取消执行

```
确认执行？[y/N]:

用户> n

✓ 对话已取消
```

## 参数类型

系统支持多种参数类型：

| 类型 | 说明 | 示例 |
|------|------|------|
| String | 字符串 | "ERROR" |
| Integer | 整数 | 42 |
| Float | 浮点数 | 3.14 |
| Boolean | 布尔值 | true/false |
| Path | 路径 | /var/log/app.log |
| Directory | 目录 | /home/user |
| File | 文件 | config.yaml |
| List | 列表 | ["a", "b", "c"] |
| Enum | 枚举 | delete/move/copy |

## 最佳实践

### 1. 提供完整信息

尽量在初次输入时提供完整信息：

```
✅ 推荐：
用户> 分析 /var/log/app.log 中的 ERROR

❌ 不推荐：
用户> 分析日志
（然后需要多轮询问）
```

### 2. 使用绝对路径

文件路径使用绝对路径更可靠：

```
✅ 推荐：
/var/log/application.log

❌ 不推荐：
../logs/app.log
```

### 3. 清晰的关键词

使用明确的搜索关键词：

```
✅ 推荐：
ERROR|WARN|FATAL

❌ 不推荐：
错误
```

## 故障排查

### 对话未启动

**问题**：输入关键词后没有启动对话

**解决方案**：
1. 检查是否使用了正确的触发词
2. 确保 LLM 已正确配置
3. 查看日志输出是否有错误

### 参数提取失败

**问题**：系统无法自动提取参数

**解决方案**：
1. 使用更明确的表达
2. 在后续对话中手动提供参数
3. 系统会逐个询问缺失的参数

### 执行失败

**问题**：参数收集完成但执行失败

**解决方案**：
1. 检查文件路径是否存在
2. 确认有相应的操作权限
3. 查看详细的错误信息

## 配置选项

在 `realconsole.yaml` 中可以调整对话相关配置：

```yaml
conversation:
  # 对话超时时间（秒）
  timeout: 300

  # 是否启用 LLM 参数提取
  llm_extraction: true

  # 是否启用智能提问
  smart_questions: true
```

## 示例场景

### 场景1：日志故障排查

```
用户> 分析 /var/log/nginx/error.log 的 502 错误

✓ 自动提取参数

❓ 需要查看特定时间段吗？

用户> 最近1小时

✓ 已记录

📋 执行命令:
grep -i '502' /var/log/nginx/error.log | grep '$(date -d "1 hour ago")'

确认执行？[y/N]: y

✓ 执行成功
[显示最近1小时的502错误日志...]
```

### 场景2：批量文件清理

```
用户> 删除文件

❓ 你想删除哪个文件？

用户> /tmp/old_logs/*.log

✓ 已记录

📋 参数摘要:
  operation = "delete"
  source = "/tmp/old_logs/*.log"

⚠️ 警告：将删除多个文件

确认执行？[y/N]: y

✓ 执行成功
已删除 15 个日志文件
```

## 进阶技巧

### 使用正则表达式

在关键词搜索中使用正则表达式：

```
用户> 分析日志

❓ 搜索关键词？

用户> (ERROR|FATAL|CRITICAL).*timeout

✓ 使用正则匹配错误和超时相关的日志
```

### 组合多个条件

```
用户> 分析 /var/log/app.log 的错误，最近24小时

✓ 自动识别：
  - file_path = /var/log/app.log
  - keyword = 错误
  - time_range = 最近24小时
```

## 反馈与改进

遇到问题或有改进建议？

- 📧 提交 Issue：https://github.com/hongxin/realconsole/issues
- 📖 查看文档：https://github.com/hongxin/realconsole/docs
- 💬 社区讨论：https://github.com/hongxin/realconsole/discussions

---

最后更新：2025-10-17
