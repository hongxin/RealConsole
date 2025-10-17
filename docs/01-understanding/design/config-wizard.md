# 配置向导设计文档

**日期**: 2025-10-15
**阶段**: Phase 5.3 Week 2 - UX 改进
**状态**: 设计中

---

## 概述

配置向导是一个交互式 CLI 工具，帮助用户在首次使用或重新配置 RealConsole 时，通过问答方式生成配置文件（`realconsole.yaml` 和 `.env`）。

### 设计目标

1. **零门槛上手**: 新用户无需阅读文档即可完成配置
2. **智能验证**: 实时验证 API key 和配置项
3. **安全优先**: 敏感信息存储在 `.env`，不提交到 git
4. **可恢复**: 支持检测现有配置，提供更新选项
5. **灵活性**: 支持最小配置（快速上手）和完整配置（高级用户）

---

## 用户流程

### 流程图

```
开始
  ↓
检测现有配置？
  ├─ 是 → 询问是否覆盖 → [覆盖/更新/退出]
  └─ 否 → 继续
  ↓
欢迎界面
  ↓
选择配置模式
  ├─ 快速配置（推荐）
  └─ 完整配置（高级）
  ↓
选择 LLM Provider
  ├─ Deepseek（远程，推荐）
  │   ├─ 输入 API key
  │   ├─ 验证 API key
  │   └─ 选择 model（默认 deepseek-chat）
  └─ Ollama（本地）
      ├─ 输入 endpoint（默认 localhost:11434）
      ├─ 检测 Ollama 服务
      └─ 选择 model（自动检测可用模型）
  ↓
配置功能开关
  ├─ Shell 命令执行（默认启用）
  ├─ Tool Calling（默认启用）
  └─ 记忆系统（可选）
  ↓
生成配置文件
  ├─ realconsole.yaml
  └─ .env
  ↓
显示下一步提示
  ↓
结束
```

### 交互示例

```
╔══════════════════════════════════════════════════════════════╗
║          欢迎使用 RealConsole 配置向导 v0.6.0              ║
╚══════════════════════════════════════════════════════════════╝

这个向导将帮助你在几分钟内完成 RealConsole 的初始配置。

? 检测到现有配置文件，如何处理？
  ▸ 更新配置（保留现有设置）
    重新配置（覆盖所有设置）
    退出向导

? 选择配置模式：
  ▸ 快速配置（推荐新用户，约 2 分钟）
    完整配置（高级用户，自定义所有选项）

? 选择 LLM Provider：
  ▸ Deepseek（远程 API，功能强大，推荐）
    Ollama（本地模型，隐私优先）

? 请输入 Deepseek API Key:
  提示: 从 https://platform.deepseek.com 获取
  输入: sk-********************************

✓ API Key 验证成功！

? 选择 Deepseek 模型：
  ▸ deepseek-chat（推荐，平衡性能与成本）
    deepseek-coder（代码优化）

? 启用 Shell 命令执行？(安全黑名单已启用)
  ▸ 是（推荐，增强功能）
    否（仅对话功能）

? 启用 Tool Calling（函数调用）？
  ▸ 是（推荐，支持文件操作、计算等）
    否（纯对话模式）

? 启用记忆系统？
    是（记录会话历史）
  ▸ 否（无状态模式）

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ 配置完成！已生成以下文件：

  📄 realconsole.yaml    配置文件
  🔐 .env                环境变量（已添加到 .gitignore）

下一步：

  1. 启动 RealConsole:
     $ ./target/release/realconsole

  2. 尝试对话:
     > 你好，请介绍一下自己

  3. 查看帮助:
     > /help

  4. 尝试 Shell 命令:
     > !ls -la

  5. 使用 Tool Calling:
     > 帮我计算 (12 + 34) * 56

需要帮助？访问: https://github.com/your-repo/realconsole

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 技术架构

### 模块结构

```
src/wizard/
├── mod.rs              # 模块导出
├── wizard.rs           # 主向导逻辑
├── prompts.rs          # 交互提示
├── validator.rs        # 配置验证（API key、endpoint 等）
├── generator.rs        # 配置文件生成
└── templates.rs        # 配置模板
```

### 核心类型

```rust
/// 配置向导
pub struct ConfigWizard {
    mode: WizardMode,
    existing_config: Option<Config>,
}

/// 向导模式
pub enum WizardMode {
    Quick,      // 快速配置（最小提问）
    Complete,   // 完整配置（所有选项）
}

/// 向导配置（用户选择）
pub struct WizardConfig {
    pub llm_provider: LlmProviderChoice,
    pub shell_enabled: bool,
    pub tool_calling_enabled: bool,
    pub memory_enabled: bool,
}

/// LLM Provider 选择
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
```

### API 验证流程

```rust
/// API Key 验证器
pub struct ApiValidator {
    client: reqwest::Client,
}

impl ApiValidator {
    /// 验证 Deepseek API Key
    pub async fn validate_deepseek_key(
        &self,
        api_key: &str,
        endpoint: &str
    ) -> Result<bool> {
        // 发送测试请求
        let response = self.client
            .post(format!("{}/chat/completions", endpoint))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "model": "deepseek-chat",
                "messages": [{"role": "user", "content": "test"}],
                "max_tokens": 1
            }))
            .send()
            .await?;

        // 200 或 400（参数错误）都说明 key 有效
        // 401 说明 key 无效
        Ok(response.status() != StatusCode::UNAUTHORIZED)
    }

    /// 检测 Ollama 服务
    pub async fn check_ollama_service(
        &self,
        endpoint: &str
    ) -> Result<Vec<String>> {
        // GET /api/tags 获取模型列表
        let response = self.client
            .get(format!("{}/api/tags", endpoint))
            .send()
            .await?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await?;
            let models = data["models"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|m| m["name"].as_str())
                        .map(String::from)
                        .collect()
                })
                .unwrap_or_default();
            Ok(models)
        } else {
            Err(anyhow!("Ollama 服务不可用"))
        }
    }
}
```

### 配置生成器

```rust
/// 配置文件生成器
pub struct ConfigGenerator;

impl ConfigGenerator {
    /// 生成 YAML 配置
    pub fn generate_yaml(config: &WizardConfig) -> Result<String> {
        match &config.llm_provider {
            LlmProviderChoice::Deepseek { model, .. } => {
                Ok(format!(r#"
# RealConsole 配置文件
# 由配置向导自动生成于 {}

prefix: "/"

llm:
  primary:
    provider: deepseek
    model: {}
    endpoint: https://api.deepseek.com/v1
    api_key: ${{DEEPSEEK_API_KEY}}

features:
  shell_enabled: {}
  tool_calling_enabled: {}
  max_tool_iterations: 5
  max_tools_per_round: 3

{}
"#,
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    model,
                    config.shell_enabled,
                    config.tool_calling_enabled,
                    if config.memory_enabled {
                        "memory:\n  capacity: 100\n  persistent_file: \"memory/session.jsonl\"\n  auto_save: true"
                    } else {
                        "# memory: (未启用)"
                    }
                ))
            }
            LlmProviderChoice::Ollama { endpoint, model } => {
                // Ollama 配置模板
                todo!()
            }
        }
    }

    /// 生成 .env 文件
    pub fn generate_env(config: &WizardConfig) -> Result<String> {
        let mut lines = vec![
            "# RealConsole 环境变量".to_string(),
            format!("# 由配置向导自动生成于 {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")),
            "".to_string(),
        ];

        match &config.llm_provider {
            LlmProviderChoice::Deepseek { api_key, .. } => {
                lines.push(format!("DEEPSEEK_API_KEY={}", api_key));
            }
            LlmProviderChoice::Ollama { endpoint, .. } => {
                lines.push(format!("OLLAMA_ENDPOINT={}", endpoint));
            }
        }

        Ok(lines.join("\n"))
    }
}
```

---

## 交互库选择

### 方案对比

| 库 | 优点 | 缺点 | 评分 |
|---|------|------|------|
| **dialoguer** | 功能丰富、活跃维护 | 依赖较多 | ⭐⭐⭐⭐⭐ |
| **inquire** | API 简洁、现代化 | 相对较新 | ⭐⭐⭐⭐☆ |
| **requestty** | 类似 inquirer.js | 活跃度低 | ⭐⭐⭐☆☆ |
| **rustyline** | 已在项目中使用 | 需要手动实现 UI | ⭐⭐⭐☆☆ |

### 推荐方案：dialoguer

**理由**：
1. 功能完整（Select, Input, Confirm, Password 等）
2. 活跃维护（18k+ stars）
3. API 简洁易用
4. 支持主题和样式定制
5. 社区成熟，文档完善

**依赖**：
```toml
[dependencies]
dialoguer = "0.11"
console = "0.15"  # dialoguer 依赖
```

**示例代码**：
```rust
use dialoguer::{Select, Input, Confirm, Password, theme::ColorfulTheme};

let theme = ColorfulTheme::default();

// 选择框
let provider = Select::with_theme(&theme)
    .with_prompt("选择 LLM Provider")
    .items(&["Deepseek (推荐)", "Ollama (本地)"])
    .default(0)
    .interact()?;

// 输入框（密码）
let api_key = Password::with_theme(&theme)
    .with_prompt("请输入 Deepseek API Key")
    .interact()?;

// 确认框
let shell_enabled = Confirm::with_theme(&theme)
    .with_prompt("启用 Shell 命令执行？")
    .default(true)
    .interact()?;
```

---

## 安全考虑

### API Key 保护

1. **输入时隐藏**: 使用 `Password` 组件，输入时显示 `*`
2. **存储隔离**: API key 仅存储在 `.env`，不写入 YAML
3. **Git 忽略**: 自动检测 `.gitignore`，确保 `.env` 已添加
4. **权限控制**: 生成的 `.env` 设置为 `0600`（仅所有者可读写）

### 验证安全

1. **超时控制**: API 验证请求设置 5 秒超时
2. **最小权限**: 验证请求仅发送最小必要数据
3. **错误处理**: 不泄露敏感信息（如完整的 API key）

```rust
// 示例：安全的错误消息
match validate_result {
    Ok(true) => println!("✓ API Key 验证成功"),
    Ok(false) => println!("✗ API Key 无效（请检查是否正确）"),
    Err(e) => println!("✗ 验证失败：网络错误或服务不可用"),
    // 不显示完整错误，避免泄露 endpoint 等信息
}
```

---

## 实现计划

### Phase 1: 基础框架（Day 1）
- [ ] 创建 `src/wizard/` 模块
- [ ] 实现 `ConfigWizard` 核心结构
- [ ] 添加 `dialoguer` 依赖
- [ ] 实现基本交互流程（无验证）

### Phase 2: 验证功能（Day 2）
- [ ] 实现 `ApiValidator`
- [ ] Deepseek API Key 验证
- [ ] Ollama 服务检测
- [ ] 错误处理与重试

### Phase 3: 配置生成（Day 3）
- [ ] 实现 `ConfigGenerator`
- [ ] YAML 配置模板
- [ ] .env 文件生成
- [ ] 文件权限设置

### Phase 4: 测试与优化（Day 4）
- [ ] 单元测试（validator, generator）
- [ ] 集成测试（完整流程）
- [ ] Sandbox 测试环境
- [ ] 文档更新

---

## 测试策略

### 单元测试

```rust
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
        assert!(yaml.contains("provider: deepseek"));
        assert!(yaml.contains("model: deepseek-chat"));
        assert!(!yaml.contains("sk-test")); // API key 应使用环境变量
    }

    #[tokio::test]
    async fn test_validate_deepseek_key_invalid() {
        let validator = ApiValidator::new();
        let result = validator
            .validate_deepseek_key("sk-invalid", "https://api.deepseek.com/v1")
            .await;

        assert!(result.is_err() || result.unwrap() == false);
    }
}
```

### Sandbox 测试

```bash
# sandbox/ 目录结构
sandbox/
├── test-init/          # 测试初始化（无现有配置）
├── test-update/        # 测试更新（有现有配置）
└── test-invalid/       # 测试错误处理
```

---

## 用户体验优化

### 进度反馈

```rust
use indicatif::{ProgressBar, ProgressStyle};

// 验证 API Key 时显示 spinner
let spinner = ProgressBar::new_spinner();
spinner.set_style(
    ProgressStyle::default_spinner()
        .template("{spinner:.green} {msg}")
        .unwrap()
);
spinner.set_message("正在验证 API Key...");

let result = validate_api_key(&api_key).await;
spinner.finish_with_message(
    if result.is_ok() { "✓ 验证成功" } else { "✗ 验证失败" }
);
```

### 错误恢复

```rust
// 验证失败时允许重试
loop {
    let api_key = prompt_api_key()?;

    match validate_api_key(&api_key).await {
        Ok(true) => break,
        Ok(false) => {
            println!("✗ API Key 无效");
            if !Confirm::new()
                .with_prompt("重新输入？")
                .default(true)
                .interact()?
            {
                return Err(anyhow!("用户取消"));
            }
        }
        Err(e) => {
            println!("✗ 验证失败: {}", e);
            // 提供跳过选项（用于网络问题）
            if Confirm::new()
                .with_prompt("跳过验证（不推荐）？")
                .default(false)
                .interact()?
            {
                break;
            }
        }
    }
}
```

---

## 命令行集成

### 新增子命令

```bash
# 运行配置向导
realconsole wizard

# 或
realconsole init

# 快速模式
realconsole wizard --quick

# 更新现有配置
realconsole wizard --update

# 显示当前配置
realconsole config show
```

### 首次运行检测

```rust
// 在 main() 中
if !config_exists() {
    println!("未检测到配置文件，启动配置向导...\n");

    if Confirm::new()
        .with_prompt("运行配置向导？")
        .default(true)
        .interact()?
    {
        run_wizard()?;
    } else {
        println!("提示：稍后可运行 `realconsole wizard` 创建配置");
        return Ok(());
    }
}
```

---

## 可访问性

1. **键盘导航**: 所有操作支持键盘（↑↓ 选择，Enter 确认）
2. **颜色对比**: 使用高对比度主题
3. **屏幕阅读器**: 提供纯文本输出模式（`--no-color`）
4. **多语言支持**: 预留 i18n 钩子（未来）

---

## 未来扩展

1. **配置验证**: `realconsole config validate`
2. **配置迁移**: 自动迁移旧版本配置
3. **云配置**: 支持从云端同步配置（v0.7+）
4. **配置模板**: 预设场景配置（开发、生产、教学等）

---

**文档版本**: v1.0
**编写日期**: 2025-10-15
**作者**: RealConsole Team
**状态**: 待审核 → 实现中
