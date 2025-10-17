//! 命令注册与分发系统
//!
//! 设计参考 Python 版本的 CommandRegistry，提供：
//! - 命令注册
//! - 别名支持
//! - 命令分组
//! - 命令查找与分发

use std::collections::HashMap;

use std::sync::Arc;

/// 命令处理函数签名（支持闭包）
pub type CommandHandler = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// 命令定义
#[derive(Clone)]
pub struct Command {
    pub name: String,
    #[allow(dead_code)]  // Phase 2 将用于增强的帮助信息
    pub desc: String,
    pub handler: CommandHandler,
    pub aliases: Vec<String>,
    pub group: Option<String>,
}

impl Command {
    pub fn new(name: impl Into<String>, desc: impl Into<String>, handler: CommandHandler) -> Self {
        Self {
            name: name.into(),
            desc: desc.into(),
            handler,
            aliases: Vec::new(),
            group: None,
        }
    }

    /// 从函数创建命令
    pub fn from_fn<F>(name: impl Into<String>, desc: impl Into<String>, handler: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        Self::new(name, desc, Arc::new(handler))
    }

    pub fn with_aliases(mut self, aliases: Vec<String>) -> Self {
        self.aliases = aliases;
        self
    }

    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

/// 命令注册表
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
    alias_map: HashMap<String, String>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            alias_map: HashMap::new(),
        }
    }

    /// 注册命令
    pub fn register(&mut self, command: Command) {
        let name = command.name.clone();

        // 注册别名映射
        for alias in &command.aliases {
            self.alias_map.insert(alias.clone(), name.clone());
        }

        self.commands.insert(name, command);
    }

    /// 获取命令（支持别名）
    pub fn get(&self, name: &str) -> Option<&Command> {
        // 先尝试别名映射
        let real_name = self.alias_map.get(name).map(|s| s.as_str()).unwrap_or(name);
        self.commands.get(real_name)
    }

    /// 列出所有命令（按名称排序）
    #[allow(dead_code)]  // Phase 2 将用于 /commands 命令
    pub fn list(&self) -> Vec<&Command> {
        let mut cmds: Vec<&Command> = self.commands.values().collect();
        cmds.sort_by(|a, b| a.name.cmp(&b.name));
        cmds
    }

    /// 按分组列出命令
    #[allow(dead_code)]  // Phase 2 将用于分组显示帮助
    pub fn list_by_group(&self) -> HashMap<String, Vec<&Command>> {
        let mut grouped: HashMap<String, Vec<&Command>> = HashMap::new();

        for cmd in self.commands.values() {
            let group = cmd.group.as_deref().unwrap_or("core");
            grouped.entry(group.to_string())
                .or_default()
                .push(cmd);
        }

        // 对每组内的命令排序
        for cmds in grouped.values_mut() {
            cmds.sort_by(|a, b| a.name.cmp(&b.name));
        }

        grouped
    }

    /// 执行命令
    pub fn execute(&self, name: &str, arg: &str) -> Result<String, String> {
        match self.get(name) {
            Some(cmd) => Ok((cmd.handler)(arg)),
            None => Err(format!("未知命令: {} (用 /help 查看)", name)),
        }
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_handler(_arg: &str) -> String {
        "test output".to_string()
    }

    #[test]
    fn test_command_registration() {
        let mut registry = CommandRegistry::new();
        let cmd = Command::from_fn("test", "Test command", test_handler);
        registry.register(cmd);

        assert!(registry.get("test").is_some());
        assert_eq!(registry.get("test").unwrap().name, "test");
    }

    #[test]
    fn test_command_aliases() {
        let mut registry = CommandRegistry::new();
        let cmd = Command::from_fn("help", "Help command", test_handler)
            .with_aliases(vec!["h".to_string(), "?".to_string()]);
        registry.register(cmd);

        assert!(registry.get("help").is_some());
        assert!(registry.get("h").is_some());
        assert!(registry.get("?").is_some());
    }

    #[test]
    fn test_command_execution() {
        let mut registry = CommandRegistry::new();
        let cmd = Command::from_fn("test", "Test command", test_handler);
        registry.register(cmd);

        let result = registry.execute("test", "");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test output");
    }
}
