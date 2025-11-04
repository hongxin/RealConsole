//! 国际化 (i18n) 支持模块
//!
//! 遵循"一分为三"哲学设计的多语言系统：
//! - 明确态：已知语言（zh-CN 中文、en-US 英文）
//! - 演化态：可扩展架构，便于添加新语言
//! - 容错态：多级回退机制（命令行 > 配置 > 环境变量 > 系统语言 > 默认中文）
//!
//! 语言文件搜索策略：
//! - 当前目录 locales/
//! - 用户配置目录 ~/.realconsole/locales/

use crate::path_resolver::PathResolver;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;

/// 全局 i18n 实例
static I18N: OnceCell<RwLock<I18n>> = OnceCell::new();

/// 支持的语言
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum Language {
    /// 简体中文（中国）
    #[serde(rename = "zh-CN", alias = "zh", alias = "chinese", alias = "中文")]
    #[default]
    ZhCn,
    /// 美式英语（美国）
    #[serde(rename = "en-US", alias = "en", alias = "english", alias = "英文")]
    EnUs,
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zh-cn" | "zh" | "chinese" | "中文" | "简体中文" => Ok(Language::ZhCn),
            "en-us" | "en" | "english" | "英文" => Ok(Language::EnUs),
            _ => Err(format!("Unknown language: {}", s)),
        }
    }
}

impl Language {
    /// 转换为标准代码
    pub fn code(&self) -> &'static str {
        match self {
            Language::ZhCn => "zh-CN",
            Language::EnUs => "en-US",
        }
    }

    /// 获取语言的本地化名称
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::ZhCn => "简体中文",
            Language::EnUs => "English",
        }
    }

    /// 从系统环境推断语言
    pub fn from_system() -> Self {
        // 尝试从 LANG 环境变量获取
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("zh") {
                return Language::ZhCn;
            } else if lang.starts_with("en") {
                return Language::EnUs;
            }
        }

        // 默认使用中文（项目优先中文）
        Language::ZhCn
    }
}


impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// 国际化管理器
#[derive(Debug)]
pub struct I18n {
    /// 当前语言
    current_lang: Language,
    /// 翻译字典：语言 -> (键 -> 值)
    translations: HashMap<Language, HashMap<String, String>>,
    /// 回退语言（默认中文）
    fallback_lang: Language,
}

impl I18n {
    /// 创建新的 i18n 实例
    pub fn new(lang: Language) -> Self {
        let mut i18n = Self {
            current_lang: lang,
            translations: HashMap::new(),
            fallback_lang: Language::ZhCn,
        };

        // 尝试加载语言文件
        i18n.load_language(Language::ZhCn);
        i18n.load_language(Language::EnUs);

        i18n
    }

    /// 加载语言文件（支持多路径搜索）
    ///
    /// # 搜索策略
    /// 1. 当前目录 locales/{lang}.yaml (基础翻译 - Web)
    /// 2. 当前目录 locales/{lang}-cli.yaml (CLI 翻译)
    /// 3. 用户配置目录 ~/.realconsole/locales/{lang}.yaml
    /// 4. 用户配置目录 ~/.realconsole/locales/{lang}-cli.yaml
    /// 5. 如果都不存在，使用内置翻译（硬编码）
    ///
    /// # 文件合并策略
    /// - 先加载基础文件（Web 翻译）
    /// - 再加载 CLI 文件（CLI 翻译）
    /// - CLI 文件中的键会覆盖基础文件中的同名键
    /// - 如果 CLI 文件不存在，只使用基础文件（向后兼容）
    fn load_language(&mut self, lang: Language) {
        let base_filename = format!("locales/{}.yaml", lang.code());
        let cli_filename = format!("locales/{}-cli.yaml", lang.code());
        let prompts_filename = format!("locales/{}-prompts.yaml", lang.code());

        let mut translations = HashMap::new();
        let mut loaded_any = false;

        // 1. 加载基础文件（Web 翻译）
        if let Some(base_path) = PathResolver::resolve(&base_filename) {
            match fs::read_to_string(&base_path) {
                Ok(content) => match serde_yaml_ng::from_str::<HashMap<String, String>>(&content) {
                    Ok(base_trans) => {
                        translations.extend(base_trans);
                        loaded_any = true;
                    }
                    Err(e) => {
                        eprintln!("⚠ 解析基础语言文件 {} 失败: {}", base_path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("⚠ 读取基础语言文件 {} 失败: {}", base_path.display(), e);
                }
            }
        }

        // 2. 加载 CLI 文件（CLI 翻译）- 可选，用于向后兼容
        if let Some(cli_path) = PathResolver::resolve(&cli_filename) {
            match fs::read_to_string(&cli_path) {
                Ok(content) => match serde_yaml_ng::from_str::<HashMap<String, String>>(&content) {
                    Ok(cli_trans) => {
                        translations.extend(cli_trans);
                        loaded_any = true;
                    }
                    Err(e) => {
                        eprintln!("⚠ 解析 CLI 语言文件 {} 失败: {}", cli_path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("⚠ 读取 CLI 语言文件 {} 失败: {}", cli_path.display(), e);
                }
            }
        }

        // 3. 加载 Prompts 文件（LLM 提示词翻译）- 可选
        if let Some(prompts_path) = PathResolver::resolve(&prompts_filename) {
            match fs::read_to_string(&prompts_path) {
                Ok(content) => match serde_yaml_ng::from_str::<HashMap<String, String>>(&content) {
                    Ok(prompts_trans) => {
                        translations.extend(prompts_trans);
                        loaded_any = true;
                    }
                    Err(e) => {
                        eprintln!("⚠ 解析 Prompts 语言文件 {} 失败: {}", prompts_path.display(), e);
                    }
                },
                Err(e) => {
                    eprintln!("⚠ 读取 Prompts 语言文件 {} 失败: {}", prompts_path.display(), e);
                }
            }
        }

        // 4. 如果没有加载任何文件，使用内置翻译
        if !loaded_any {
            self.load_builtin_translations(lang);
        } else {
            self.translations.insert(lang, translations);
        }
    }

    /// 加载内置翻译（回退方案）
    fn load_builtin_translations(&mut self, lang: Language) {
        let translations = match lang {
            Language::ZhCn => builtin_translations_zh_cn(),
            Language::EnUs => builtin_translations_en_us(),
        };
        self.translations.insert(lang, translations);
    }

    /// 获取翻译字符串
    pub fn t(&self, key: &str) -> String {
        // 1. 尝试当前语言
        if let Some(translations) = self.translations.get(&self.current_lang) {
            if let Some(value) = translations.get(key) {
                return value.clone();
            }
        }

        // 2. 回退到默认语言
        if self.current_lang != self.fallback_lang {
            if let Some(translations) = self.translations.get(&self.fallback_lang) {
                if let Some(value) = translations.get(key) {
                    return value.clone();
                }
            }
        }

        // 3. 最后回退：返回键本身
        key.to_string()
    }

    /// 获取带参数的翻译字符串
    pub fn t_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut result = self.t(key);
        for (placeholder, value) in args {
            result = result.replace(&format!("{{{}}}", placeholder), value);
        }
        result
    }

    /// 切换语言
    pub fn set_language(&mut self, lang: Language) {
        self.current_lang = lang;
    }

    /// 获取当前语言
    pub fn current_language(&self) -> Language {
        self.current_lang
    }
}

/// 初始化全局 i18n 实例
pub fn init(lang: Language) {
    I18N.get_or_init(|| RwLock::new(I18n::new(lang)));
}

/// 获取翻译字符串（简便函数）
pub fn t(key: &str) -> String {
    if let Some(i18n) = I18N.get() {
        if let Ok(guard) = i18n.read() {
            return guard.t(key);
        }
    }
    key.to_string()
}

/// 获取带参数的翻译字符串（简便函数）
pub fn t_with_args(key: &str, args: &[(&str, &str)]) -> String {
    if let Some(i18n) = I18N.get() {
        if let Ok(guard) = i18n.read() {
            return guard.t_with_args(key, args);
        }
    }
    key.to_string()
}

/// 切换语言（简便函数）
pub fn set_language(lang: Language) {
    if let Some(i18n) = I18N.get() {
        if let Ok(mut guard) = i18n.write() {
            guard.set_language(lang);
        }
    }
}

/// 获取当前语言（简便函数）
pub fn current_language() -> Language {
    if let Some(i18n) = I18N.get() {
        if let Ok(guard) = i18n.read() {
            return guard.current_language();
        }
    }
    Language::default()
}

/// 内置中文翻译（作为回退）
fn builtin_translations_zh_cn() -> HashMap<String, String> {
    let mut map = HashMap::new();

    // 欢迎和提示
    map.insert(
        "welcome.version".to_string(),
        "RealConsole v{version}".to_string(),
    );
    map.insert("welcome.hint".to_string(), "直接输入问题或".to_string());
    map.insert("welcome.help".to_string(), "/help".to_string());
    map.insert("welcome.exit".to_string(), "Ctrl-D 退出".to_string());

    // 命令和操作
    map.insert("command.interrupted".to_string(), "^C".to_string());
    map.insert("command.bye".to_string(), "Bye 👋".to_string());
    map.insert("command.error".to_string(), "错误:".to_string());

    // 配置相关
    map.insert(
        "config.wizard_title".to_string(),
        "=== RealConsole 配置向导 ===".to_string(),
    );
    map.insert(
        "config.mode_quick".to_string(),
        "模式: 快速配置（使用推荐默认值）".to_string(),
    );
    map.insert(
        "config.mode_complete".to_string(),
        "模式: 完整配置（可自定义所有选项）".to_string(),
    );
    map.insert(
        "config.save_failed".to_string(),
        "✗ 保存配置失败:".to_string(),
    );
    map.insert(
        "config.wizard_failed".to_string(),
        "✗ 配置向导失败:".to_string(),
    );
    map.insert(
        "config.not_found".to_string(),
        "配置文件不存在:".to_string(),
    );
    map.insert(
        "config.run_wizard".to_string(),
        "请运行 'realconsole wizard' 创建配置".to_string(),
    );
    map.insert("config.file_label".to_string(), "配置文件:".to_string());
    map.insert(
        "config.read_failed".to_string(),
        "读取配置文件失败:".to_string(),
    );

    // 首次运行
    map.insert(
        "first_run.welcome".to_string(),
        "欢迎使用 RealConsole！".to_string(),
    );
    map.insert(
        "first_run.no_config".to_string(),
        "未检测到配置文件，首次使用需要进行配置。".to_string(),
    );
    map.insert(
        "first_run.choose_one".to_string(),
        "请选择以下方式之一：".to_string(),
    );
    map.insert(
        "first_run.option1".to_string(),
        "运行配置向导（推荐）".to_string(),
    );
    map.insert("first_run.option2".to_string(), "快速配置模式".to_string());
    map.insert(
        "first_run.option3".to_string(),
        "手动创建 realconsole.yaml 和 .env".to_string(),
    );
    map.insert(
        "first_run.hint".to_string(),
        "提示: 向导将帮助你在 2 分钟内完成配置".to_string(),
    );

    // LLM 相关
    map.insert(
        "llm.init_failed".to_string(),
        "⚠ {type} LLM 初始化失败:".to_string(),
    );
    map.insert(
        "llm.client_failed".to_string(),
        "{provider} 客户端创建失败:".to_string(),
    );
    map.insert(
        "llm.unknown_provider".to_string(),
        "未知的 LLM provider:".to_string(),
    );
    map.insert(
        "llm.need_api_key".to_string(),
        "{provider} 需要 api_key".to_string(),
    );

    // 错误和警告
    map.insert(
        "error.env_load_failed".to_string(),
        "⚠ .env 加载失败:".to_string(),
    );
    map.insert("error.repl_error".to_string(), "REPL 错误:".to_string());
    map.insert(
        "error.use_default_config".to_string(),
        "使用默认配置继续运行...".to_string(),
    );

    map
}

/// 内置英文翻译（作为回退）
fn builtin_translations_en_us() -> HashMap<String, String> {
    let mut map = HashMap::new();

    // Welcome and hints
    map.insert(
        "welcome.version".to_string(),
        "RealConsole v{version}".to_string(),
    );
    map.insert(
        "welcome.hint".to_string(),
        "Enter your question or".to_string(),
    );
    map.insert("welcome.help".to_string(), "/help".to_string());
    map.insert(
        "welcome.exit".to_string(),
        "Press Ctrl-D to exit".to_string(),
    );

    // Commands and operations
    map.insert("command.interrupted".to_string(), "^C".to_string());
    map.insert("command.bye".to_string(), "Bye 👋".to_string());
    map.insert("command.error".to_string(), "Error:".to_string());

    // Configuration
    map.insert(
        "config.wizard_title".to_string(),
        "=== RealConsole Configuration Wizard ===".to_string(),
    );
    map.insert(
        "config.mode_quick".to_string(),
        "Mode: Quick setup (using recommended defaults)".to_string(),
    );
    map.insert(
        "config.mode_complete".to_string(),
        "Mode: Complete setup (customize all options)".to_string(),
    );
    map.insert(
        "config.save_failed".to_string(),
        "✗ Failed to save configuration:".to_string(),
    );
    map.insert(
        "config.wizard_failed".to_string(),
        "✗ Configuration wizard failed:".to_string(),
    );
    map.insert(
        "config.not_found".to_string(),
        "Configuration file not found:".to_string(),
    );
    map.insert(
        "config.run_wizard".to_string(),
        "Please run 'realconsole wizard' to create configuration".to_string(),
    );
    map.insert(
        "config.file_label".to_string(),
        "Configuration file:".to_string(),
    );
    map.insert(
        "config.read_failed".to_string(),
        "Failed to read configuration file:".to_string(),
    );

    // First run
    map.insert(
        "first_run.welcome".to_string(),
        "Welcome to RealConsole!".to_string(),
    );
    map.insert(
        "first_run.no_config".to_string(),
        "No configuration file detected. Initial setup required.".to_string(),
    );
    map.insert(
        "first_run.choose_one".to_string(),
        "Please choose one of the following:".to_string(),
    );
    map.insert(
        "first_run.option1".to_string(),
        "Run configuration wizard (recommended)".to_string(),
    );
    map.insert(
        "first_run.option2".to_string(),
        "Quick configuration mode".to_string(),
    );
    map.insert(
        "first_run.option3".to_string(),
        "Manually create realconsole.yaml and .env".to_string(),
    );
    map.insert(
        "first_run.hint".to_string(),
        "Tip: The wizard will help you complete setup in 2 minutes".to_string(),
    );

    // LLM related
    map.insert(
        "llm.init_failed".to_string(),
        "⚠ {type} LLM initialization failed:".to_string(),
    );
    map.insert(
        "llm.client_failed".to_string(),
        "{provider} client creation failed:".to_string(),
    );
    map.insert(
        "llm.unknown_provider".to_string(),
        "Unknown LLM provider:".to_string(),
    );
    map.insert(
        "llm.need_api_key".to_string(),
        "{provider} requires api_key".to_string(),
    );

    // Errors and warnings
    map.insert(
        "error.env_load_failed".to_string(),
        "⚠ Failed to load .env:".to_string(),
    );
    map.insert("error.repl_error".to_string(), "REPL error:".to_string());
    map.insert(
        "error.use_default_config".to_string(),
        "Continuing with default configuration...".to_string(),
    );

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_str() {
        assert_eq!("zh-CN".parse::<Language>(), Ok(Language::ZhCn));
        assert_eq!("zh".parse::<Language>(), Ok(Language::ZhCn));
        assert_eq!("中文".parse::<Language>(), Ok(Language::ZhCn));
        assert_eq!("en-US".parse::<Language>(), Ok(Language::EnUs));
        assert_eq!("en".parse::<Language>(), Ok(Language::EnUs));
        assert_eq!("english".parse::<Language>(), Ok(Language::EnUs));
        assert!("unknown".parse::<Language>().is_err());
    }

    #[test]
    fn test_language_code() {
        assert_eq!(Language::ZhCn.code(), "zh-CN");
        assert_eq!(Language::EnUs.code(), "en-US");
    }

    #[test]
    fn test_i18n_translation() {
        let i18n = I18n::new(Language::ZhCn);
        let welcome = i18n.t("welcome.hint");
        assert!(welcome.contains("输入") || welcome == "welcome.hint");
    }

    #[test]
    fn test_i18n_fallback() {
        let i18n = I18n::new(Language::EnUs);
        // 如果 key 不存在，应该回退到 key 本身
        let result = i18n.t("nonexistent.key");
        assert_eq!(result, "nonexistent.key");
    }

    #[test]
    fn test_i18n_with_args() {
        let i18n = I18n::new(Language::ZhCn);
        let result = i18n.t_with_args("welcome.version", &[("version", "1.0.0")]);
        assert!(result.contains("1.0.0"));
    }
}
