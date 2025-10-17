# Phase 9.2 实施计划 - Agent错误修复集成

**文档版本**: v1.0
**创建日期**: 2025-10-17
**当前状态**: 部分完成 (结构体集成完成，功能实现待完成)

## 📋 项目目标

将Phase 9.1开发的错误自动修复系统集成到Agent主循环，实现：
1. Shell命令执行失败时自动分析和建议修复
2. 交互式修复流程（显示建议→用户选择→执行→记录反馈）
3. `/fix`命令用于手动重试失败命令
4. 从用户反馈中学习，优化修复策略排序

## ✅ 已完成工作

### 1. Agent结构体增强 (已完成)

**文件**: `src/agent.rs`
**修改位置**: Lines 41-73

```rust
// ✨ Phase 9.2: 错误自动修复支持
use crate::shell_executor::ShellExecutorWithFixer;
use crate::error_fixer::{FeedbackLearner, FeedbackType, FixOutcome};

pub struct Agent {
    // ... 其他字段 ...

    // ✨ Phase 9.2: Shell执行器（带错误修复）
    pub shell_executor_with_fixer: Arc<ShellExecutorWithFixer>,
    // 最后失败的命令（用于/fix命令）
    pub last_failed_command: Arc<RwLock<Option<String>>>,
}
```

**设计说明**:
- `shell_executor_with_fixer`: 包含错误分析、修复策略生成、反馈学习的完整执行器
- `last_failed_command`: 记录最后失败的命令，供`/fix`命令使用

### 2. Agent初始化逻辑 (已完成)

**文件**: `src/agent.rs`
**修改位置**: Lines 148-216

```rust
// ✨ Phase 9.2: 初始化错误修复系统
let feedback_learner = Arc::new(FeedbackLearner::new());
// 如果配置了持久化路径，设置存储路径
if let Some(ref config_dir) = dirs::config_dir() {
    let storage_path = config_dir.join("realconsole").join("feedback.json");
    let learner_with_storage = FeedbackLearner::new().with_storage(storage_path);
    // 尝试从磁盘加载历史反馈
    let _ = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            learner_with_storage.load_from_disk().await
        })
    });
    let feedback_learner = Arc::new(learner_with_storage);

    let shell_executor_with_fixer = Arc::new(
        ShellExecutorWithFixer::new()
            .with_feedback_learner(feedback_learner)
    );

    // ... 返回Agent实例
}
```

**功能特性**:
- ✅ 自动持久化到 `~/.config/realconsole/feedback.json`
- ✅ 启动时自动加载历史反馈数据
- ✅ Fallback支持（无config_dir时使用内存存储）

### 3. 编译状态 (已完成)

- ✅ 编译通过，无错误
- ⚠️ 未使用字段警告（预期的，因为字段尚未在方法中使用）

## 🚧 待完成工作

### 步骤1: 修改handle_shell()方法使用ShellExecutorWithFixer

**文件**: `src/agent.rs`
**当前位置**: Lines 410-433
**预计工作量**: 30分钟

#### 当前代码
```rust
/// 处理 Shell 命令
fn handle_shell(&self, cmd: &str) -> String {
    if !self.config.features.shell_enabled {
        return format!("{}", "Shell 执行已禁用".red());
    }

    // 特殊处理：cd 命令需要在主进程中生效
    let cmd_trimmed = cmd.trim();
    if cmd_trimmed.starts_with("cd ") || cmd_trimmed == "cd" {
        return self.handle_cd_command(cmd_trimmed);
    }

    // 使用 block_in_place 在同步上下文中调用异步代码
    match tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            crate::shell_executor::execute_shell(cmd).await
        })
    }) {
        Ok(output) => output,
        Err(e) => {
            // 使用用户友好的错误格式
            e.format_user_friendly()
        }
    }
}
```

#### 修改后代码
```rust
/// 处理 Shell 命令 (✨ Phase 9.2: 带错误修复支持)
fn handle_shell(&self, cmd: &str) -> String {
    if !self.config.features.shell_enabled {
        return format!("{}", "Shell 执行已禁用".red());
    }

    // 特殊处理：cd 命令需要在主进程中生效
    let cmd_trimmed = cmd.trim();
    if cmd_trimmed.starts_with("cd ") || cmd_trimmed == "cd" {
        return self.handle_cd_command(cmd_trimmed);
    }

    // ✨ Phase 9.2: 使用带错误修复的执行器
    match tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            self.shell_executor_with_fixer.execute_with_analysis(cmd).await
        })
    }) {
        execution_result => {
            // 如果执行失败且有修复建议，启动交互式修复流程
            if !execution_result.success && !execution_result.fix_suggestions.is_empty() {
                // 保存失败的命令供/fix使用
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let mut last_cmd = self.last_failed_command.write().await;
                        *last_cmd = Some(cmd.to_string());
                    })
                });

                // 显示错误和修复建议
                self.display_fix_suggestions(&execution_result)
            } else {
                // 正常输出
                execution_result.output
            }
        }
    }
}
```

#### 关键变更点
1. **从execute_shell改为execute_with_analysis**: 获取包含修复建议的ExecutionResult
2. **失败检测**: 检查`!execution_result.success && !execution_result.fix_suggestions.is_empty()`
3. **保存失败命令**: 写入`last_failed_command`供`/fix`命令使用
4. **调用display_fix_suggestions**: 显示修复建议并处理用户交互

---

### 步骤2: 实现display_fix_suggestions()交互式修复

**文件**: `src/agent.rs`
**插入位置**: Lines 500+ (在handle_shell之后)
**预计工作量**: 1小时

#### 完整实现代码

```rust
/// 显示修复建议并处理用户交互 (✨ Phase 9.2)
///
/// 实现交互式修复流程：
/// 1. 显示错误信息和分析
/// 2. 列出所有修复建议（按效能分数排序）
/// 3. 用户选择建议或跳过
/// 4. 执行选择的修复
/// 5. 记录反馈到学习系统
fn display_fix_suggestions(&self, result: &crate::shell_executor::ExecutionResult) -> String {
    use colored::Colorize;
    use std::io::{self, Write};

    let mut output = String::new();

    // 1. 显示原始错误
    output.push_str(&format!("\n{}\n", "❌ 命令执行失败".red().bold()));
    output.push_str(&format!("{}\n\n", result.output));

    // 2. 显示错误分析（如果有）
    if let Some(ref analysis) = result.error_analysis {
        output.push_str(&format!("{}\n", "🔍 错误分析".cyan().bold()));
        output.push_str(&format!("  类型: {:?}\n", analysis.category));
        output.push_str(&format!("  严重程度: {:?}\n", analysis.severity));

        if !analysis.possible_causes.is_empty() {
            output.push_str(&format!("\n  可能原因:\n"));
            for cause in &analysis.possible_causes {
                output.push_str(&format!("    • {}\n", cause.dimmed()));
            }
        }
        output.push_str("\n");
    }

    // 3. 显示修复建议列表
    output.push_str(&format!("{}\n", "💡 修复建议".green().bold()));
    for (idx, strategy) in result.fix_suggestions.iter().enumerate() {
        let risk_indicator = if strategy.risk_level < 5 {
            "🟢 低风险".green()
        } else if strategy.risk_level < 8 {
            "🟡 中风险".yellow()
        } else {
            "🔴 高风险".red()
        };

        output.push_str(&format!(
            "\n  [{}] {} - {}\n",
            idx + 1,
            strategy.name.bold(),
            risk_indicator
        ));
        output.push_str(&format!("      命令: {}\n", strategy.command.cyan()));
        output.push_str(&format!("      说明: {}\n", strategy.description.dimmed()));
        output.push_str(&format!("      预期: {}\n", strategy.expected_outcome.dimmed()));
    }

    // 4. 打印到stdout（因为需要交互）
    println!("{}", output);

    // 5. 提示用户选择
    println!("\n{}", "请选择修复方案:".yellow().bold());
    println!("  {} - 执行对应的修复建议", "1-N".cyan());
    println!("  {} - 跳过，不修复", "s".cyan());
    println!("  {} - 取消，返回", "c".cyan());
    print!("\n选择 [1-{}/s/c]: ", result.fix_suggestions.len());
    let _ = io::stdout().flush();

    // 6. 读取用户输入
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return "输入失败".to_string();
    }

    let choice = input.trim().to_lowercase();

    // 7. 处理用户选择
    match choice.as_str() {
        "s" | "skip" => {
            // 用户跳过修复
            self.record_fix_feedback_skipped(result);
            "\n⏭  已跳过修复".yellow().to_string()
        }
        "c" | "cancel" => {
            // 用户取消
            "\n❌ 已取消".red().to_string()
        }
        _ => {
            // 解析数字选择
            if let Ok(idx) = choice.parse::<usize>() {
                if idx > 0 && idx <= result.fix_suggestions.len() {
                    let selected = &result.fix_suggestions[idx - 1];
                    self.execute_fix_strategy(selected, result)
                } else {
                    "\n❌ 无效的选择".red().to_string()
                }
            } else {
                "\n❌ 无效的输入，请输入数字、's'或'c'".red().to_string()
            }
        }
    }
}

/// 执行选定的修复策略 (✨ Phase 9.2)
fn execute_fix_strategy(
    &self,
    strategy: &crate::error_fixer::FixStrategy,
    original_result: &crate::shell_executor::ExecutionResult,
) -> String {
    use colored::Colorize;

    println!("\n{} {}", "🔧 执行修复:".cyan().bold(), strategy.command.cyan());

    // 执行修复命令
    let fix_result = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            crate::shell_executor::execute_shell(&strategy.command).await
        })
    });

    match fix_result {
        Ok(output) => {
            // 修复成功
            println!("\n{}", "✅ 修复执行成功".green().bold());

            // 记录成功的反馈
            if let Some(ref analysis) = original_result.error_analysis {
                self.record_fix_feedback_success(analysis, strategy);
            }

            format!("\n{}\n", output)
        }
        Err(e) => {
            // 修复失败
            println!("\n{}", "❌ 修复执行失败".red().bold());

            // 记录失败的反馈
            if let Some(ref analysis) = original_result.error_analysis {
                self.record_fix_feedback_failure(analysis, strategy);
            }

            format!("\n{}\n", e.format_user_friendly())
        }
    }
}

/// 记录用户跳过修复的反馈 (✨ Phase 9.2)
fn record_fix_feedback_skipped(&self, result: &crate::shell_executor::ExecutionResult) {
    if let Some(ref analysis) = result.error_analysis {
        for strategy in &result.fix_suggestions {
            tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    self.shell_executor_with_fixer
                        .record_feedback(analysis, strategy, FeedbackType::Skipped, FixOutcome::Unknown)
                        .await;
                })
            });
        }
    }
}

/// 记录修复成功的反馈 (✨ Phase 9.2)
fn record_fix_feedback_success(
    &self,
    analysis: &crate::error_fixer::ErrorAnalysis,
    strategy: &crate::error_fixer::FixStrategy,
) {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            self.shell_executor_with_fixer
                .record_feedback(analysis, strategy, FeedbackType::Accepted, FixOutcome::Success)
                .await;
        })
    });
}

/// 记录修复失败的反馈 (✨ Phase 9.2)
fn record_fix_feedback_failure(
    &self,
    analysis: &crate::error_fixer::ErrorAnalysis,
    strategy: &crate::error_fixer::FixStrategy,
) {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            self.shell_executor_with_fixer
                .record_feedback(analysis, strategy, FeedbackType::Accepted, FixOutcome::Failure)
                .await;
        })
    });
}
```

#### 实现说明

**设计哲学（一分为三）**:
1. **展示层**: display_fix_suggestions() - 显示错误分析和修复建议
2. **交互层**: 处理用户输入（1-N/s/c）
3. **执行层**: execute_fix_strategy() - 执行修复并记录反馈

**关键特性**:
- ✅ **彩色输出**: 风险等级用🟢🟡🔴标识
- ✅ **详细信息**: 显示命令、说明、预期结果
- ✅ **交互友好**: 清晰的选项提示
- ✅ **反馈记录**: 自动记录到学习系统
- ✅ **三种选择**: 执行修复/跳过/取消

---

### 步骤3: 添加/fix命令

**文件**: `src/commands/mod.rs`
**预计工作量**: 30分钟

#### 1. 创建fix命令处理器

**新建文件**: `src/commands/fix_cmd.rs`

```rust
//! /fix 命令 - 重试最后失败的命令并尝试修复
//!
//! ✨ Phase 9.2: 错误修复命令

use crate::agent::Agent;
use colored::Colorize;

/// 处理 /fix 命令
///
/// 功能：
/// - 重新执行最后一个失败的命令
/// - 自动分析错误并提供修复建议
/// - 支持交互式修复流程
pub fn handle_fix(agent: &Agent, _arg: &str) -> String {
    // 获取最后失败的命令
    let last_cmd = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let cmd_guard = agent.last_failed_command.read().await;
            cmd_guard.clone()
        })
    });

    match last_cmd {
        Some(cmd) => {
            println!("{} {}", "🔄 重试命令:".cyan().bold(), cmd.cyan());

            // 直接调用handle_shell重新执行（会自动触发错误分析和修复建议）
            agent.handle(&format!("!{}", cmd))
        }
        None => {
            format!(
                "{}\n{}",
                "❌ 没有可重试的失败命令".red(),
                "提示: 执行一个失败的命令后再使用 /fix".dimmed()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandRegistry;
    use crate::config::Config;

    #[tokio::test]
    async fn test_fix_no_previous_command() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        let result = handle_fix(&agent, "");
        assert!(result.contains("没有可重试的失败命令"));
    }

    #[tokio::test]
    async fn test_fix_with_previous_command() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 先执行一个会失败的命令
        agent.handle("!nonexistent_cmd_xyz");

        // 验证last_failed_command被设置
        let last_cmd = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let guard = agent.last_failed_command.read().await;
                guard.clone()
            })
        });

        assert!(last_cmd.is_some());
    }
}
```

#### 2. 注册fix命令

**文件**: `src/commands/mod.rs`

```rust
// 在文件开头添加
pub mod fix_cmd;

// 在 register_commands() 函数中添加
pub fn register_commands(
    agent: &Agent,
    stats_collector: &Arc<StatsCollector>,
) -> CommandRegistry {
    // ... 其他命令 ...

    // ✨ Phase 9.2: 错误修复命令
    {
        let agent_clone = agent.clone(); // 需要Agent实现Clone或使用Arc
        registry.register(Command::from_fn(
            "fix",
            "重试最后失败的命令并尝试自动修复",
            move |arg| fix_cmd::handle_fix(&agent_clone, arg),
        ));
    }

    // ... 返回registry
}
```

**注意**: 这需要Agent实现Clone trait或将Agent包装在Arc中传递。

#### 替代方案（推荐）

由于Agent结构较复杂，直接在`handle_command()`中特殊处理`/fix`:

**文件**: `src/agent.rs`，`handle_command()`方法

```rust
/// 处理命令
fn handle_command(&self, input: &str) -> String {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd_name = parts[0];
    let arg = parts.get(1).copied().unwrap_or("");

    // ✨ Phase 9.2: 特殊处理 /fix 命令
    if cmd_name == "fix" {
        return self.handle_fix_command();
    }

    match self.registry.execute(cmd_name, arg) {
        Ok(output) => output,
        Err(err) => format!("{}", err.red()),
    }
}

/// 处理 /fix 命令 (✨ Phase 9.2)
fn handle_fix_command(&self) -> String {
    // 获取最后失败的命令
    let last_cmd = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let cmd_guard = self.last_failed_command.read().await;
            cmd_guard.clone()
        })
    });

    match last_cmd {
        Some(cmd) => {
            println!("{} {}", "🔄 重试命令:".cyan().bold(), cmd.cyan());

            // 重新执行失败的命令
            self.handle_shell(&cmd)
        }
        None => {
            format!(
                "{}\n{}",
                "❌ 没有可重试的失败命令".red(),
                "提示: 执行一个失败的命令后再使用 /fix".dimmed()
            )
        }
    }
}
```

**优点**:
- ✅ 无需修改CommandRegistry
- ✅ 直接访问Agent的所有字段
- ✅ 实现简单，代码集中

---

### 步骤4: 添加/fix命令到help

**文件**: `src/commands/core.rs` 或相应的help命令处理器

```rust
// 在help命令输出中添加
"  /fix                    - 重试最后失败的命令并尝试自动修复"
```

---

### 步骤5: 测试Agent错误修复集成

**文件**: `src/agent.rs` (在tests module中添加)

```rust
#[cfg(test)]
mod tests {
    // ... 现有测试 ...

    // ========== Phase 9.2: 错误修复测试 ==========

    #[tokio::test(flavor = "multi_thread")]
    async fn test_shell_error_analysis() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 执行一个会失败的命令
        let result = agent.handle("!nonexistent_command_xyz");

        // 应该包含错误信息
        assert!(!result.is_empty());

        // 验证last_failed_command被设置
        let last_cmd = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let guard = agent.last_failed_command.read().await;
                guard.clone()
            })
        });

        assert!(last_cmd.is_some());
        assert_eq!(last_cmd.unwrap(), "nonexistent_command_xyz");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fix_command_no_previous_failure() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 直接调用/fix（没有之前的失败命令）
        let result = agent.handle("/fix");

        assert!(result.contains("没有可重试的失败命令"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_fix_command_with_previous_failure() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 1. 执行一个会失败的命令
        agent.handle("!some_failing_command");

        // 2. 验证最后失败的命令被记录
        let last_cmd = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let guard = agent.last_failed_command.read().await;
                guard.clone()
            })
        });
        assert!(last_cmd.is_some());

        // 3. 调用/fix（注意：这会再次尝试执行失败的命令）
        // 由于是自动化测试，我们只验证命令不会panic
        let result = agent.handle("/fix");
        assert!(!result.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_feedback_learner_persistence() {
        let mut config = Config::default();
        config.features.shell_enabled = true;
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 验证feedback_learner已初始化
        let learner = agent.shell_executor_with_fixer.feedback_learner();

        // 获取学习摘要
        let summary = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                learner.get_summary().await
            })
        });

        // 初始状态应该有0条记录
        assert_eq!(summary.total_records, 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_shell_executor_with_fixer_creation() {
        let config = Config::default();
        let registry = CommandRegistry::new();
        let agent = Agent::new(config, registry);

        // 验证shell_executor_with_fixer已创建
        assert!(Arc::strong_count(&agent.shell_executor_with_fixer) >= 1);
    }
}
```

---

## 📊 完整实施检查清单

### Phase 9.2.1: 基础集成 ✅
- [x] Agent结构体增加字段
- [x] Agent初始化逻辑
- [x] 持久化配置
- [x] 编译通过

### Phase 9.2.2: 核心功能 🚧
- [ ] 修改handle_shell()使用ShellExecutorWithFixer
- [ ] 实现display_fix_suggestions()交互流程
- [ ] 实现execute_fix_strategy()执行修复
- [ ] 实现feedback记录方法（3个）

### Phase 9.2.3: /fix命令 🚧
- [ ] 实现handle_fix_command()方法
- [ ] 在handle_command()中特殊处理/fix
- [ ] 更新help命令文档

### Phase 9.2.4: 测试与文档 🚧
- [ ] 添加5个集成测试
- [ ] 手动测试错误修复流程
- [ ] 测试/fix命令
- [ ] 更新用户文档

---

## 🎯 预期效果

### 用户体验示例

```bash
# 场景1: 命令不存在
» !tree
❌ 命令执行失败
bash: tree: command not found

🔍 错误分析
  类型: CommandNotFound
  严重程度: Warning

  可能原因:
    • 命令 'tree' 未安装
    • 命令路径不在 PATH 环境变量中

💡 修复建议

  [1] 使用包管理器安装tree - 🟢 低风险
      命令: brew install tree
      说明: 在macOS上使用Homebrew安装tree命令
      预期: 安装成功后可以使用tree命令

  [2] 使用find替代tree - 🟢 低风险
      命令: find . -print | sed -e 's;[^/]*/;|____;g;s;____|;  |;g'
      说明: 使用find和sed模拟tree的输出
      预期: 显示目录树结构

请选择修复方案:
  1-N - 执行对应的修复建议
  s - 跳过，不修复
  c - 取消，返回

选择 [1-2/s/c]: 1

🔧 执行修复: brew install tree

✅ 修复执行成功

==> Downloading https://...
==> Installing tree
🍺  tree was successfully installed
```

```bash
# 场景2: 使用/fix命令重试
» !python script.py
ModuleNotFoundError: No module named 'requests'

💡 修复建议
  [1] 安装requests模块 - 🟢 低风险
      命令: pip install requests
      ...

选择 [1-2/s/c]: s
⏭  已跳过修复

» /fix
🔄 重试命令: python script.py

ModuleNotFoundError: No module named 'requests'

💡 修复建议
  [1] 安装requests模块 - 🟢 低风险 (⭐ 推荐)
      ...

选择 [1-2/s/c]: 1

🔧 执行修复: pip install requests
✅ 修复执行成功
Successfully installed requests-2.31.0
```

---

## 📈 性能与优化

### 性能指标
- **错误分析**: < 10ms (正则匹配)
- **LLM增强分析**: ~500-2000ms (可选，默认关闭)
- **策略生成**: < 5ms (规则引擎)
- **反馈记录**: < 1ms (异步后台)
- **持久化**: < 10ms (异步保存)

### 优化建议
1. **缓存常见错误模式**: 避免重复分析
2. **异步反馈记录**: 不阻塞用户体验
3. **批量持久化**: 每N条或每M秒保存一次
4. **LRU策略**: 限制内存中的反馈记录数量

---

## 🔄 下一步计划 (Phase 10)

完成Phase 9.2后，进入Phase 10: 任务分解与规划系统

### 核心功能
1. **TaskDecomposer**: LLM驱动的任务分解
2. **TaskPlanner**: 依赖分析和执行计划
3. **TaskExecutor**: 多步骤任务执行引擎

### 集成点
- 在Agent中添加`task_system`字段
- 新增`/plan <描述>`命令：分解任务
- 新增`/execute [任务ID]`命令：执行任务
- 与错误修复系统集成：失败任务自动修复

---

## 📝 实施注意事项

### 代码质量
- ✅ 所有新方法添加文档注释
- ✅ 使用`colored` crate实现彩色输出
- ✅ 遵循Rust命名规范
- ✅ 添加`#[cfg(test)]`测试
- ✅ 使用`tokio::task::block_in_place`处理异步调用

### 用户体验
- ✅ 清晰的错误提示
- ✅ 友好的交互界面
- ✅ 风险等级可视化（🟢🟡🔴）
- ✅ 支持取消和跳过
- ✅ 自动保存学习结果

### 安全性
- ✅ 三层安全验证（继承自ShellExecutorWithFixer）
- ✅ 高风险命令需要确认（risk_level >= 5）
- ✅ 危险命令自动过滤
- ✅ 用户明确知晓将执行的命令

---

## 📚 相关文档

- **设计文档**: `docs/03-evolution/phases/phase9.1-week2-error-auto-fixing.md`
- **学习系统**: `docs/03-evolution/phases/phase9.1-week3-feedback-learning.md`
- **CHANGELOG**: `docs/CHANGELOG.md` (v0.9.2 section)
- **用户指南**: `docs/02-practice/user/user-guide.md` (需更新)

---

**最后更新**: 2025-10-17
**状态**: 待实施
**预计完成时间**: 3-4小时开发 + 1小时测试
