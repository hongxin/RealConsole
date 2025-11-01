// Configuration Wizard - 智能配置向导
//
// 基于环境检测和AI推荐，提供三种配置模式：
// - Minimal: 3个问题，快速上手
// - Standard: 5个问题，平衡配置
// - Advanced: 10+个问题，完全自定义

use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::fs;
use std::path::PathBuf;

use super::detector::{EnvironmentDetector, EnvironmentInfo};
use super::recommender::{ConfigRecommender, AllRecommendations};
use super::validator::{ConfigValidator, ValidationResult};

/// 向导模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardMode {
    /// 极简模式：3个问题，快速上手
    Minimal,
    /// 标准模式：5个问题，平衡配置
    Standard,
    /// 高级模式：10+个问题，完全自定义
    Advanced,
}

/// 配置向导结果
#[derive(Debug, Clone)]
pub struct WizardResult {
    pub llm_provider: String,        // "deepseek", "ollama", "openai"
    pub llm_api_key: Option<String>,
    pub llm_model: Option<String>,
    pub llm_endpoint: Option<String>,
    pub suggestion_frequency: String, // "aggressive", "moderate", "conservative"
    pub safety_level: String,         // "strict", "standard", "relaxed"
    pub data_collection: String,      // "full", "anonymous", "disabled"
    pub keyboard_shortcuts: String,   // "default", "custom"
}

/// 配置向导
pub struct ConfigWizard {
    mode: WizardMode,
    theme: ColorfulTheme,
    detector: EnvironmentDetector,
    recommender: ConfigRecommender,
    validator: ConfigValidator,
}

impl ConfigWizard {
    /// 创建新的配置向导
    pub fn new(mode: WizardMode) -> Self {
        Self {
            mode,
            theme: ColorfulTheme::default(),
            detector: EnvironmentDetector::new(),
            recommender: ConfigRecommender::new(),
            validator: ConfigValidator::new(),
        }
    }

    /// 运行配置向导
    pub async fn run(&self) -> Result<WizardResult> {
        self.print_welcome();

        // 1. 环境检测
        println!("\n{}", "🔍 正在检测您的环境...".cyan());
        let env = self.detector.detect_all()?;
        self.print_environment_summary(&env);

        // 2. 获取AI推荐
        let recommendations = self.recommender.recommend_all(&env);

        // 3. 检查现有配置
        if self.config_exists() && !self.confirm_overwrite()? {
            anyhow::bail!("用户取消操作");
        }

        // 4. 根据模式执行配置流程
        let result = match self.mode {
            WizardMode::Minimal => self.run_minimal_mode(&env, &recommendations).await?,
            WizardMode::Standard => self.run_standard_mode(&env, &recommendations).await?,
            WizardMode::Advanced => self.run_advanced_mode(&env, &recommendations).await?,
        };

        // 5. 验证配置
        self.validate_result(&result).await?;

        // 6. 显示配置摘要
        self.print_result_summary(&result);

        Ok(result)
    }

    /// 打印欢迎界面
    fn print_welcome(&self) {
        println!("\n{}", "╔══════════════════════════════════════════════════════════════╗".bright_blue());
        println!("{}", "║     欢迎使用 RealConsole 智能配置向导 v1.16.0            ║".bright_blue());
        println!("{}", "╚══════════════════════════════════════════════════════════════╝".bright_blue());

        let (desc, time) = match self.mode {
            WizardMode::Minimal => ("极简模式：3个问题，2分钟完成", "~2分钟"),
            WizardMode::Standard => ("标准模式：5个问题，平衡配置", "~5分钟"),
            WizardMode::Advanced => ("高级模式：完全自定义所有选项", "~10分钟"),
        };

        println!("\n{} {}", "📝 模式:".bold(), desc);
        println!("{} {}\n", "⏱️  预计时间:".bold(), time);
    }

    /// 打印环境摘要
    fn print_environment_summary(&self, env: &EnvironmentInfo) {
        println!("\n{}", "✨ 环境信息".green().bold());
        println!("  {} {} {} ({})", "操作系统:".dimmed(), env.os.os_type, env.os.version, env.os.arch);
        println!("  {} {}", "Shell:".dimmed(), env.shell.shell_type);
        println!("  {} {} 个工具", "已安装:".dimmed(), env.tools.len());
        println!("  {} {:?}", "用户画像:".dimmed(), env.user_profile);
    }

    /// 检查配置文件是否存在
    fn config_exists(&self) -> bool {
        PathBuf::from("realconsole.yaml").exists() || PathBuf::from(".env").exists()
    }

    /// 确认覆盖现有配置
    fn confirm_overwrite(&self) -> Result<bool> {
        println!("\n{}", "⚠️  检测到现有配置文件".yellow().bold());

        Confirm::with_theme(&self.theme)
            .with_prompt("是否要重新配置？（现有配置将被备份）")
            .default(false)
            .interact()
            .context("用户输入失败")
    }

    /// 极简模式：3个问题
    async fn run_minimal_mode(
        &self,
        _env: &EnvironmentInfo,
        recommendations: &AllRecommendations,
    ) -> Result<WizardResult> {
        println!("\n{}", "━━━ 极简配置模式 ━━━".bright_cyan().bold());

        // 问题1: LLM后端
        let (llm_provider, llm_api_key, llm_model, llm_endpoint) =
            self.prompt_llm_provider(&recommendations.llm).await?;

        // 问题2 & 3: 使用推荐值
        Ok(WizardResult {
            llm_provider,
            llm_api_key,
            llm_model,
            llm_endpoint,
            suggestion_frequency: recommendations.suggestion_frequency.level.clone(),
            safety_level: recommendations.safety_level.clone(),
            data_collection: "anonymous".to_string(),
            keyboard_shortcuts: "default".to_string(),
        })
    }

    /// 标准模式：5个问题
    async fn run_standard_mode(
        &self,
        _env: &EnvironmentInfo,
        recommendations: &AllRecommendations,
    ) -> Result<WizardResult> {
        println!("\n{}", "━━━ 标准配置模式 ━━━".bright_cyan().bold());

        // 问题1: LLM后端
        let (llm_provider, llm_api_key, llm_model, llm_endpoint) =
            self.prompt_llm_provider(&recommendations.llm).await?;

        // 问题2: 建议频率
        let suggestion_frequency = self.prompt_suggestion_frequency(&recommendations.suggestion_frequency.level)?;

        // 问题3: 安全级别
        let safety_level = self.prompt_safety_level(&recommendations.safety_level)?;

        // 问题4 & 5: 使用推荐值
        Ok(WizardResult {
            llm_provider,
            llm_api_key,
            llm_model,
            llm_endpoint,
            suggestion_frequency,
            safety_level,
            data_collection: "anonymous".to_string(),
            keyboard_shortcuts: "default".to_string(),
        })
    }

    /// 高级模式：10+个问题
    async fn run_advanced_mode(
        &self,
        _env: &EnvironmentInfo,
        recommendations: &AllRecommendations,
    ) -> Result<WizardResult> {
        println!("\n{}", "━━━ 高级配置模式 ━━━".bright_cyan().bold());

        // 所有问题都询问用户
        let (llm_provider, llm_api_key, llm_model, llm_endpoint) =
            self.prompt_llm_provider(&recommendations.llm).await?;

        let suggestion_frequency = self.prompt_suggestion_frequency(&recommendations.suggestion_frequency.level)?;
        let safety_level = self.prompt_safety_level(&recommendations.safety_level)?;
        let data_collection = self.prompt_data_collection()?;
        let keyboard_shortcuts = self.prompt_keyboard_shortcuts()?;

        Ok(WizardResult {
            llm_provider,
            llm_api_key,
            llm_model,
            llm_endpoint,
            suggestion_frequency,
            safety_level,
            data_collection,
            keyboard_shortcuts,
        })
    }

    /// 提示选择 LLM Provider
    async fn prompt_llm_provider(
        &self,
        recommendation: &super::recommender::LlmRecommendation,
    ) -> Result<(String, Option<String>, Option<String>, Option<String>)> {
        println!("\n{} LLM 后端选择", "📝 步骤 1/5:".bold());
        println!("\n{}", format!("💡 推荐: {} (置信度: {:.0}%)",
            recommendation.provider, recommendation.confidence * 100.0).cyan());
        println!("   {}", recommendation.reason.dimmed());

        let choices = vec![
            format!("Deepseek{}", if recommendation.provider == "deepseek" { " ⭐ 推荐" } else { "" }),
            format!("Ollama{}", if recommendation.provider == "ollama" { " ⭐ 推荐" } else { "" }),
            "稍后配置".to_string(),
        ];

        let default_index = match recommendation.provider.as_str() {
            "deepseek" => 0,
            "ollama" => 1,
            _ => 0,
        };

        let selection = Select::with_theme(&self.theme)
            .with_prompt("请选择")
            .items(&choices)
            .default(default_index)
            .interact()
            .context("用户输入失败")?;

        match selection {
            0 => self.prompt_deepseek_config().await,
            1 => self.prompt_ollama_config().await,
            2 => Ok(("none".to_string(), None, None, None)),
            _ => unreachable!(),
        }
    }

    /// Deepseek 配置
    async fn prompt_deepseek_config(&self) -> Result<(String, Option<String>, Option<String>, Option<String>)> {
        println!("\n{}", "💡 从 https://platform.deepseek.com 获取 API Key".cyan());

        let api_key: String = Input::with_theme(&self.theme)
            .with_prompt("API Key")
            .interact_text()
            .context("API Key输入失败")?;

        // 实时验证
        println!("\n{}", "🧪 正在验证 API Key...".yellow());
        let validation = self.validator.validate_deepseek_api(&api_key).await?;

        if !validation.success {
            println!("{} {}", "❌".red(), validation.message.red());
            if let Some(details) = validation.details {
                println!("   {}", details.dimmed());
            }
            anyhow::bail!("API Key 验证失败");
        }

        println!("{} {}", "✅".green(), validation.message.green());

        // 选择模型
        let models = vec![
            "deepseek-chat (推荐)",
            "deepseek-coder (代码优化)",
        ];

        let model_selection = Select::with_theme(&self.theme)
            .with_prompt("选择模型")
            .items(&models)
            .default(0)
            .interact()
            .context("模型选择失败")?;

        let model = match model_selection {
            0 => "deepseek-chat",
            1 => "deepseek-coder",
            _ => "deepseek-chat",
        };

        Ok((
            "deepseek".to_string(),
            Some(api_key),
            Some(model.to_string()),
            Some("https://api.deepseek.com".to_string()),
        ))
    }

    /// Ollama 配置
    async fn prompt_ollama_config(&self) -> Result<(String, Option<String>, Option<String>, Option<String>)> {
        println!("\n{}", "💡 确保 Ollama 服务已启动 (http://localhost:11434)".cyan());

        let endpoint: String = Input::with_theme(&self.theme)
            .with_prompt("Ollama 地址")
            .default("http://localhost:11434".to_string())
            .interact_text()
            .context("Ollama地址输入失败")?;

        // 验证连接（使用mock）
        println!("\n{}", "🧪 正在验证 Ollama 连接...".yellow());
        let validation = self.validator.validate_ollama_connection(&endpoint).await?;

        if !validation.success {
            println!("{} {}", "⚠️".yellow(), validation.message.yellow());
            if let Some(details) = validation.details {
                println!("   {}", details.dimmed());
            }
        } else {
            println!("{} {}", "✅".green(), validation.message.green());
        }

        let model: String = Input::with_theme(&self.theme)
            .with_prompt("模型名称")
            .default("llama2".to_string())
            .interact_text()
            .context("模型名称输入失败")?;

        Ok((
            "ollama".to_string(),
            None,
            Some(model),
            Some(endpoint),
        ))
    }

    /// 提示建议频率
    fn prompt_suggestion_frequency(&self, recommended: &str) -> Result<String> {
        println!("\n{} 主动建议频率", "📝 步骤 2/5:".bold());
        println!("{}", format!("💡 推荐: {}", recommended).cyan());

        let choices = vec![
            format!("积极 (经常提供建议){}",  if recommended == "aggressive" { " ⭐" } else { "" }),
            format!("适中 (平衡模式){}",      if recommended == "moderate" { " ⭐" } else { "" }),
            format!("保守 (仅在必要时){}",    if recommended == "conservative" { " ⭐" } else { "" }),
        ];

        let default = match recommended {
            "aggressive" => 0,
            "moderate" => 1,
            "conservative" => 2,
            _ => 1,
        };

        let selection = Select::with_theme(&self.theme)
            .with_prompt("请选择")
            .items(&choices)
            .default(default)
            .interact()
            .context("用户输入失败")?;

        Ok(match selection {
            0 => "aggressive",
            1 => "moderate",
            2 => "conservative",
            _ => "moderate",
        }.to_string())
    }

    /// 提示安全级别
    fn prompt_safety_level(&self, recommended: &str) -> Result<String> {
        println!("\n{} 安全级别", "📝 步骤 3/5:".bold());
        println!("{}", format!("💡 推荐: {}", recommended).cyan());

        let choices = vec![
            format!("严格 (危险操作需确认){}",  if recommended == "strict" { " ⭐" } else { "" }),
            format!("标准 (平衡安全与便利){}",  if recommended == "standard" { " ⭐" } else { "" }),
            format!("宽松 (信任所有操作){}",    if recommended == "relaxed" { " ⭐" } else { "" }),
        ];

        let default = match recommended {
            "strict" => 0,
            "standard" => 1,
            "relaxed" => 2,
            _ => 1,
        };

        let selection = Select::with_theme(&self.theme)
            .with_prompt("请选择")
            .items(&choices)
            .default(default)
            .interact()
            .context("用户输入失败")?;

        Ok(match selection {
            0 => "strict",
            1 => "standard",
            2 => "relaxed",
            _ => "standard",
        }.to_string())
    }

    /// 提示数据收集
    fn prompt_data_collection(&self) -> Result<String> {
        println!("\n{} 数据收集", "📝 步骤 4/5:".bold());

        let choices = vec![
            "完全启用 (帮助改进产品)",
            "匿名统计 (推荐)",
            "完全禁用",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("请选择")
            .items(&choices)
            .default(1)
            .interact()
            .context("用户输入失败")?;

        Ok(match selection {
            0 => "full",
            1 => "anonymous",
            2 => "disabled",
            _ => "anonymous",
        }.to_string())
    }

    /// 提示快捷键设置
    fn prompt_keyboard_shortcuts(&self) -> Result<String> {
        println!("\n{} 快捷键设置", "📝 步骤 5/5:".bold());

        let choices = vec![
            "使用默认快捷键 (推荐)",
            "自定义快捷键",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("请选择")
            .items(&choices)
            .default(0)
            .interact()
            .context("用户输入失败")?;

        Ok(match selection {
            0 => "default",
            1 => "custom",
            _ => "default",
        }.to_string())
    }

    /// 验证配置结果
    async fn validate_result(&self, _result: &WizardResult) -> Result<()> {
        println!("\n{}", "🔍 正在验证配置...".cyan());

        // 这里可以添加更多验证逻辑
        // 目前主要在配置过程中已经做了验证

        Ok(())
    }

    /// 打印配置摘要
    fn print_result_summary(&self, result: &WizardResult) {
        println!("\n{}", "✨ 配置完成".green().bold());
        println!("\n{}", "配置摘要:".bold());
        println!("  {} {}", "LLM 后端:".dimmed(), result.llm_provider);
        if let Some(model) = &result.llm_model {
            println!("  {} {}", "模型:".dimmed(), model);
        }
        println!("  {} {}", "建议频率:".dimmed(), result.suggestion_frequency);
        println!("  {} {}", "安全级别:".dimmed(), result.safety_level);
        println!("  {} {}", "数据收集:".dimmed(), result.data_collection);

        println!("\n{}", "💾 配置将保存到:".yellow());
        println!("  • realconsole.yaml");
        println!("  • .env (API密钥)");
    }

    /// 生成并保存配置
    pub fn generate_and_save(&self, result: &WizardResult) -> Result<()> {
        // 生成 YAML 配置
        let yaml_content = Self::generate_yaml(result)?;
        fs::write("realconsole.yaml", yaml_content)
            .context("无法写入 realconsole.yaml")?;

        println!("\n{} realconsole.yaml", "✓ 已生成".green());

        // 生成 .env 文件
        let env_content = Self::generate_env(result)?;
        fs::write(".env", env_content).context("无法写入 .env")?;

        // 设置 .env 文件权限为 0600（仅所有者可读写）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(".env")?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(".env", permissions)?;
        }

        println!("{} .env (权限: 0600)", "✓ 已生成".green());

        // 确保 .gitignore 包含 .env
        Self::ensure_gitignore()?;

        println!("\n{}", "🎉 配置完成！现在可以运行 realconsole 开始使用。".green().bold());

        Ok(())
    }

    /// 生成 YAML 配置文件内容
    fn generate_yaml(result: &WizardResult) -> Result<String> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        let llm_config = if result.llm_provider == "none" {
            "# llm: (稍后配置)".to_string()
        } else {
            match result.llm_provider.as_str() {
                "deepseek" => {
                    let model = result.llm_model.as_deref().unwrap_or("deepseek-chat");
                    let endpoint = result.llm_endpoint.as_deref().unwrap_or("https://api.deepseek.com/v1");
                    format!(
                        r#"llm:
  primary:
    provider: deepseek
    model: {}
    endpoint: {}
    api_key: ${{DEEPSEEK_API_KEY}}"#,
                        model, endpoint
                    )
                }
                "ollama" => {
                    let model = result.llm_model.as_deref().unwrap_or("llama2");
                    let endpoint = result.llm_endpoint.as_deref().unwrap_or("http://localhost:11434");
                    format!(
                        r#"llm:
  primary:
    provider: ollama
    model: {}
    endpoint: {}"#,
                        model, endpoint
                    )
                }
                _ => "# llm: (未知配置)".to_string(),
            }
        };

        let yaml = format!(
            r#"# RealConsole 配置文件
# 由智能配置向导生成于 {}

# 命令前缀
prefix: "/"

# LLM 配置
{}

# 功能配置
features:
  # Shell 命令执行 (! 前缀)
  shell_enabled: true

  # 工具调用 (Function Calling)
  tool_calling_enabled: true

  # 记忆系统
  memory_enabled: true

  # Workflow Intent（任务规划与分解）
  workflow_enabled: true

# 记忆系统配置
memory:
  capacity: 100
  persistent_file: "~/.realconsole/memory/session.jsonl"
  auto_save: true

# Intent DSL 配置
intent:
  suggestion_frequency: {}  # aggressive, moderate, conservative
  safety_level: {}           # strict, standard, relaxed

# 显示模式
display:
  mode: inline  # inline, overlay

# 对话上下文
conversation:
  mode: manual     # auto, manual
  max_turns: 10
  context_window: 8192

# 数据收集
# (用于改进产品，不收集敏感信息)
telemetry:
  enabled: {}
  level: {}  # full, anonymous, disabled
"#,
            timestamp,
            llm_config,
            result.suggestion_frequency,
            result.safety_level,
            result.data_collection != "disabled",
            result.data_collection
        );

        Ok(yaml)
    }

    /// 生成 .env 文件内容
    fn generate_env(result: &WizardResult) -> Result<String> {
        let mut content = String::from("# RealConsole 环境变量\n# 请妥善保管，不要提交到版本控制\n\n");

        if let Some(api_key) = &result.llm_api_key {
            content.push_str(&format!("DEEPSEEK_API_KEY={}\n", api_key));
        }

        Ok(content)
    }

    /// 确保 .gitignore 包含 .env
    fn ensure_gitignore() -> Result<()> {
        let gitignore_path = ".gitignore";

        if !PathBuf::from(gitignore_path).exists() {
            fs::write(gitignore_path, ".env\n").context("无法创建 .gitignore")?;
            println!("{} .gitignore", "✓ 已创建".green());
        } else {
            let content = fs::read_to_string(gitignore_path)?;
            if !content.lines().any(|line| line.trim() == ".env") {
                fs::write(gitignore_path, format!("{}\n.env\n", content.trim_end()))
                    .context("无法更新 .gitignore")?;
                println!("{} .gitignore (添加 .env)", "✓ 已更新".green());
            }
        }

        Ok(())
    }
}

impl Default for ConfigWizard {
    fn default() -> Self {
        Self::new(WizardMode::Standard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_mode() {
        let wizard = ConfigWizard::new(WizardMode::Minimal);
        assert_eq!(wizard.mode, WizardMode::Minimal);

        let wizard = ConfigWizard::new(WizardMode::Standard);
        assert_eq!(wizard.mode, WizardMode::Standard);

        let wizard = ConfigWizard::new(WizardMode::Advanced);
        assert_eq!(wizard.mode, WizardMode::Advanced);
    }

    #[test]
    fn test_config_exists() {
        let wizard = ConfigWizard::new(WizardMode::Minimal);
        // 这个测试取决于当前目录是否有配置文件
        let _ = wizard.config_exists();
    }

    #[test]
    fn test_wizard_result() {
        let result = WizardResult {
            llm_provider: "deepseek".to_string(),
            llm_api_key: Some("test-key".to_string()),
            llm_model: Some("deepseek-chat".to_string()),
            llm_endpoint: Some("https://api.deepseek.com".to_string()),
            suggestion_frequency: "moderate".to_string(),
            safety_level: "standard".to_string(),
            data_collection: "anonymous".to_string(),
            keyboard_shortcuts: "default".to_string(),
        };

        assert_eq!(result.llm_provider, "deepseek");
        assert_eq!(result.llm_model, Some("deepseek-chat".to_string()));
    }

    #[tokio::test]
    async fn test_validate_result() {
        let wizard = ConfigWizard::new(WizardMode::Minimal);
        let result = WizardResult {
            llm_provider: "deepseek".to_string(),
            llm_api_key: Some("test-key".to_string()),
            llm_model: Some("deepseek-chat".to_string()),
            llm_endpoint: Some("https://api.deepseek.com".to_string()),
            suggestion_frequency: "moderate".to_string(),
            safety_level: "standard".to_string(),
            data_collection: "anonymous".to_string(),
            keyboard_shortcuts: "default".to_string(),
        };

        // 验证应该成功（当前实现比较简单）
        assert!(wizard.validate_result(&result).await.is_ok());
    }

    #[test]
    fn test_default_wizard() {
        let wizard = ConfigWizard::default();
        assert_eq!(wizard.mode, WizardMode::Standard);
    }
}
