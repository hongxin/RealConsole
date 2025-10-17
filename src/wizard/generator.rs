//! 配置文件生成器

use anyhow::{Context, Result};
use chrono::Local;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::wizard::{LlmProviderChoice, WizardConfig};

/// 配置文件生成器
pub struct ConfigGenerator;

impl ConfigGenerator {
    /// 生成并保存配置文件
    pub fn save_config(config: &WizardConfig) -> Result<()> {
        // 生成 YAML 配置
        let yaml_content = Self::generate_yaml(config)?;
        fs::write("realconsole.yaml", yaml_content)
            .context("无法写入 realconsole.yaml")?;

        println!("✓ 已生成 realconsole.yaml");

        // 生成 .env 文件
        let env_content = Self::generate_env(config)?;
        fs::write(".env", env_content).context("无法写入 .env")?;

        // 设置 .env 文件权限为 0600（仅所有者可读写）
        #[cfg(unix)]
        {
            let metadata = fs::metadata(".env")?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o600);
            fs::set_permissions(".env", permissions)?;
        }

        println!("✓ 已生成 .env (权限: 0600)");

        // 确保 .gitignore 包含 .env
        Self::ensure_gitignore()?;

        Ok(())
    }

    /// 生成 YAML 配置文件内容
    fn generate_yaml(config: &WizardConfig) -> Result<String> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        let llm_config = match &config.llm_provider {
            LlmProviderChoice::Deepseek { model, .. } => {
                format!(
                    r#"llm:
  primary:
    provider: deepseek
    model: {}
    endpoint: https://api.deepseek.com/v1
    api_key: ${{DEEPSEEK_API_KEY}}"#,
                    model
                )
            }
            LlmProviderChoice::Ollama { endpoint, model } => {
                format!(
                    r#"llm:
  primary:
    provider: ollama
    model: {}
    endpoint: {}"#,
                    model, endpoint
                )
            }
        };

        let memory_config = if config.memory_enabled {
            r#"memory:
  capacity: 100
  persistent_file: "memory/session.jsonl"
  auto_save: true"#
        } else {
            "# memory: (未启用)"
        };

        let yaml = format!(
            r#"# RealConsole 配置文件
# 由配置向导自动生成于 {}

# 命令前缀
prefix: "/"

# LLM 配置
{}

# 功能配置
features:
  shell_enabled: {}
  shell_timeout: 10
  tool_calling_enabled: {}
  max_tool_iterations: 5
  max_tools_per_round: 3

# 记忆系统
{}

# Intent DSL 配置（默认值已优化）
intent:
  llm_extraction_enabled: false
  llm_validation_enabled: false
  validation_threshold: 0.7
  require_confirmation: true
"#,
            timestamp,
            llm_config,
            config.shell_enabled,
            config.tool_calling_enabled,
            memory_config
        );

        Ok(yaml)
    }

    /// 生成 .env 文件内容
    fn generate_env(config: &WizardConfig) -> Result<String> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");

        let mut lines = vec![
            "# RealConsole 环境变量".to_string(),
            format!("# 由配置向导自动生成于 {}", timestamp),
            "#".to_string(),
            "# 警告：此文件包含敏感信息，请勿提交到 git！".to_string(),
            "# 已自动添加到 .gitignore".to_string(),
            "".to_string(),
        ];

        match &config.llm_provider {
            LlmProviderChoice::Deepseek { api_key, endpoint, .. } => {
                lines.push("# Deepseek API 配置".to_string());
                lines.push(format!("DEEPSEEK_API_KEY={}", api_key));
                if endpoint != "https://api.deepseek.com/v1" {
                    lines.push(format!("DEEPSEEK_ENDPOINT={}", endpoint));
                }
            }
            LlmProviderChoice::Ollama { endpoint, .. } => {
                lines.push("# Ollama 配置".to_string());
                if endpoint != "http://localhost:11434" {
                    lines.push(format!("OLLAMA_ENDPOINT={}", endpoint));
                }
            }
        }

        lines.push("".to_string());
        lines.push("# 可选：调试模式".to_string());
        lines.push("# RUST_LOG=debug".to_string());
        lines.push("# RUST_BACKTRACE=1".to_string());

        Ok(lines.join("\n"))
    }

    /// 确保 .gitignore 包含 .env
    fn ensure_gitignore() -> Result<()> {
        let gitignore_path = ".gitignore";

        if Path::new(gitignore_path).exists() {
            let content = fs::read_to_string(gitignore_path)?;

            if !content.lines().any(|line| line.trim() == ".env") {
                let mut new_content = content;
                if !new_content.ends_with('\n') {
                    new_content.push('\n');
                }
                new_content.push_str(".env\n");

                fs::write(gitignore_path, new_content)?;
                println!("✓ 已更新 .gitignore（添加 .env）");
            }
        } else {
            // 创建新的 .gitignore
            fs::write(gitignore_path, ".env\n")?;
            println!("✓ 已创建 .gitignore（添加 .env）");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_yaml_deepseek() {
        let config = WizardConfig {
            llm_provider: LlmProviderChoice::Deepseek {
                api_key: "sk-test".to_string(),
                model: "deepseek-chat".to_string(),
                endpoint: "https://api.deepseek.com/v1".to_string(),
            },
            shell_enabled: true,
            tool_calling_enabled: true,
            memory_enabled: false,
        };

        let yaml = ConfigGenerator::generate_yaml(&config).unwrap();

        // 验证关键内容
        assert!(yaml.contains("provider: deepseek"));
        assert!(yaml.contains("model: deepseek-chat"));
        assert!(yaml.contains("${DEEPSEEK_API_KEY}")); // API key 应使用环境变量
        assert!(!yaml.contains("sk-test")); // 不应包含实际 API key
        assert!(yaml.contains("shell_enabled: true"));
        assert!(yaml.contains("tool_calling_enabled: true"));
        assert!(yaml.contains("# memory: (未启用)"));
    }

    #[test]
    fn test_generate_yaml_ollama() {
        let config = WizardConfig {
            llm_provider: LlmProviderChoice::Ollama {
                endpoint: "http://localhost:11434".to_string(),
                model: "qwen3:4b".to_string(),
            },
            shell_enabled: true,
            tool_calling_enabled: true,
            memory_enabled: true,
        };

        let yaml = ConfigGenerator::generate_yaml(&config).unwrap();

        assert!(yaml.contains("provider: ollama"));
        assert!(yaml.contains("model: qwen3:4b"));
        assert!(yaml.contains("endpoint: http://localhost:11434"));
        assert!(yaml.contains("memory:"));
        assert!(yaml.contains("capacity: 100"));
    }

    #[test]
    fn test_generate_env_deepseek() {
        let config = WizardConfig {
            llm_provider: LlmProviderChoice::Deepseek {
                api_key: "sk-test-key-12345".to_string(),
                model: "deepseek-chat".to_string(),
                endpoint: "https://api.deepseek.com/v1".to_string(),
            },
            shell_enabled: true,
            tool_calling_enabled: true,
            memory_enabled: false,
        };

        let env = ConfigGenerator::generate_env(&config).unwrap();

        assert!(env.contains("DEEPSEEK_API_KEY=sk-test-key-12345"));
        assert!(env.contains("# 警告：此文件包含敏感信息"));
    }

    #[test]
    fn test_generate_env_ollama() {
        let config = WizardConfig {
            llm_provider: LlmProviderChoice::Ollama {
                endpoint: "http://localhost:11434".to_string(),
                model: "qwen3:4b".to_string(),
            },
            shell_enabled: true,
            tool_calling_enabled: true,
            memory_enabled: false,
        };

        let env = ConfigGenerator::generate_env(&config).unwrap();

        // Ollama 使用默认 endpoint，不应该写入 .env
        assert!(!env.contains("OLLAMA_ENDPOINT"));
    }

    #[test]
    fn test_generate_env_ollama_custom_endpoint() {
        let config = WizardConfig {
            llm_provider: LlmProviderChoice::Ollama {
                endpoint: "http://192.168.1.100:11434".to_string(),
                model: "qwen3:4b".to_string(),
            },
            shell_enabled: true,
            tool_calling_enabled: true,
            memory_enabled: false,
        };

        let env = ConfigGenerator::generate_env(&config).unwrap();

        // 非默认 endpoint 应该写入 .env
        assert!(env.contains("OLLAMA_ENDPOINT=http://192.168.1.100:11434"));
    }
}
