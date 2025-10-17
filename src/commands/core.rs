//! 核心命令实现
//!
//! 提供基础命令：
//! - /help - 显示帮助信息
//! - /quit - 退出程序
//! - /version - 显示版本信息

use crate::command::{Command, CommandRegistry};
use colored::Colorize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 注册核心命令
pub fn register_core_commands(registry: &mut CommandRegistry) {
    // /help 命令
    let help_cmd = Command::from_fn("help", "显示帮助信息", cmd_help)
        .with_aliases(vec!["h".to_string(), "?".to_string()])
        .with_group("core");
    registry.register(help_cmd);

    // /quit 命令
    let quit_cmd = Command::from_fn("quit", "退出程序", cmd_quit)
        .with_aliases(vec!["q".to_string(), "exit".to_string()])
        .with_group("core");
    registry.register(quit_cmd);

    // /version 命令
    let version_cmd = Command::from_fn("version", "显示版本信息", cmd_version)
        .with_aliases(vec!["v".to_string()])
        .with_group("core");
    registry.register(version_cmd);

    // /commands 命令（列出所有命令）
    let commands_cmd = Command::from_fn("commands", "列出所有可用命令", cmd_commands)
        .with_group("core");
    registry.register(commands_cmd);

    // /examples 命令（使用示例）
    let examples_cmd = Command::from_fn("examples", "查看使用示例", cmd_examples)
        .with_aliases(vec!["ex".to_string()])
        .with_group("core");
    registry.register(examples_cmd);

    // /quickref 命令（快速参考）
    let quickref_cmd = Command::from_fn("quickref", "快速参考卡片", cmd_quickref)
        .with_aliases(vec!["qr".to_string()])
        .with_group("core");
    registry.register(quickref_cmd);
}

/// /help 命令处理器
fn cmd_help(arg: &str) -> String {
    let arg = arg.trim();

    // 根据参数路由到不同帮助页面
    match arg {
        "" => cmd_help_quick(),
        "all" => cmd_help_all(),
        "tools" => cmd_help_tools(),
        "memory" => cmd_help_memory(),
        "log" => cmd_help_log(),
        "shell" => cmd_help_shell(),
        _ => format!(
            "{} 未知的帮助主题: {}\n使用 {} 查看可用主题",
            "✗".red(),
            arg.yellow(),
            "/help".cyan()
        ),
    }
}

/// 快速帮助（简洁版）
fn cmd_help_quick() -> String {
    format!(
        r#"{}

{}
  直接输入问题即可，无需命令前缀
  {} 计算 2 的 10 次方
  {} 用 Rust 写一个 hello world

{}
  常见命令可直接输入，无需前缀（智能识别）
  {}         列出文件（自动识别）
  {}         显示当前目录
  {}   查看Git状态
  {}        强制Shell执行

{}
  {}      显示此帮助
  {}  显示所有命令（详细）
  {}   查看使用示例
  {}   快速参考卡片
  {}      退出程序

{}
  {}        列出所有工具
  {}   调用工具

{}
  {}    查看最近对话
  {}        查看执行统计

{}
  使用 {} 查看命令详情
  系统自动识别命令类型，使用 {} 查看路由说明
"#,
        format!("{} {}", "RealConsole".bold().cyan(), format!("v{}", VERSION).dimmed()),
        "💬 智能对话:".bold().green(),
        "示例:".dimmed(),
        "示例:".dimmed(),
        "🚀 智能命令路由 (Phase 10.1):".bold().cyan(),
        "ls".green(),
        "pwd".green(),
        "git status".green(),
        "!ls -la".yellow(),
        "⚡ 快速命令:".bold(),
        "/help".green(),
        "/help all".green(),
        "/examples".green(),
        "/quickref".green(),
        "/quit".green(),
        "🛠️ 工具调用:".bold().yellow(),
        "/tools".green(),
        "/tools call <name> <args>".green(),
        "💾 记忆与日志:".bold(),
        "/memory recent".green(),
        "/log stats".green(),
        "提示:".bold().dimmed(),
        "/help <命令>".cyan(),
        "/help shell".cyan()
    )
}

/// 详细帮助（完整文档）
fn cmd_help_all() -> String {
    format!(
        r#"{}

━━━ 核心命令 ━━━
  {} [主题]       显示帮助信息
    别名: /h, /?
    主题: all, tools, memory, log, shell

  {}              退出程序
    别名: /q, /exit

  {}           显示版本信息
    别名: /v

  {}         列出所有可用命令

━━━ LLM 命令 ━━━
  {}               显示 LLM 状态
  {} <问题>        直接提问（使用 fallback）

━━━ 工具管理 ━━━
  {}                    列出所有工具
  {}             列出所有工具（同上）
  {} <name>        查看工具详情
  {} <name> <args> 调用工具

  示例:
    /tools call calculator {{"expression": "10+5"}}
    /tools info http_get

━━━ 记忆系统 ━━━
  {} [n]        显示最近 n 条对话（默认5）
  {} <关键词>   搜索对话历史
  {}             清空记忆
  {} [文件]       保存到文件

━━━ 执行日志 ━━━
  {} [n]           显示最近 n 条日志
  {} <关键词>      搜索日志
  {}                显示统计信息
  {}               显示失败记录

━━━ Shell 执行 ━━━
  {}                   执行 shell 命令

  安全限制: 禁止 rm -rf /, sudo, shutdown 等危险命令
  超时时间: 30 秒

  示例:
    !ls -la
    !pwd
    !echo "hello"

更多信息: 使用 {} 查看使用示例
"#,
        format!("{} - 完整命令参考", "RealConsole".bold().cyan()),
        "/help".green(),
        "/quit".green(),
        "/version".green(),
        "/commands".green(),
        "/llm".green(),
        "/ask".green(),
        "/tools".green(),
        "/tools list".green(),
        "/tools info".green(),
        "/tools call".green(),
        "/memory recent".green(),
        "/memory search".green(),
        "/memory clear".green(),
        "/memory save".green(),
        "/log recent".green(),
        "/log search".green(),
        "/log stats".green(),
        "/log failed".green(),
        "!<命令>".yellow(),
        "/examples".cyan()
    )
}

/// 工具命令帮助
fn cmd_help_tools() -> String {
    format!(
        r#"{}

用法:
  {}                     列出所有可用工具
  {}                同上
  {} <工具名>       查看工具详细信息
  {} <工具名> <JSON参数>  调用工具

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
  /tools call calculator {{"expression": "2^10"}}

  # 获取网页内容
  /tools call http_get {{"url": "https://httpbin.org/get"}}

  # 解析 JSON
  /tools call json_parse {{"text": "{{\"name\": \"John\"}}"}}

提示:
  • 工具调用支持迭代模式（最多5轮）
  • 每轮最多调用3个工具（并行）
  • 在配置文件中可调整限制
"#,
        "🛠️ 工具管理命令".bold(),
        "/tools".green(),
        "/tools list".green(),
        "/tools info".green(),
        "/tools call".green()
    )
}

/// 记忆命令帮助
fn cmd_help_memory() -> String {
    format!(
        r#"{}

用法:
  {} [数量]        显示最近 n 条对话（默认5）
  {} <关键词>   搜索包含关键词的对话
  {}             清空所有记忆
  {} [文件]       保存记忆到文件（默认 memory.json）

示例:
  /memory recent 10       # 显示最近10条
  /memory search "Rust"   # 搜索包含 Rust 的对话
  /memory save history.json  # 保存到指定文件

提示:
  • 记忆容量默认100条（环形缓冲区）
  • 可在配置文件中调整容量
  • 支持持久化到文件
"#,
        "💾 记忆系统命令".bold(),
        "/memory recent".green(),
        "/memory search".green(),
        "/memory clear".green(),
        "/memory save".green()
    )
}

/// 日志命令帮助
fn cmd_help_log() -> String {
    format!(
        r#"{}

用法:
  {} [数量]           显示最近 n 条日志（默认10）
  {} <关键词>      搜索包含关键词的日志
  {}                显示统计信息（总数、成功率等）
  {}               显示所有失败的命令

示例:
  /log recent 20          # 显示最近20条
  /log search "error"     # 搜索错误日志
  /log stats              # 查看统计
  /log failed             # 查看失败记录

提示:
  • 日志包含命令、类型、耗时、状态
  • 日志容量默认1000条
  • 用于分析命令执行情况
"#,
        "📊 执行日志命令".bold(),
        "/log recent".green(),
        "/log search".green(),
        "/log stats".green(),
        "/log failed".green()
    )
}

/// Shell 命令帮助
fn cmd_help_shell() -> String {
    format!(
        r#"{}

🚀 智能命令路由 (Phase 10.1):
  RealConsole 现在支持智能识别常见命令，无需 ! 前缀

  ✓ 直接输入常见命令（80+ 支持）:
    {}                  自动识别为 shell 命令
    {}                 自动识别
    {}         自动识别
    {}         自动识别
    {}       自动识别

  ✓ 强制 Shell 执行（! 前缀）:
    {}             强制作为 shell 命令执行

  ✓ 系统命令（/ 前缀）:
    {}             执行系统内置命令

  ✓ 自然语言（智能识别）:
    {}           自动路由到 LLM
    {}       自动路由到 LLM

路由优先级:
  1. 强制 Shell (!) - 最高优先级
  2. 系统命令 (/) - 次高优先级
  3. 常见 Shell - 智能识别（80+ 命令）
  4. 自然语言 - 兜底处理

安全限制:
  以下命令被禁止执行（黑名单）：
    • rm -rf /           - 删除根目录
    • sudo <任意命令>     - 权限提升
    • shutdown/reboot    - 系统关机/重启
    • mkfs               - 格式化磁盘
    • dd if=/dev/*       - 直接写磁盘
    • > /dev/sd*         - 写入设备文件

执行限制:
  • 超时时间: 30 秒
  • 输出限制: 100 KB
  • 跨平台: Unix(/bin/sh) 和 Windows(cmd)

提示:
  • 系统会自动识别命令类型，无需记忆前缀
  • 危险命令会被拒绝并显示详细错误
  • 中文疑问句自动识别为自然语言
"#,
        "🔧 Shell 执行 & 智能路由".bold(),
        "ls".green(),
        "pwd".green(),
        "git status".green(),
        "docker ps".green(),
        "cargo build".green(),
        "!ls -la".yellow(),
        "/help".cyan(),
        "你好".dimmed(),
        "帮我分析这个错误".dimmed()
    )
}

/// /quit 命令处理器
fn cmd_quit(_arg: &str) -> String {
    // 返回特殊标记，由 REPL 检测并退出
    "__QUIT__".to_string()
}

/// /version 命令处理器
fn cmd_version(_arg: &str) -> String {
    format!(
        "{} {}\n{}\n\n{}\n{}\n{}\n{}\n{}\n{}\n\n{}\n  {}\n  {}\n  {}",
        "RealConsole".bold(),
        VERSION.cyan(),
        "融合东方哲学智慧的智能 CLI Agent (Rust 实现)".dimmed(),
        "✓ Phase 1: 最小内核".green(),
        "✓ Phase 2: 流式输出 + Shell 执行".green(),
        "✓ Phase 3: Intent DSL + 实体提取".green(),
        "✓ Phase 4: 工具调用系统 + 记忆/日志".green(),
        "✓ Phase 5: 增强工具系统 + 性能优化".green(),
        "226 tests passing ✓".dimmed(),
        "功能特性:".bold(),
        "🛠️ 工具调用 (14 个工具: 5 基础 + 9 高级)".yellow(),
        "🧠 Intent DSL (16 个内置意图)".yellow(),
        "💾 记忆系统 + 执行日志".yellow()
    )
}

/// /commands 命令处理器
fn cmd_commands(_arg: &str) -> String {
    // 这个命令需要访问 registry，暂时返回占位符
    // 实际实现需要在运行时注入 registry 引用
    format!(
        "使用 {} 或 {} 查看所有可用命令",
        "/help".cyan(),
        "/help all".cyan()
    )
}

/// /examples 命令处理器
fn cmd_examples(_arg: &str) -> String {
    format!(
        r#"{}

━━━ 智能对话 ━━━
  计算 2 的 10 次方
  用 Rust 写一个 hello world
  解释一下什么是闭包
  推荐一些 Rust 学习资源

━━━ 智能命令路由 (新!) ━━━
  ls                           # 自动识别为 shell 命令
  pwd                          # 无需 ! 前缀
  git status                   # 常见命令直接执行
  docker ps -a                 # 80+ 命令自动识别
  cargo build --release        # 开发工具命令
  !custom_script.sh            # 强制 shell 执行

━━━ 工具调用 ━━━
  /tools call calculator {{"expression": "sqrt(144)"}}
  /tools call datetime {{"format": "RFC3339"}}
  /tools call http_get {{"url": "https://api.github.com/users/octocat"}}
  /tools call json_parse {{"text": "{{\"name\": \"John\", \"age\": 30}}"}}
  /tools call base64 {{"operation": "encode", "text": "Hello World"}}

━━━ 记忆查询 ━━━
  /memory recent 10
  /memory search "Rust"
  /memory save my_history.json

━━━ 日志分析 ━━━
  /log stats
  /log failed
  /log recent 20
  /log search "error"

{}
  复制任意示例直接粘贴即可使用
  使用 {} 查看各命令详细说明
  使用 {} 查看智能路由说明
"#,
        "💡 RealConsole 使用示例".bold(),
        "提示:".bold().dimmed(),
        "/help <命令>".cyan(),
        "/help shell".cyan()
    )
}

/// /quickref 命令处理器
fn cmd_quickref(_arg: &str) -> String {
    format!(
        r#"
╭─────────────── {} ───────────────╮
│                                                     │
│  {}        直接输入问题                        │
│  {}      !<命令>                            │
│  {}        /<命令>                            │
│                                                     │
│  {}:                                          │
│    {}         帮助                               │
│    {}        工具列表                           │
│    {}       记忆管理                           │
│    {}          日志查询                           │
│    {}         退出                               │
│                                                     │
│  {}:                                            │
│    {}        取消当前操作                       │
│    {}        退出程序                           │
│    {}          历史命令                            │
│                                                     │
│  {}: {} 或 {} │
╰─────────────────────────────────────────────────────╯
"#,
        "RealConsole 快速参考".cyan(),
        "智能对话".bold(),
        "执行 Shell".bold(),
        "系统命令".bold(),
        "常用命令".bold(),
        "/help".green(),
        "/tools".green(),
        "/memory".green(),
        "/log".green(),
        "/quit".green(),
        "快捷键".bold(),
        "Ctrl+C".yellow(),
        "Ctrl+D".yellow(),
        "↑/↓".yellow(),
        "更多".bold(),
        "/help all".cyan(),
        "/examples".cyan()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_command() {
        let output = cmd_help("");
        assert!(output.contains("RealConsole"));
        assert!(output.contains("/help"));
        assert!(output.contains("智能对话"));
    }

    #[test]
    fn test_help_all() {
        let output = cmd_help("all");
        assert!(output.contains("完整命令参考"));
        assert!(output.contains("/tools"));
        assert!(output.contains("/memory"));
    }

    #[test]
    fn test_help_tools() {
        let output = cmd_help("tools");
        assert!(output.contains("工具管理命令"));
        assert!(output.contains("calculator"));
    }

    #[test]
    fn test_examples_command() {
        let output = cmd_examples("");
        assert!(output.contains("使用示例"));
        assert!(output.contains("智能对话"));
    }

    #[test]
    fn test_quickref_command() {
        let output = cmd_quickref("");
        assert!(output.contains("快速参考"));
        assert!(output.contains("/help"));
    }

    #[test]
    fn test_quit_command() {
        let output = cmd_quit("");
        assert_eq!(output, "__QUIT__");
    }

    #[test]
    fn test_version_command() {
        let output = cmd_version("");
        assert!(output.contains("RealConsole"));
        assert!(output.contains(VERSION));
    }

    #[test]
    fn test_register_core_commands() {
        let mut registry = CommandRegistry::new();
        register_core_commands(&mut registry);

        assert!(registry.get("help").is_some());
        assert!(registry.get("quit").is_some());
        assert!(registry.get("version").is_some());
        assert!(registry.get("h").is_some()); // 别名测试
    }
}
