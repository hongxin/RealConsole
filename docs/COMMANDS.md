# RealConsole 命令参考手册

本文档详细列出了 RealConsole 支持的所有命令类型，包括直接执行的 Shell 命令和系统命令。

## 命令类型概述

RealConsole 采用智能命令路由系统，支持四种命令类型：

1. **常见 Shell 命令** - 80+ 常用命令，无需前缀直接执行
2. **强制 Shell 命令** - 使用 `!` 前缀强制执行任何 Shell 命令
3. **系统命令** - 使用 `/` 前缀的 RealConsole 特有功能
4. **自然语言** - 智能识别意图或使用 LLM 处理

## 一、常见 Shell 命令（无需前缀）

以下 80+ 常用 Shell 命令可以直接输入执行，无需任何前缀：

### 文件导航
- `ls`, `ll` - 列出文件和目录
- `cd` - 切换目录
- `pwd` - 显示当前目录
- `tree` - 树形显示目录结构

### 文件操作
- `cat` - 显示文件内容
- `less`, `more` - 分页查看文件
- `head`, `tail` - 查看文件开头/结尾
- `touch` - 创建文件
- `mkdir` - 创建目录
- `rm`, `rmdir` - 删除文件/目录
- `cp`, `mv` - 复制/移动文件
- `ln` - 创建链接
- `chmod`, `chown` - 修改权限/所有者

### 文本处理
- `grep` - 文本搜索
- `sed` - 流编辑器
- `awk` - 文本处理工具
- `sort` - 排序
- `uniq` - 去重
- `wc` - 统计行数/单词数
- `cut` - 剪切文本
- `tr` - 字符转换

### 系统信息
- `ps` - 进程状态
- `top`, `htop` - 系统监控
- `df`, `du` - 磁盘使用情况
- `free` - 内存使用情况
- `uptime` - 系统运行时间
- `uname` - 系统信息
- `whoami` - 当前用户

### 网络工具
- `curl` - 网络请求
- `wget` - 下载文件
- `ping` - 网络连通性测试
- `netstat` - 网络状态
- `ssh` - 远程连接
- `scp` - 安全文件传输

### 压缩归档
- `tar` - 归档工具
- `gzip`, `gunzip` - 压缩/解压
- `zip`, `unzip` - ZIP 压缩

### 开发工具
- `git` - 版本控制
- `find` - 文件查找
- `diff` - 文件比较
- `which`, `whereis` - 命令查找
- `man` - 手册页

### 其他常用
- `echo` - 输出文本
- `date` - 日期时间
- `cal` - 日历
- `clear` - 清屏
- `history` - 命令历史

## 二、强制 Shell 命令（! 前缀）

使用 `!` 前缀可以强制执行任何 Shell 命令，包括不在常见命令列表中的命令：

```bash
!custom_command        # 执行自定义命令
!sudo command          # 执行需要权限的命令（受安全限制）
!pip install package   # Python 包管理
!npm run build         # Node.js 构建命令
```

**安全限制**：危险命令如 `rm -rf /`、`sudo` 等会被阻止执行。

## 三、系统命令（/ 前缀）

### 核心命令
- `/help` 或 `/h` 或 `/?` - 显示帮助信息
- `/quit` 或 `/q` 或 `/exit` - 退出程序
- `/version` - 显示版本信息
- `/commands` - 列出所有可用命令
- `/examples` - 显示使用示例
- `/quickref` - 快速参考手册

### LLM 相关
- `/ask <问题>` - 向 LLM 提问
- `/llm diag` - LLM 诊断信息

### 任务管理
- `/plan <任务>` - 任务分解和规划
- `/execute <任务>` - 执行任务
- `/tasks` - 显示任务列表
- `/task_status <任务ID>` - 查看任务状态

### 日志分析
- `/log-analyze <文件>` 或 `/la` - 分析日志文件
- `/log-tail <文件>` 或 `/lt` - 实时查看日志
- `/log-errors <文件>` 或 `/le` - 查看错误日志

### 系统监控
- `/sys` - 系统概览
- `/cpu` - CPU 使用情况
- `/memory-info` - 内存信息
- `/disk` - 磁盘使用情况
- `/top` - 进程监控

### 执行日志管理
- `/log recent [数量]` - 查看最近的执行日志
- `/log search <关键词>` - 搜索执行日志
- `/log stats` - 执行统计
- `/log type <类型>` - 按类型过滤日志
- `/log failed` - 查看失败的执行
- `/log clear` - 清空执行日志

### 记忆管理
- `/memory recent [数量]` - 查看最近的记忆
- `/memory search <关键词>` - 搜索记忆
- `/memory clear` - 清空记忆
- `/memory dump` - 导出记忆
- `/memory save` - 保存记忆到文件
- `/memory type <类型>` - 按类型过滤记忆

### 统一追踪 (v1.5.0 新增)
- `/trace` 或 `/t` - 显示最近 20 条记录（四维聚合）
- `/trace all [n]` - 显示最近 N 条记录
- `/trace history [n]` 或 `/trace h [n]` - 仅显示 History 维度（统计）
- `/trace log [n]` 或 `/trace l [n]` - 仅显示 log 维度（协同）
- `/trace llm [n]` - 仅显示 llm-log 维度（黑盒）
- `/trace context [n]` 或 `/trace c [n]` - 仅显示 Context 维度（记忆）
- `/trace search <关键词>` 或 `/trace s <关键词>` - 关键词搜索
- `/trace stats` - 显示统计信息

**四维观测体系**：
- 📊 **History**（统计维度）- 命令频率，使用模式
- 🔗 **log**（协同维度）- 端到端执行追踪
- 🤖 **llm-log**（黑盒维度）- LLM API 调用详情
- 💭 **Context**（记忆维度）- 对话上下文状态

### 统计信息
- `/dashboard` - 显示统计仪表板
- `/stats` - 显示详细统计

### 工具管理
- `/tools list` - 列出可用工具
- `/tools call <工具名> <参数>` - 调用工具
- `/tools info <工具名>` - 查看工具信息

### 命令历史
- `/history search <关键词>` - 搜索历史命令
- `/history clear` - 清空历史记录
- `/history stats` - 历史统计

### Git 命令
- `/git-status` 或 `/gs` - 显示 Git 状态
- `/git-diff` 或 `/gd` - 显示 Git 变更
- `/git-branch` 或 `/gb` - 显示分支信息
- `/git-analyze` 或 `/ga` - 分析变更并建议提交信息

### 项目上下文
- `/project` 或 `/proj` - 显示当前项目信息

### 错误修复
- `/fix` - 重试上次失败的 Shell 命令

## 四、自然语言处理

RealConsole 支持自然语言输入，系统会自动识别意图或使用 LLM 处理：

### 意图识别示例
- "列出所有 Rust 文件" → 自动执行 `find . -name "*.rs"`
- "查看系统内存使用" → 自动执行 `free -h`
- "搜索包含 error 的日志" → 自动执行 `grep -i error *.log`

### LLM 对话示例
- "帮我解释这个代码的作用"
- "如何优化这个算法"
- "写一个 Python 函数来计算斐波那契数列"

## 五、多轮对话支持

RealConsole 支持多轮对话，用于需要收集多个参数的操作：

### 支持的对话意图
- **日志分析** - 需要文件路径、关键词、时间范围等参数
- **文件操作** - 需要操作类型、源文件、目标路径等参数

### 对话控制
- `cancel` 或 `exit` - 取消当前对话
- `y` 或 `yes` - 确认执行
- `n` 或 `no` - 拒绝执行

## 六、命令别名系统

RealConsole 为常用命令提供了简短的别名：

| 完整命令 | 别名 | 描述 |
|---------|------|------|
| `/trace` | `/t` | 统一追踪 (v1.5.0) |
| `/git-status` | `/gs` | Git 状态 |
| `/git-diff` | `/gd` | Git 变更 |
| `/git-branch` | `/gb` | Git 分支 |
| `/git-analyze` | `/ga` | Git 分析 |
| `/project` | `/proj` | 项目信息 |
| `/log-analyze` | `/la` | 日志分析 |
| `/log-tail` | `/lt` | 日志跟踪 |
| `/log-errors` | `/le` | 错误日志 |
| `/help` | `/h`, `/?` | 帮助 |
| `/quit` | `/q`, `/exit` | 退出 |

## 七、智能特性

### 智能命令路由
- 自动识别命令类型，无需记忆前缀
- 常见命令直接执行，特殊功能使用系统命令
- 自然语言自动转换为相应操作

### 错误自动修复
- 当 Shell 命令执行失败时，提供修复建议
- 支持交互式选择修复策略
- 学习用户反馈，改进修复效果

### 上下文感知
- 记忆用户操作历史
- 跟踪工作上下文（当前目录、项目类型等）
- 基于上下文提供智能建议

## 八、使用技巧

1. **快速查找命令**：使用 `/commands` 查看所有可用命令
2. **命令补全**：在 REPL 中使用 Tab 键补全命令
3. **历史搜索**：使用 Ctrl+R 搜索命令历史
4. **自然语言优先**：尽量使用自然语言，系统会自动识别意图
5. **错误重试**：使用 `/fix` 重试失败的 Shell 命令

## 九、安全注意事项

- 危险命令（如 `rm -rf /`、`sudo` 等）会被阻止执行
- 系统命令在安全沙箱中执行
- 用户数据保存在本地，不会上传到云端
- 敏感操作需要用户确认

---

**文档版本**: 1.5.0
**最后更新**: 2025-10-23
**维护者**: RealConsole 开发团队
**新增功能**: v1.5.0 新增 `/trace` 统一追踪命令，详见 `docs/04-reports/trace-command-design.md`