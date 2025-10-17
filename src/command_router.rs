//! 命令智能路由器
//!
//! 负责智能识别用户输入类型，决定路由到：
//! 1. Shell命令直接执行（常见命令）
//! 2. 系统命令执行（/前缀）
//! 3. LLM智能处理（自然语言）
//!
//! 设计理念：道法自然
//! - 用户习惯无感过渡
//! - 常见命令零延迟
//! - 智能功能逐步引导

use std::collections::HashSet;
use once_cell::sync::Lazy;

/// 常见Shell命令列表
///
/// 包含用户最常用的命令，这些命令会被直接识别并执行
static COMMON_SHELL_COMMANDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        // 文件导航
        "ls", "ll", "cd", "pwd", "tree",

        // 文件操作
        "cat", "less", "more", "head", "tail", "touch", "mkdir", "rm", "rmdir",
        "cp", "mv", "ln", "chmod", "chown",

        // 文件搜索
        "find", "locate", "which", "whereis",

        // 文本处理
        "grep", "egrep", "fgrep", "sed", "awk", "cut", "sort", "uniq", "wc",

        // 进程管理
        "ps", "top", "htop", "kill", "killall", "pkill", "pgrep",

        // 网络工具
        "ping", "curl", "wget", "netstat", "ss", "ip", "ifconfig",

        // 系统信息
        "uname", "hostname", "whoami", "who", "w", "uptime", "free", "df", "du",

        // 压缩解压
        "tar", "gzip", "gunzip", "zip", "unzip", "bzip2", "bunzip2",

        // Git命令
        "git", "gitk",

        // 编辑器
        "vi", "vim", "nano", "emacs",

        // 其他常用
        "echo", "date", "cal", "bc", "man", "info", "history", "clear", "exit",

        // 开发工具
        "make", "cmake", "gcc", "g++", "clang", "rustc", "cargo", "npm", "yarn",
        "python", "python3", "node", "java", "javac", "ruby", "perl", "go",

        // Docker & 容器
        "docker", "docker-compose", "kubectl", "podman",

        // 数据库
        "mysql", "psql", "sqlite3", "redis-cli", "mongo",
    ]
    .iter()
    .copied()
    .collect()
});

/// 命令类型识别结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandType {
    /// Shell命令（常见命令，直接执行）
    CommonShell(String),

    /// Shell命令（强制执行，使用!前缀）
    ForcedShell(String),

    /// 系统命令（使用/前缀）
    SystemCommand(String, String),  // (命令名, 参数)

    /// 自然语言（需要LLM处理）
    NaturalLanguage(String),
}

/// 命令路由器
pub struct CommandRouter {
    /// 系统命令前缀（默认 "/"）
    system_prefix: String,

    /// 是否启用智能路由
    smart_routing_enabled: bool,
}

impl CommandRouter {
    /// 创建命令路由器
    pub fn new(system_prefix: String) -> Self {
        Self {
            system_prefix,
            smart_routing_enabled: true,
        }
    }

    /// 禁用智能路由（回退到传统模式）
    pub fn disable_smart_routing(mut self) -> Self {
        self.smart_routing_enabled = false;
        self
    }

    /// 路由用户输入到对应的处理器
    ///
    /// 优先级：
    /// 1. 强制Shell (!前缀) - 最高优先级
    /// 2. 系统命令 (/前缀) - 次高优先级
    /// 3. 常见Shell命令 - 智能识别
    /// 4. 自然语言 - 兜底处理
    pub fn route(&self, input: &str) -> CommandType {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return CommandType::NaturalLanguage(String::new());
        }

        // 1. 检查强制Shell前缀 (!)
        if let Some(cmd) = trimmed.strip_prefix('!') {
            return CommandType::ForcedShell(cmd.to_string());
        }

        // 2. 检查系统命令前缀 (/)
        if let Some(input) = trimmed.strip_prefix(&self.system_prefix) {
            let parts: Vec<&str> = input.splitn(2, ' ').collect();
            let cmd_name = parts[0].to_string();
            let arg = parts.get(1).copied().unwrap_or("").to_string();
            return CommandType::SystemCommand(cmd_name, arg);
        }

        // 3. 智能识别常见Shell命令（如果启用）
        if self.smart_routing_enabled {
            if let Some(cmd_type) = self.detect_common_shell(trimmed) {
                return cmd_type;
            }
        }

        // 4. 默认为自然语言
        CommandType::NaturalLanguage(trimmed.to_string())
    }

    /// 检测是否为常见Shell命令
    ///
    /// 检测规则：
    /// 1. 提取第一个单词（命令名）
    /// 2. 检查是否在常见命令列表中
    /// 3. 排除明显的自然语言（包含"我"、"你"、"吗"等）
    fn detect_common_shell(&self, input: &str) -> Option<CommandType> {
        // 提取第一个单词
        let first_word = input.split_whitespace().next()?;

        // 检查是否在常见命令列表中
        if COMMON_SHELL_COMMANDS.contains(first_word) {
            // 额外检查：排除明显的自然语言
            if self.looks_like_natural_language(input) {
                return None;
            }

            return Some(CommandType::CommonShell(input.to_string()));
        }

        None
    }

    /// 判断输入是否看起来像自然语言
    ///
    /// 启发式规则：
    /// - 包含中文疑问词：吗、呢、吧、嘛
    /// - 包含中文代词：我、你、他、她、它、我们、你们
    /// - 包含长句子（超过5个单词且有中文）
    fn looks_like_natural_language(&self, input: &str) -> bool {
        // 检查中文疑问词
        if input.contains('吗') || input.contains('呢')
            || input.contains('吧') || input.contains('嘛') {
            return true;
        }

        // 检查中文代词
        let chinese_pronouns = ["我", "你", "他", "她", "它", "我们", "你们", "他们"];
        if chinese_pronouns.iter().any(|p| input.contains(p)) {
            return true;
        }

        // 检查是否包含中文且单词数量较多（可能是问句）
        let has_chinese = input.chars().any(|c| {
            matches!(c, '\u{4e00}'..='\u{9fff}')
        });

        if has_chinese {
            let word_count = input.split_whitespace().count();
            if word_count > 5 {
                return true;
            }
        }

        false
    }

    /// 获取使用提示
    pub fn usage_hint(&self) -> String {
        let prefix = &self.system_prefix;
        format!(
            r#"💡 RealConsole 使用提示：

1. 直接输入常见命令（零延迟）：
   {}ls{}                  - 列出文件
   {}pwd{}                 - 显示当前目录
   {}git status{}          - 查看Git状态

2. 系统命令（{}前缀）：
   {}{}help{}             - 查看帮助
   {}{}history{}          - 查看历史记录
   {}{}plan <目标>{}      - 智能任务规划

3. 强制Shell执行（!前缀）：
   {}!ls -la{}             - 强制以Shell方式执行

4. 自然语言对话（直接输入）：
   {}帮我分析这个错误日志{}
   {}如何优化这段代码{}

智能提示：系统会自动识别命令类型，无需手动添加前缀。
"#,
            "\x1b[32m",  // green
            "\x1b[0m",   // reset
            "\x1b[32m",
            "\x1b[0m",
            "\x1b[32m",
            "\x1b[0m",
            prefix,
            "\x1b[36m",  // cyan
            prefix,
            "\x1b[0m",
            "\x1b[36m",
            prefix,
            "\x1b[0m",
            "\x1b[36m",
            prefix,
            "\x1b[0m",
            "\x1b[33m",  // yellow
            "\x1b[0m",
            "\x1b[35m",  // magenta
            "\x1b[0m",
            "\x1b[35m",
            "\x1b[0m",
        )
    }
}

impl Default for CommandRouter {
    fn default() -> Self {
        Self::new("/".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_forced_shell() {
        let router = CommandRouter::default();

        let result = router.route("!ls -la");
        assert_eq!(result, CommandType::ForcedShell("ls -la".to_string()));

        let result = router.route("!pwd");
        assert_eq!(result, CommandType::ForcedShell("pwd".to_string()));
    }

    #[test]
    fn test_route_system_command() {
        let router = CommandRouter::default();

        let result = router.route("/help");
        assert_eq!(result, CommandType::SystemCommand("help".to_string(), "".to_string()));

        let result = router.route("/plan 创建项目");
        assert_eq!(result, CommandType::SystemCommand("plan".to_string(), "创建项目".to_string()));
    }

    #[test]
    fn test_route_common_shell() {
        let router = CommandRouter::default();

        let result = router.route("ls");
        assert_eq!(result, CommandType::CommonShell("ls".to_string()));

        let result = router.route("ls -la");
        assert_eq!(result, CommandType::CommonShell("ls -la".to_string()));

        let result = router.route("pwd");
        assert_eq!(result, CommandType::CommonShell("pwd".to_string()));

        let result = router.route("git status");
        assert_eq!(result, CommandType::CommonShell("git status".to_string()));
    }

    #[test]
    fn test_route_natural_language() {
        let router = CommandRouter::default();

        let result = router.route("你好");
        assert!(matches!(result, CommandType::NaturalLanguage(_)));

        let result = router.route("帮我分析这个错误");
        assert!(matches!(result, CommandType::NaturalLanguage(_)));

        let result = router.route("what is the weather");
        assert!(matches!(result, CommandType::NaturalLanguage(_)));
    }

    #[test]
    fn test_looks_like_natural_language() {
        let router = CommandRouter::default();

        // 应该识别为自然语言
        assert!(router.looks_like_natural_language("ls这个命令是什么吗？"));
        assert!(router.looks_like_natural_language("你能帮我执行ls命令吗"));
        assert!(router.looks_like_natural_language("我想知道当前目录"));

        // 不应该识别为自然语言
        assert!(!router.looks_like_natural_language("ls -la"));
        assert!(!router.looks_like_natural_language("pwd"));
        assert!(!router.looks_like_natural_language("git status"));
    }

    #[test]
    fn test_common_shell_commands_coverage() {
        let router = CommandRouter::default();

        // 测试常见命令列表
        let common_cmds = vec![
            "ls", "cd", "pwd", "cat", "grep", "find", "ps", "top",
            "git", "docker", "npm", "cargo", "python", "make",
        ];

        for cmd in common_cmds {
            let result = router.route(cmd);
            assert!(
                matches!(result, CommandType::CommonShell(_)),
                "Command '{}' should be recognized as common shell command",
                cmd
            );
        }
    }

    #[test]
    fn test_empty_input() {
        let router = CommandRouter::default();

        let result = router.route("");
        assert_eq!(result, CommandType::NaturalLanguage("".to_string()));

        let result = router.route("   ");
        assert_eq!(result, CommandType::NaturalLanguage("".to_string()));
    }

    #[test]
    fn test_disable_smart_routing() {
        let router = CommandRouter::default().disable_smart_routing();

        // 禁用智能路由后，普通命令应该被视为自然语言
        let result = router.route("ls");
        assert!(matches!(result, CommandType::NaturalLanguage(_)));

        // 但强制Shell和系统命令仍然有效
        let result = router.route("!ls");
        assert!(matches!(result, CommandType::ForcedShell(_)));

        let result = router.route("/help");
        assert!(matches!(result, CommandType::SystemCommand(_, _)));
    }

    #[test]
    fn test_priority_order() {
        let router = CommandRouter::default();

        // 强制Shell优先级最高
        let result = router.route("!ls");  // 不是 "ls"
        assert!(matches!(result, CommandType::ForcedShell(_)));

        // 系统命令次优先级
        let result = router.route("/ls");  // 即使ls是常见命令
        assert!(matches!(result, CommandType::SystemCommand(_, _)));
    }

    #[test]
    fn test_edge_cases() {
        let router = CommandRouter::default();

        // 命令 + 中文参数（应该仍然识别为Shell命令）
        let result = router.route("echo 你好");
        assert_eq!(result, CommandType::CommonShell("echo 你好".to_string()));

        // 中文 + 命令（应该识别为自然语言）
        let result = router.route("请帮我运行 ls 命令");
        assert!(matches!(result, CommandType::NaturalLanguage(_)));
    }
}
