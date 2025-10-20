//! 配置向导核心逻辑

use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password, Select};
use std::path::Path;

use super::generator::ConfigGenerator;
use super::validator::ApiValidator;

/// 向导模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WizardMode {
    /// 快速配置（最小提问，推荐新用户）
    Quick,
    /// 完整配置（所有选项，高级用户）
    Complete,
}

/// LLM Provider 选择
#[derive(Debug, Clone)]
pub enum LlmProviderChoice {
    Deepseek {
        api_key: String,
        model: String,
        endpoint: String,
    },
    Ollama {
        endpoint: String,
        model: String,
    },
}

/// 向导配置（用户选择结果）
#[derive(Debug, Clone)]
pub struct WizardConfig {
    pub llm_provider: LlmProviderChoice,
    pub shell_enabled: bool,
    pub tool_calling_enabled: bool,
    pub memory_enabled: bool,
    pub workflow_enabled: bool,
}

/// 配置向导
pub struct ConfigWizard {
    mode: WizardMode,
    theme: ColorfulTheme,
    validator: ApiValidator,
}

impl ConfigWizard {
    /// 创建新的配置向导
    pub fn new(mode: WizardMode) -> Self {
        Self {
            mode,
            theme: ColorfulTheme::default(),
            validator: ApiValidator::new(),
        }
    }

    /// 运行配置向导
    pub async fn run(&self) -> Result<WizardConfig> {
        self.print_welcome();

        // 检查现有配置
        if Path::new("realconsole.yaml").exists() && !self.confirm_overwrite()? {
            anyhow::bail!("用户取消");
        }

        // 选择 LLM Provider
        let llm_provider = self.prompt_llm_provider().await?;

        // 配置功能开关
        let shell_enabled = self.prompt_shell_enabled()?;
        let tool_calling_enabled = self.prompt_tool_calling()?;
        let memory_enabled = self.prompt_memory()?;
        let workflow_enabled = self.prompt_workflow()?;

        Ok(WizardConfig {
            llm_provider,
            shell_enabled,
            tool_calling_enabled,
            memory_enabled,
            workflow_enabled,
        })
    }

    /// 打印欢迎界面
    fn print_welcome(&self) {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║         欢迎使用 RealConsole 配置向导 v0.7.0              ║");
        println!("╚══════════════════════════════════════════════════════════════╝\n");

        match self.mode {
            WizardMode::Quick => {
                println!("这个向导将帮助你在 2 分钟内完成快速配置。\n");
            }
            WizardMode::Complete => {
                println!("这个向导将引导你完成所有配置选项。\n");
            }
        }
    }

    /// 确认覆盖现有配置
    fn confirm_overwrite(&self) -> Result<bool> {
        println!("⚠️  检测到现有配置文件 (realconsole.yaml)\n");

        let choices = vec!["更新配置（推荐）", "重新配置（覆盖所有设置）", "取消"];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("如何处理")
            .items(&choices)
            .default(0)
            .interact()?;

        Ok(selection != 2) // 0 或 1 都继续，2 取消
    }

    /// 提示选择 LLM Provider
    async fn prompt_llm_provider(&self) -> Result<LlmProviderChoice> {
        println!("\n━━━ LLM Provider 配置 ━━━\n");

        let choices = vec![
            "Deepseek (远程 API，功能强大，推荐)",
            "Ollama (本地模型，隐私优先)",
        ];

        let selection = Select::with_theme(&self.theme)
            .with_prompt("选择 LLM Provider")
            .items(&choices)
            .default(0)
            .interact()?;

        match selection {
            0 => self.prompt_deepseek_config().await,
            1 => self.prompt_ollama_config().await,
            _ => unreachable!(),
        }
    }

    /// 提示 Deepseek 配置
    async fn prompt_deepseek_config(&self) -> Result<LlmProviderChoice> {
        println!("\n💡 提示: 从 https://platform.deepseek.com 获取 API Key\n");

        let api_key = self.prompt_api_key_with_validation().await?;

        // 选择模型
        let models = vec![
            "deepseek-chat (推荐，平衡性能与成本)",
            "deepseek-coder (代码优化)",
        ];

        let model_idx = if self.mode == WizardMode::Complete {
            Select::with_theme(&self.theme)
                .with_prompt("选择 Deepseek 模型")
                .items(&models)
                .default(0)
                .interact()?
        } else {
            0 // 快速模式使用默认
        };

        let model = match model_idx {
            0 => "deepseek-chat",
            1 => "deepseek-coder",
            _ => "deepseek-chat",
        }
        .to_string();

        Ok(LlmProviderChoice::Deepseek {
            api_key,
            model,
            endpoint: "https://api.deepseek.com/v1".to_string(),
        })
    }

    /// 提示并验证 API Key
    async fn prompt_api_key_with_validation(&self) -> Result<String> {
        use indicatif::{ProgressBar, ProgressStyle};

        loop {
            let api_key: String = Password::with_theme(&self.theme)
                .with_prompt("请输入 Deepseek API Key")
                .interact()?;

            if api_key.trim().is_empty() {
                println!("✗ API Key 不能为空\n");
                continue;
            }

            // 显示验证进度
            let spinner = ProgressBar::new_spinner();
            spinner.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} {msg}")
                    .unwrap(),
            );
            spinner.set_message("正在验证 API Key...");
            spinner.enable_steady_tick(std::time::Duration::from_millis(100));

            let validation_result = self
                .validator
                .validate_deepseek_key(&api_key, "https://api.deepseek.com/v1")
                .await;

            spinner.finish_and_clear();

            match validation_result {
                Ok(true) => {
                    println!("✓ API Key 验证成功！\n");
                    return Ok(api_key);
                }
                Ok(false) => {
                    println!("✗ API Key 无效（请检查是否正确）\n");

                    if !Confirm::with_theme(&self.theme)
                        .with_prompt("重新输入")
                        .default(true)
                        .interact()?
                    {
                        anyhow::bail!("用户取消");
                    }
                }
                Err(e) => {
                    println!("⚠️  验证失败: {}\n", e);
                    println!("可能原因：网络问题或服务不可用\n");

                    let choices = vec!["重试", "跳过验证（不推荐）", "取消"];
                    let choice = Select::with_theme(&self.theme)
                        .with_prompt("如何处理")
                        .items(&choices)
                        .default(0)
                        .interact()?;

                    match choice {
                        0 => continue,
                        1 => {
                            println!("⚠️  已跳过验证，请确保 API Key 正确\n");
                            return Ok(api_key);
                        }
                        2 => anyhow::bail!("用户取消"),
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    /// 提示 Ollama 配置
    async fn prompt_ollama_config(&self) -> Result<LlmProviderChoice> {
        let endpoint: String = if self.mode == WizardMode::Complete {
            Input::with_theme(&self.theme)
                .with_prompt("Ollama endpoint")
                .default("http://localhost:11434".to_string())
                .interact()?
        } else {
            "http://localhost:11434".to_string()
        };

        // 检测 Ollama 服务并列出可用模型
        use indicatif::{ProgressBar, ProgressStyle};

        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.set_message("正在检测 Ollama 服务...");
        spinner.enable_steady_tick(std::time::Duration::from_millis(100));

        let models_result = self.validator.check_ollama_service(&endpoint).await;
        spinner.finish_and_clear();

        let model = match models_result {
            Ok(models) if !models.is_empty() => {
                println!("✓ Ollama 服务可用，检测到 {} 个模型\n", models.len());

                let selection = Select::with_theme(&self.theme)
                    .with_prompt("选择模型")
                    .items(&models)
                    .default(0)
                    .interact()?;

                models[selection].clone()
            }
            Ok(_) => {
                println!("⚠️  Ollama 服务可用，但未检测到模型\n");
                println!("请先运行: ollama pull qwen3:4b\n");

                Input::with_theme(&self.theme)
                    .with_prompt("手动输入模型名称")
                    .default("qwen3:4b".to_string())
                    .interact()?
            }
            Err(e) => {
                println!("✗ Ollama 服务不可用: {}\n", e);
                println!("请确保 Ollama 已安装并运行: ollama serve\n");

                if !Confirm::with_theme(&self.theme)
                    .with_prompt("继续配置（稍后启动 Ollama）")
                    .default(false)
                    .interact()?
                {
                    anyhow::bail!("用户取消");
                }

                Input::with_theme(&self.theme)
                    .with_prompt("输入模型名称")
                    .default("qwen3:4b".to_string())
                    .interact()?
            }
        };

        Ok(LlmProviderChoice::Ollama { endpoint, model })
    }

    /// 提示 Shell 命令执行配置
    fn prompt_shell_enabled(&self) -> Result<bool> {
        println!("\n━━━ 功能配置 ━━━\n");

        if self.mode == WizardMode::Quick {
            println!("✓ Shell 命令执行: 已启用（安全黑名单保护）");
            Ok(true)
        } else {
            Confirm::with_theme(&self.theme)
                .with_prompt("启用 Shell 命令执行？(安全黑名单已启用)")
                .default(true)
                .interact()
                .context("用户取消")
        }
    }

    /// 提示 Tool Calling 配置
    fn prompt_tool_calling(&self) -> Result<bool> {
        if self.mode == WizardMode::Quick {
            println!("✓ Tool Calling: 已启用（支持 14+ 内置工具）");
            Ok(true)
        } else {
            Confirm::with_theme(&self.theme)
                .with_prompt("启用 Tool Calling（函数调用）？")
                .default(true)
                .interact()
                .context("用户取消")
        }
    }

    /// 提示记忆系统配置
    fn prompt_memory(&self) -> Result<bool> {
        if self.mode == WizardMode::Quick {
            println!("✓ 记忆系统: 已启用");
            Ok(true)
        } else {
            Confirm::with_theme(&self.theme)
                .with_prompt("启用记忆系统？")
                .default(true)
                .interact()
                .context("用户取消")
        }
    }

    /// 提示 Workflow Intent 系统配置
    fn prompt_workflow(&self) -> Result<bool> {
        if self.mode == WizardMode::Quick {
            println!("✓ Workflow Intent: 已禁用（可在配置文件中启用）\n");
            Ok(false)
        } else {
            println!("\n💡 Workflow Intent 系统可将常用任务模式固化为模板，提升 40-50% 性能");
            Confirm::with_theme(&self.theme)
                .with_prompt("启用 Workflow Intent 系统？（实验性功能）")
                .default(false)
                .interact()
                .context("用户取消")
        }
    }

    /// 生成配置文件并显示下一步提示
    pub fn generate_and_save(&self, config: &WizardConfig) -> Result<()> {
        println!("\n━━━ 生成配置文件 ━━━\n");

        // 生成并保存配置
        ConfigGenerator::save_config(config)?;

        // 显示成功消息和下一步提示
        self.print_completion();

        Ok(())
    }

    /// 打印完成信息
    fn print_completion(&self) {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
        println!("✓ 配置完成！已生成以下文件：\n");
        println!("  📄 realconsole.yaml    配置文件");
        println!("  🔐 .env                环境变量（已添加到 .gitignore）\n");
        println!("下一步：\n");
        println!("  1. 启动 RealConsole:");
        println!("     $ cargo run --release\n");
        println!("  2. 尝试对话:");
        println!("     > 你好，请介绍一下自己\n");
        println!("  3. 查看帮助:");
        println!("     > /help\n");
        println!("  4. 尝试 Shell 命令:");
        println!("     > !ls -la\n");
        println!("  5. 使用 Tool Calling:");
        println!("     > 帮我计算 (12 + 34) * 56\n");
        println!("需要帮助？访问: https://github.com/your-repo/realconsole\n");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wizard_creation() {
        let wizard = ConfigWizard::new(WizardMode::Quick);
        assert_eq!(wizard.mode, WizardMode::Quick);
    }

    #[test]
    fn test_wizard_mode_eq() {
        assert_eq!(WizardMode::Quick, WizardMode::Quick);
        assert_eq!(WizardMode::Complete, WizardMode::Complete);
        assert_ne!(WizardMode::Quick, WizardMode::Complete);
    }
}
