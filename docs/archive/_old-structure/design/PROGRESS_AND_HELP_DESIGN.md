# 进度指示器与帮助系统增强设计

**版本**: v1.0
**日期**: 2025-10-15
**阶段**: Phase 5.3 Week 2 Day 4
**状态**: 设计中

---

## 1. 概述

本文档描述 RealConsole v0.5.0 的进度指示器系统和帮助系统增强设计，旨在提升用户在长时间操作中的体验感知和获取帮助的效率。

### 1.1 设计目标

**进度指示器**:
- ✅ 实时反馈：用户清楚知道系统在运行
- ✅ 时间感知：长时间操作有预期
- ✅ 可取消性：用户可中断长时间操作
- ✅ 性能优化：不影响实际执行速度

**帮助系统**:
- ✅ 上下文敏感：根据用户当前状态提供相关帮助
- ✅ 示例丰富：每个命令都有实际可用示例
- ✅ 快速查阅：快速参考卡片，一屏显示关键信息
- ✅ 渐进式学习：从基础到高级

### 1.2 用户场景

**场景 1：LLM 思考中**
```
用户: 用 Rust 实现一个二叉树
RealConsole> ⠋ 正在思考... (2s)
```

**场景 2：工具执行中**
```
用户: 下载 https://example.com/large-file.zip
RealConsole> ⠙ 正在下载... (15s) [████████░░░░░░░░] 45%
```

**场景 3：首次使用帮助**
```
用户: /help
RealConsole> 显示简洁的快速入门指南

用户: /help advanced
RealConsole> 显示高级功能详细说明
```

---

## 2. 进度指示器系统

### 2.1 设计原则

1. **非侵入性**: 不阻塞主流程，不影响性能
2. **信息丰富**: 显示状态、耗时、进度（如果可获取）
3. **视觉友好**: 使用 spinner、进度条等视觉元素
4. **可中断性**: Ctrl+C 优雅中断

### 2.2 Spinner 动画

#### 动画帧设计

```rust
pub struct Spinner {
    frames: Vec<&'static str>,
    current: AtomicUsize,
    message: String,
    start_time: Instant,
}

impl Spinner {
    // Braille spinner（流畅、现代）
    const FRAMES: &'static [&'static str] = &[
        "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"
    ];

    pub fn new(message: impl Into<String>) -> Self;
    pub fn tick(&self);
    pub fn update_message(&self, message: impl Into<String>);
    pub fn finish(&self, final_message: impl Into<String>);
}
```

**使用示例**:
```rust
let spinner = Spinner::new("正在思考...");
let handle = spinner.start();

// 长时间操作
let result = llm.chat(input).await;

spinner.finish("✓ 完成");
handle.join();
```

#### 输出效果

```
⠋ 正在思考... (2s)
⠙ 正在思考... (3s)
⠹ 正在思考... (4s)
✓ 完成 (4.2s)
```

### 2.3 进度条

#### 进度条设计

```rust
pub struct ProgressBar {
    total: usize,
    current: AtomicUsize,
    width: usize,
    message: String,
}

impl ProgressBar {
    pub fn new(total: usize, message: impl Into<String>) -> Self;
    pub fn inc(&self, delta: usize);
    pub fn set(&self, value: usize);
    pub fn finish(&self);
}
```

**使用场景**:
- 批量工具调用（执行多个工具）
- 文件下载/上传
- 批量处理任务

**输出效果**:
```
正在处理... [████████████░░░░░░░░] 60% (3/5)
```

### 2.4 LLM 流式输出增强

#### 当前实现

```rust
manager.chat_stream(text, |token| {
    print!("{}", token);
    let _ = io::stdout().flush();
}).await
```

**问题**:
- 无初始反馈（首个 token 前无提示）
- 无耗时显示
- 无速度感知（tokens/s）

#### 改进方案

```rust
// 1. 显示初始提示
print!("{} ", "AI:".dimmed());
let start = Instant::now();

// 2. 流式输出（带统计）
let mut token_count = 0;
manager.chat_stream(text, |token| {
    print!("{}", token);
    token_count += 1;
    let _ = io::stdout().flush();
}).await?;

// 3. 显示统计信息
let elapsed = start.elapsed();
let tokens_per_sec = token_count as f64 / elapsed.as_secs_f64();
println!("\n{} {:.1} tokens/s, {:.1}s",
    "ⓘ".dimmed(),
    tokens_per_sec.to_string().dimmed(),
    elapsed.as_secs_f64().to_string().dimmed()
);
```

**输出效果**:
```
AI: 在Rust中实现二叉树，你可以使用结构体...（流式输出）
ⓘ 45.2 tokens/s, 3.2s
```

### 2.5 取消操作支持

#### Signal 处理

```rust
use tokio::signal;

pub async fn with_cancellation<F, T>(
    future: F,
    on_cancel: impl FnOnce(),
) -> Result<T, Cancelled>
where
    F: Future<Output = T>,
{
    tokio::select! {
        result = future => Ok(result),
        _ = signal::ctrl_c() => {
            on_cancel();
            Err(Cancelled)
        }
    }
}
```

**使用示例**:
```rust
match with_cancellation(
    llm.chat(input),
    || println!("\n{}", "操作已取消".yellow())
).await {
    Ok(result) => result,
    Err(Cancelled) => return,
}
```

**输出效果**:
```
⠋ 正在思考... (5s)
^C
操作已取消
```

### 2.6 实现优先级

由于进度指示器涉及复杂的异步处理和终端控制，考虑到时间和复杂度，我们采用**渐进式实现**策略：

#### Phase 1（本次实现）：基础增强
- ✅ LLM 流式输出统计（tokens/s, 耗时）
- ✅ 初始提示优化
- ✅ 文档化设计（为未来实现铺路）

#### Phase 2（未来）：完整进度系统
- ⏳ Spinner 动画（需要 indicatif crate）
- ⏳ 进度条（需要 indicatif crate）
- ⏳ Ctrl+C 优雅取消（需要 ctrlc crate）

**原因**:
1. **依赖考虑**: indicatif 是重量级依赖，需要评估
2. **终端兼容**: 不同终端对 ANSI 控制码支持不同
3. **实用优先**: 统计信息已能大幅提升用户体验

---

## 3. 帮助系统增强

### 3.1 多层次帮助

#### 快速帮助（/help）

**设计原则**: 一屏显示，新手友好

```
RealConsole v0.5.0 - 智能 CLI Agent

💬 智能对话:
  直接输入问题即可，无需命令前缀
  示例: 计算 2 的 10 次方
  示例: 用 Rust 写一个 hello world

⚡ 快速命令:
  /help      显示此帮助
  /help all  显示所有命令（详细）
  /examples  查看使用示例
  /quit      退出程序

🛠️ 工具调用:
  /tools        列出所有工具
  /tools call <name> <args>

💾 记忆与日志:
  /memory recent    查看最近对话
  /log stats        查看执行统计

提示: 使用 /help <命令> 查看命令详情
      使用 ! 前缀执行 shell 命令（受限）
```

#### 详细帮助（/help all）

**设计原则**: 完整文档，分组清晰

```
RealConsole v0.5.0 - 完整命令参考

━━━ 核心命令 ━━━
  /help [主题]       显示帮助信息
    别名: /h, /?
    主题: all, tools, memory, log, shell

  /quit              退出程序
    别名: /q, /exit

  /version           显示版本信息
    别名: /v

━━━ LLM 命令 ━━━
  /llm               显示 LLM 状态
  /ask <问题>        直接提问（使用 fallback）

━━━ 工具管理 ━━━
  /tools                    列出所有工具
  /tools list               列出所有工具（同上）
  /tools info <name>        查看工具详情
  /tools call <name> <args> 调用工具

  示例:
    /tools call calculator {"expression": "10+5"}
    /tools info http_get

━━━ 记忆系统 ━━━
  /memory recent [n]        显示最近 n 条对话（默认5）
  /memory search <关键词>   搜索对话历史
  /memory clear             清空记忆
  /memory save [文件]       保存到文件

━━━ 执行日志 ━━━
  /log recent [n]           显示最近 n 条日志
  /log search <关键词>      搜索日志
  /log stats                显示统计信息
  /log failed               显示失败记录

━━━ Shell 执行 ━━━
  !<命令>                   执行 shell 命令

  安全限制: 禁止 rm -rf /, sudo, shutdown 等危险命令
  超时时间: 30 秒

  示例:
    !ls -la
    !pwd
    !echo "hello"
```

#### 命令特定帮助（/help <命令>）

```
$ realconsole
> /help tools

🛠️ 工具管理命令

用法:
  /tools                     列出所有可用工具
  /tools list                同上
  /tools info <工具名>       查看工具详细信息
  /tools call <工具名> <JSON参数>  调用工具

可用工具 (14个):
  基础工具 (5个):
    • calculator      - 数学计算
    • datetime        - 日期时间
    • uuid_generator  - UUID 生成
    • base64          - Base64 编解码
    • random          - 随机数生成

  高级工具 (9个):
    • http_get        - HTTP GET 请求
    • http_post       - HTTP POST 请求
    • json_parse      - JSON 解析
    • json_query      - JSON 查询 (JQ)
    • text_search     - 文本搜索
    • text_replace    - 文本替换
    • file_read       - 文件读取
    • file_write      - 文件写入
    • sys_info        - 系统信息

示例:
  # 计算数学表达式
  /tools call calculator {"expression": "2^10"}

  # 获取网页内容
  /tools call http_get {"url": "https://httpbin.org/get"}

  # 解析 JSON
  /tools call json_parse {"text": "{\"name\": \"John\"}"}

提示:
  • 工具调用支持迭代模式（最多5轮）
  • 每轮最多调用3个工具（并行）
  • 在配置文件中可调整限制

更多信息: https://docs.realconsole.com/tools
```

### 3.2 示例命令库

#### 示例命令（/examples）

```
💡 RealConsole 使用示例

━━━ 智能对话 ━━━
  计算 2 的 10 次方
  用 Rust 写一个 hello world
  解释一下什么是闭包

━━━ 工具调用 ━━━
  /tools call calculator {"expression": "sqrt(144)"}
  /tools call datetime {"format": "RFC3339"}
  /tools call http_get {"url": "https://api.github.com/users/octocat"}

━━━ 记忆查询 ━━━
  /memory recent 10
  /memory search "Rust"

━━━ 日志分析 ━━━
  /log stats
  /log failed
  /log recent 20

━━━ Shell 命令 ━━━
  !ls -la
  !cat config.yaml
  !git status

提示: 复制任意示例直接粘贴即可使用
```

#### 快速参考卡片（/quickref）

```
╭─────────────── RealConsole 快速参考 ───────────────╮
│                                                     │
│  智能对话        直接输入问题                        │
│  执行 Shell      !<命令>                            │
│  系统命令        /<命令>                            │
│                                                     │
│  常用命令:                                          │
│    /help         帮助                               │
│    /tools        工具列表                           │
│    /memory       记忆管理                           │
│    /log          日志查询                           │
│    /quit         退出                               │
│                                                     │
│  快捷键:                                            │
│    Ctrl+C        取消当前操作                       │
│    Ctrl+D        退出程序                           │
│    ↑/↓          历史命令                            │
│                                                     │
│  更多: /help all 或访问 docs.realconsole.com       │
╰─────────────────────────────────────────────────────╯
```

### 3.3 上下文敏感帮助

#### 错误后的帮助建议

```rust
pub fn suggest_help_for_error(error: &RealError) -> Option<String> {
    match error.code {
        ErrorCode::ToolNotFound => {
            Some("使用 /tools 查看所有可用工具".to_string())
        }
        ErrorCode::ShellDangerousCommand => {
            Some("查看 /help shell 了解 Shell 命令安全限制".to_string())
        }
        ErrorCode::ConfigNotFound => {
            Some("运行 realconsole wizard 创建配置文件".to_string())
        }
        _ => None,
    }
}
```

**输出效果**:
```
✗ [E200] 工具不存在

工具 'my_tool' 不存在

建议修复方案:
  1. 使用 /tools 查看所有可用工具
  2. 检查工具名称拼写是否正确

💡 提示: 输入 /help tools 查看工具使用帮助
```

### 3.4 实现计划

#### Phase 1（本次实现）：帮助内容增强
- ✅ 重写 /help 命令（简洁版）
- ✅ 添加 /help all（完整文档）
- ✅ 添加 /help <主题>（主题帮助）
- ✅ 添加 /examples（示例库）
- ✅ 添加 /quickref（快速参考）

#### Phase 2（未来）：上下文敏感
- ⏳ 根据错误自动提示帮助
- ⏳ 智能补全建议
- ⏳ 首次使用引导

---

## 4. 实现策略

### 4.1 进度指示器

**本次实现范围**:
1. LLM 流式输出统计（tokens/s, 耗时）
2. 初始提示优化（"AI: " 前缀）
3. 完成后统计信息显示

**代码位置**:
- `src/agent.rs` - `handle_text_streaming()` 方法
- `src/llm_manager.rs` - `chat_stream()` 方法

### 4.2 帮助系统

**本次实现范围**:
1. 重写 `cmd_help()` 函数（简洁版）
2. 添加 `cmd_help_all()` 函数（详细版）
3. 添加 `cmd_help_topic()` 函数（主题帮助）
4. 添加 `cmd_examples()` 函数（示例库）
5. 添加 `cmd_quickref()` 函数（快速参考）

**代码位置**:
- `src/commands/core.rs` - 帮助命令实现
- `src/commands/help.rs` - 新增专门的帮助模块（可选）

### 4.3 测试策略

**单元测试**:
- 帮助文本包含关键信息
- 命令别名正确注册
- 示例格式正确

**手动测试**:
- 帮助文本排版美观
- 示例可直接复制使用
- 快速参考卡片对齐正确

---

## 5. 成功标准

### 5.1 进度指示器

- ✅ LLM 流式输出显示 tokens/s
- ✅ 显示总耗时
- ✅ 初始提示清晰（"AI: "）
- ✅ 不影响性能

### 5.2 帮助系统

- ✅ 新手 5 分钟内找到常用命令
- ✅ 每个命令都有可用示例
- ✅ 快速参考一屏显示
- ✅ 帮助文本易读、美观

### 5.3 用户反馈

- ✅ "我知道系统在工作"
- ✅ "帮助很有用，示例可以直接用"
- ✅ "快速参考卡片很方便"

---

## 6. 未来扩展

### 6.1 完整进度系统（Phase 2）

- Spinner 动画（indicatif）
- 进度条（批量操作）
- Ctrl+C 优雅中断
- 嵌套进度（子任务）

### 6.2 智能帮助（Phase 3）

- 错误后自动提示帮助
- 智能补全（bash-like）
- 首次使用交互式教程
- 上下文相关建议

### 6.3 文档集成（Phase 4）

- 在线文档链接
- 本地文档缓存
- 搜索文档（/docs search）
- 版本特定文档

---

**设计完成**: 2025-10-15
**下一步**: 实现帮助系统增强
