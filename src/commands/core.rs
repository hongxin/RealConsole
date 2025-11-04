//! 核心命令实现
//!
//! 提供基础命令：
//! - /help - 显示帮助信息
//! - /quit - 退出程序
//! - /version - 显示版本信息

use crate::command::{Command, CommandRegistry};
use crate::i18n;
use colored::Colorize;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 注册核心命令
pub fn register_core_commands(registry: &mut CommandRegistry) {
    // /help 命令
    let help_cmd = Command::from_fn("help", &i18n::t("cli.cmd.help.desc"), cmd_help)
        .with_aliases(vec!["h".to_string(), "?".to_string()])
        .with_group("core");
    registry.register(help_cmd);

    // /quit 命令
    let quit_cmd = Command::from_fn("quit", &i18n::t("cli.cmd.quit.desc"), cmd_quit)
        .with_aliases(vec!["q".to_string(), "exit".to_string()])
        .with_group("core");
    registry.register(quit_cmd);

    // /version 命令
    let version_cmd = Command::from_fn("version", &i18n::t("cli.cmd.version.desc"), cmd_version)
        .with_aliases(vec!["v".to_string()])
        .with_group("core");
    registry.register(version_cmd);

    // /commands 命令（列出所有命令）
    let commands_cmd =
        Command::from_fn("commands", &i18n::t("cli.cmd.commands.desc"), cmd_commands).with_group("core");
    registry.register(commands_cmd);

    // /examples 命令（使用示例）
    let examples_cmd = Command::from_fn("examples", &i18n::t("cli.cmd.examples.desc"), cmd_examples)
        .with_aliases(vec!["ex".to_string()])
        .with_group("core");
    registry.register(examples_cmd);

    // /quickref 命令（快速参考）
    let quickref_cmd = Command::from_fn("quickref", &i18n::t("cli.cmd.quickref.desc"), cmd_quickref)
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
            "{}\n{}",
            i18n::t_with_args("cli.help.unknown_topic", &[("topic", arg)]).yellow(),
            i18n::t_with_args("cli.help.unknown_hint", &[("cmd", "/help")]).cyan()
        ),
    }
}

/// 快速帮助（简洁版）
fn cmd_help_quick() -> String {
    let version_str = i18n::t_with_args("cli.help.quick.version", &[("version", VERSION)]);
    let header = format!(
        " {} {}",
        i18n::t("cli.help.quick.header").bold().cyan(),
        version_str.dimmed()
    );

    format!(
        r#"{}

{}
  {}
  {} {}
  {} {}

{}
  {}
  {}         {}
  {}         {}
  {}   {}
  {}        {}

{}
  {}      {}
  {}  {}
  {}   {}
  {}   {}
  {}      {}

{}
  {}        {}
  {}   {}

{}
  {}    {}
  {}        {}

{}
  {}
  {}
"#,
        header,
        i18n::t("cli.help.quick.chat_title").bold(),
        i18n::t("cli.help.quick.chat_desc"),
        i18n::t("cli.help.quick.chat_example1").dimmed(),
        i18n::t("cli.help.quick.chat_example1_text"),
        i18n::t("cli.help.quick.chat_example2").dimmed(),
        i18n::t("cli.help.quick.chat_example2_text"),
        i18n::t("cli.help.quick.routing_title").bold(),
        i18n::t("cli.help.quick.routing_desc"),
        i18n::t("cli.help.quick.routing_ls").green(),
        i18n::t("cli.help.quick.routing_ls_desc"),
        i18n::t("cli.help.quick.routing_pwd").green(),
        i18n::t("cli.help.quick.routing_pwd_desc"),
        i18n::t("cli.help.quick.routing_git").green(),
        i18n::t("cli.help.quick.routing_git_desc"),
        i18n::t("cli.help.quick.routing_shell").green(),
        i18n::t("cli.help.quick.routing_shell_desc"),
        i18n::t("cli.help.quick.commands_title").bold(),
        i18n::t("cli.help.quick.cmd_help").green(),
        i18n::t("cli.help.quick.cmd_help_desc"),
        i18n::t("cli.help.quick.cmd_help_all").green(),
        i18n::t("cli.help.quick.cmd_help_all_desc"),
        i18n::t("cli.help.quick.cmd_examples").green(),
        i18n::t("cli.help.quick.cmd_examples_desc"),
        i18n::t("cli.help.quick.cmd_quickref").green(),
        i18n::t("cli.help.quick.cmd_quickref_desc"),
        i18n::t("cli.help.quick.cmd_quit").green(),
        i18n::t("cli.help.quick.cmd_quit_desc"),
        i18n::t("cli.help.quick.tools_title").bold(),
        i18n::t("cli.help.quick.tools_list").green(),
        i18n::t("cli.help.quick.tools_list_desc"),
        i18n::t("cli.help.quick.tools_call").green(),
        i18n::t("cli.help.quick.tools_call_desc"),
        i18n::t("cli.help.quick.memory_title").bold(),
        i18n::t("cli.help.quick.memory_recent").green(),
        i18n::t("cli.help.quick.memory_recent_desc"),
        i18n::t("cli.help.quick.log_stats").green(),
        i18n::t("cli.help.quick.log_stats_desc"),
        i18n::t("cli.help.quick.tips_title").bold(),
        i18n::t_with_args("cli.help.quick.tips_detail", &[("cmd", "/help <命令>")]).cyan(),
        i18n::t_with_args("cli.help.quick.tips_routing", &[("cmd", "/help shell")]).cyan()
    )
}

/// 详细帮助（完整文档）
fn cmd_help_all() -> String {
    let title = format!("{}", i18n::t("cli.help.all.title").bold().cyan());

    format!(
        r#"{}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}

{}
"#,
        title,
        i18n::t("cli.help.all.core_title").bold(),
        i18n::t("cli.help.all.core_help").green(),
        i18n::t("cli.help.all.core_quit").green(),
        i18n::t("cli.help.all.core_version").green(),
        i18n::t("cli.help.all.core_commands").green(),
        i18n::t("cli.help.all.llm_title").bold(),
        i18n::t("cli.help.all.llm_status").green(),
        i18n::t("cli.help.all.llm_ask").green(),
        i18n::t("cli.help.all.tools_title").bold(),
        i18n::t("cli.help.all.tools_list").green(),
        i18n::t("cli.help.all.tools_info").green(),
        i18n::t("cli.help.all.tools_call").green(),
        i18n::t("cli.help.all.tools_example"),
        i18n::t("cli.help.all.memory_title").bold(),
        i18n::t("cli.help.all.memory_recent").green(),
        i18n::t("cli.help.all.memory_search").green(),
        i18n::t("cli.help.all.memory_clear").green(),
        i18n::t("cli.help.all.memory_save").green(),
        i18n::t("cli.help.all.log_title").bold(),
        i18n::t("cli.help.all.log_recent").green(),
        i18n::t("cli.help.all.log_search").green(),
        i18n::t("cli.help.all.log_stats").green(),
        i18n::t("cli.help.all.log_failed").green(),
        i18n::t("cli.help.all.shell_title").bold(),
        i18n::t("cli.help.all.shell_prefix").yellow(),
        i18n::t("cli.help.all.shell_safety"),
        i18n::t("cli.help.all.shell_example"),
        i18n::t_with_args("cli.help.all.more", &[
            ("examples", "/examples"),
            ("help_tools", "/help tools"),
            ("quickref", "/quickref")
        ]).cyan()
    )
}

/// 工具命令帮助
fn cmd_help_tools() -> String {
    format!(
        r#"{}

{}
  {}
  {}
  {}
  {}

{}
  {}
    {}
    {}
    {}
    {}
    {}

  {}
    {}
    {}
    {}
    {}
    {}
    {}
    {}
    {}
    {}

{}
  {}
  {}

  {}
  {}

  {}
  {}

{}
  {}
  {}
  {}
"#,
        i18n::t("cli.help.tools.title").bold(),
        i18n::t("cli.help.tools.usage_title"),
        i18n::t("cli.help.tools.usage_list1").green(),
        i18n::t("cli.help.tools.usage_list2").green(),
        i18n::t("cli.help.tools.usage_info").green(),
        i18n::t("cli.help.tools.usage_call").green(),
        i18n::t("cli.help.tools.available"),
        i18n::t("cli.help.tools.basic_title"),
        i18n::t("cli.help.tools.basic_calculator"),
        i18n::t("cli.help.tools.basic_datetime"),
        i18n::t("cli.help.tools.basic_uuid"),
        i18n::t("cli.help.tools.basic_base64"),
        i18n::t("cli.help.tools.basic_random"),
        i18n::t("cli.help.tools.advanced_title"),
        i18n::t("cli.help.tools.advanced_http_get"),
        i18n::t("cli.help.tools.advanced_http_post"),
        i18n::t("cli.help.tools.advanced_json_parse"),
        i18n::t("cli.help.tools.advanced_json_query"),
        i18n::t("cli.help.tools.advanced_text_search"),
        i18n::t("cli.help.tools.advanced_text_replace"),
        i18n::t("cli.help.tools.advanced_file_read"),
        i18n::t("cli.help.tools.advanced_file_write"),
        i18n::t("cli.help.tools.advanced_sys_info"),
        i18n::t("cli.help.tools.examples_title"),
        i18n::t("cli.help.tools.example1_comment"),
        i18n::t("cli.help.tools.example1_cmd"),
        i18n::t("cli.help.tools.example2_comment"),
        i18n::t("cli.help.tools.example2_cmd"),
        i18n::t("cli.help.tools.example3_comment"),
        i18n::t("cli.help.tools.example3_cmd"),
        i18n::t("cli.help.tools.tips_title"),
        i18n::t("cli.help.tools.tip1"),
        i18n::t("cli.help.tools.tip2"),
        i18n::t("cli.help.tools.tip3")
    )
}

/// 记忆命令帮助
fn cmd_help_memory() -> String {
    format!(
        r#"{}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}

{}
  {}
  {}
  {}
"#,
        i18n::t("cli.help.memory.title").bold(),
        i18n::t("cli.help.memory.usage_title"),
        i18n::t("cli.help.memory.usage_recent").green(),
        i18n::t("cli.help.memory.usage_search").green(),
        i18n::t("cli.help.memory.usage_clear").green(),
        i18n::t("cli.help.memory.usage_save").green(),
        i18n::t("cli.help.memory.examples_title"),
        i18n::t("cli.help.memory.example1"),
        i18n::t("cli.help.memory.example2"),
        i18n::t("cli.help.memory.example3"),
        i18n::t("cli.help.memory.tips_title"),
        i18n::t("cli.help.memory.tip1"),
        i18n::t("cli.help.memory.tip2"),
        i18n::t("cli.help.memory.tip3")
    )
}

/// 日志命令帮助
fn cmd_help_log() -> String {
    format!(
        r#"{}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}
"#,
        i18n::t("cli.help.log.title").bold(),
        i18n::t("cli.help.log.usage_title"),
        i18n::t("cli.help.log.usage_recent").green(),
        i18n::t("cli.help.log.usage_search").green(),
        i18n::t("cli.help.log.usage_stats").green(),
        i18n::t("cli.help.log.usage_failed").green(),
        i18n::t("cli.help.log.examples_title"),
        i18n::t("cli.help.log.example1"),
        i18n::t("cli.help.log.example2"),
        i18n::t("cli.help.log.example3"),
        i18n::t("cli.help.log.example4"),
        i18n::t("cli.help.log.tips_title"),
        i18n::t("cli.help.log.tip1"),
        i18n::t("cli.help.log.tip2"),
        i18n::t("cli.help.log.tip3")
    )
}

/// Shell 命令帮助
fn cmd_help_shell() -> String {
    format!(
        r#"{}

{}
  {}

  {}
    {}                  {}
    {}                 {}
    {}         {}
    {}         {}
    {}       {}

  {}
    {}             {}

  {}
    {}             {}

  {}
    {}           {}
    {}       {}

{}
  {}
  {}
  {}
  {}

{}
  {}
    {}
    {}
    {}
    {}
    {}
    {}

{}
  {}
  {}
  {}

{}
  {}
  {}
  {}
"#,
        i18n::t("cli.help.shell.title").bold(),
        i18n::t("cli.help.shell.routing_title"),
        i18n::t("cli.help.shell.routing_intro"),
        i18n::t("cli.help.shell.direct_commands"),
        i18n::t("cli.help.shell.direct_ls").green(),
        i18n::t("cli.help.shell.direct_ls_desc"),
        i18n::t("cli.help.shell.direct_pwd").green(),
        i18n::t("cli.help.shell.direct_pwd_desc"),
        i18n::t("cli.help.shell.direct_git").green(),
        i18n::t("cli.help.shell.direct_git_desc"),
        i18n::t("cli.help.shell.direct_docker").green(),
        i18n::t("cli.help.shell.direct_docker_desc"),
        i18n::t("cli.help.shell.direct_cargo").green(),
        i18n::t("cli.help.shell.direct_cargo_desc"),
        i18n::t("cli.help.shell.force_shell"),
        i18n::t("cli.help.shell.force_example").yellow(),
        i18n::t("cli.help.shell.force_desc"),
        i18n::t("cli.help.shell.system_cmd"),
        i18n::t("cli.help.shell.system_example").cyan(),
        i18n::t("cli.help.shell.system_desc"),
        i18n::t("cli.help.shell.natural_lang"),
        i18n::t("cli.help.shell.natural_example1").dimmed(),
        i18n::t("cli.help.shell.natural_desc1"),
        i18n::t("cli.help.shell.natural_example2").dimmed(),
        i18n::t("cli.help.shell.natural_desc2"),
        i18n::t("cli.help.shell.priority_title"),
        i18n::t("cli.help.shell.priority1"),
        i18n::t("cli.help.shell.priority2"),
        i18n::t("cli.help.shell.priority3"),
        i18n::t("cli.help.shell.priority4"),
        i18n::t("cli.help.shell.safety_title"),
        i18n::t("cli.help.shell.safety_intro"),
        i18n::t("cli.help.shell.safety_rm"),
        i18n::t("cli.help.shell.safety_sudo"),
        i18n::t("cli.help.shell.safety_shutdown"),
        i18n::t("cli.help.shell.safety_mkfs"),
        i18n::t("cli.help.shell.safety_dd"),
        i18n::t("cli.help.shell.safety_dev"),
        i18n::t("cli.help.shell.limits_title"),
        i18n::t("cli.help.shell.limits_timeout"),
        i18n::t("cli.help.shell.limits_output"),
        i18n::t("cli.help.shell.limits_platform"),
        i18n::t("cli.help.shell.tips_title"),
        i18n::t("cli.help.shell.tip1"),
        i18n::t("cli.help.shell.tip2"),
        i18n::t("cli.help.shell.tip3")
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
        i18n::t("cli.version.app_name").bold(),
        VERSION.cyan(),
        i18n::t("cli.version.tagline").dimmed(),
        i18n::t("cli.version.phase1").green(),
        i18n::t("cli.version.phase2").green(),
        i18n::t("cli.version.phase3").green(),
        i18n::t("cli.version.phase4").green(),
        i18n::t("cli.version.phase5").green(),
        i18n::t("cli.version.tests").dimmed(),
        i18n::t("cli.version.features").bold(),
        i18n::t("cli.version.feature_tools").yellow(),
        i18n::t("cli.version.feature_intent").yellow(),
        i18n::t("cli.version.feature_memory").yellow()
    )
}

/// /commands 命令处理器
fn cmd_commands(_arg: &str) -> String {
    // 这个命令需要访问 registry，暂时返回占位符
    // 实际实现需要在运行时注入 registry 引用
    i18n::t_with_args("cli.commands.hint", &[
        ("help", "/help"),
        ("help_all", "/help all")
    ])
}

/// /examples 命令处理器
fn cmd_examples(_arg: &str) -> String {
    format!(
        r#"{}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}

{}
  {}
  {}
  {}
  {}

{}
  {}
  {}
  {}
"#,
        i18n::t("cli.examples.title").bold(),
        i18n::t("cli.examples.chat_title").bold(),
        i18n::t("cli.examples.chat1"),
        i18n::t("cli.examples.chat2"),
        i18n::t("cli.examples.chat3"),
        i18n::t("cli.examples.chat4"),
        i18n::t("cli.examples.routing_title").bold(),
        i18n::t("cli.examples.routing1"),
        i18n::t("cli.examples.routing2"),
        i18n::t("cli.examples.routing3"),
        i18n::t("cli.examples.routing4"),
        i18n::t("cli.examples.routing5"),
        i18n::t("cli.examples.routing6"),
        i18n::t("cli.examples.tools_title").bold(),
        i18n::t("cli.examples.tools1"),
        i18n::t("cli.examples.tools2"),
        i18n::t("cli.examples.tools3"),
        i18n::t("cli.examples.tools4"),
        i18n::t("cli.examples.tools5"),
        i18n::t("cli.examples.memory_title").bold(),
        i18n::t("cli.examples.memory1"),
        i18n::t("cli.examples.memory2"),
        i18n::t("cli.examples.memory3"),
        i18n::t("cli.examples.log_title").bold(),
        i18n::t("cli.examples.log1"),
        i18n::t("cli.examples.log2"),
        i18n::t("cli.examples.log3"),
        i18n::t("cli.examples.log4"),
        i18n::t("cli.examples.tips_title").bold().dimmed(),
        i18n::t("cli.examples.tip1"),
        i18n::t_with_args("cli.examples.tip2", &[("help_cmd", "/help <命令>")]).cyan(),
        i18n::t_with_args("cli.examples.tip3", &[("help_shell", "/help shell")]).cyan()
    )
}

/// /quickref 命令处理器
fn cmd_quickref(_arg: &str) -> String {
    format!(
        r#"{}

{}
  {}                    {}  {}
  {}                 {}  {}
  {}                        {}  {}

{}
  {}       {}
  {}      {}
  {}     {}
  {}        {}
  {}       {}

{}
  {}      {}
  {}      {}
  {}        {}

{}: {} | {}
"#,
        i18n::t("cli.quickref.title").bold().cyan(),
        i18n::t("cli.quickref.basic_usage").bold(),
        i18n::t("cli.quickref.usage_chat"),
        i18n::t("cli.quickref.usage_chat_example").dimmed(),
        i18n::t("cli.quickref.usage_chat_text"),
        i18n::t("cli.quickref.usage_shell"),
        i18n::t("cli.quickref.usage_shell_example").dimmed(),
        i18n::t("cli.quickref.usage_shell_text"),
        i18n::t("cli.quickref.usage_system"),
        i18n::t("cli.quickref.usage_system_example").dimmed(),
        i18n::t("cli.quickref.usage_system_text"),
        i18n::t("cli.quickref.common_cmds").bold(),
        i18n::t("cli.quickref.cmd_help").green(),
        i18n::t("cli.quickref.cmd_help_desc"),
        i18n::t("cli.quickref.cmd_tools").green(),
        i18n::t("cli.quickref.cmd_tools_desc"),
        i18n::t("cli.quickref.cmd_memory").green(),
        i18n::t("cli.quickref.cmd_memory_desc"),
        i18n::t("cli.quickref.cmd_log").green(),
        i18n::t("cli.quickref.cmd_log_desc"),
        i18n::t("cli.quickref.cmd_quit").green(),
        i18n::t("cli.quickref.cmd_quit_desc"),
        i18n::t("cli.quickref.shortcuts").bold(),
        i18n::t("cli.quickref.shortcut_cancel").yellow(),
        i18n::t("cli.quickref.shortcut_cancel_desc"),
        i18n::t("cli.quickref.shortcut_exit").yellow(),
        i18n::t("cli.quickref.shortcut_exit_desc"),
        i18n::t("cli.quickref.shortcut_history").yellow(),
        i18n::t("cli.quickref.shortcut_history_desc"),
        i18n::t("cli.quickref.more").bold(),
        i18n::t("cli.quickref.more_help_all").cyan(),
        i18n::t("cli.quickref.more_examples").cyan()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 初始化 i18n 用于测试
    /// 使用 std::panic::catch_unwind 捕获可能的 panic
    fn init_i18n() {
        use crate::i18n::Language;
        use std::panic;

        // 尝试初始化，如果失败则忽略（测试环境可能找不到 locale 文件）
        let _ = panic::catch_unwind(|| {
            crate::i18n::init(Language::ZhCn);
        });
    }

    #[test]
    fn test_help_command() {
        init_i18n();
        let output = cmd_help("");
        assert!(output.contains("RealConsole"));
        assert!(output.contains("/help"));
        assert!(output.contains("智能对话"));
    }

    #[test]
    fn test_help_all() {
        init_i18n();
        let output = cmd_help("all");
        assert!(output.contains("完整命令参考"));
        assert!(output.contains("/tools"));
        assert!(output.contains("/memory"));
    }

    #[test]
    fn test_help_tools() {
        init_i18n();
        let output = cmd_help("tools");
        assert!(output.contains("工具管理命令"));
        assert!(output.contains("calculator"));
    }

    #[test]
    fn test_examples_command() {
        init_i18n();
        let output = cmd_examples("");
        assert!(output.contains("使用示例"));
        assert!(output.contains("智能对话"));
    }

    #[test]
    fn test_quickref_command() {
        init_i18n();
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
        init_i18n();
        let output = cmd_version("");
        assert!(output.contains("RealConsole"));
        assert!(output.contains(VERSION));
    }

    #[test]
    fn test_register_core_commands() {
        init_i18n();
        let mut registry = CommandRegistry::new();
        register_core_commands(&mut registry);

        assert!(registry.get("help").is_some());
        assert!(registry.get("quit").is_some());
        assert!(registry.get("version").is_some());
        assert!(registry.get("h").is_some()); // 别名测试
    }
}
